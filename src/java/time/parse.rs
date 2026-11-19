//! Parsing with a `DateTimeFormatter`: `DateTimeParseContext`, the `parse` method of each printer-parser of
//! `DateTimeFormatterBuilder` in strict mode (the mode of `ofPattern` and `ISO_INSTANT`), and `Parsed.resolve` on
//! the ISO chronology with the SMART (`ofPattern`) or STRICT (`ISO_INSTANT`) resolver.
//!
//! `Parsed.fieldValues` is a `HashMap` upstream whose iteration order follows identity hash codes, so it varies
//! between JVM runs; here the fields keep their insertion order, and `Display` prints them sorted by name.

use std::fmt;

use num_bigint::BigInt;

use super::formatter::{
    DateTimeFormatter, Field, Item, Number, SignStyle, TextStyle, compute_week, date_field, localized_dow_of, start_of_week_offset,
    time_field, week_of_month, MINIMAL_DAYS,
};
use super::zone::ZoneId;
use super::{Instant, LocalDate, LocalDateTime, LocalTime, MONTHS, WEEKDAYS};
use crate::{Error, Result};

/// A parse position: the index after what was parsed, or `!index` of the error.
type Pos = i64;

fn date_time(message: impl Into<String>) -> Error {
    Error::new("DateTimeException", message)
}

/// `DateTimeFormatterBuilder.DayPeriod` of `en` (`midnight:00:00;noon:12:00;morning1:00:00-12:00;afternoon1:12:00-18:00;
/// evening1:18:00-21:00;night1:21:00-24:00`, then `am` and `pm`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DayPeriod {
    from: i64,
    to: i64,
}

impl DayPeriod {
    fn of_index(index: i64) -> DayPeriod {
        let (from, to) = match index {
            0 => (0, 720),
            1 => (720, 1440),
            2 => (0, 0),
            3 => (720, 720),
            4 => (0, 720),
            6 => (720, 1080),
            8 => (1080, 1260),
            _ => (1260, 1440),
        };
        DayPeriod { from, to }
    }

    fn duration(&self) -> i64 {
        if self.from > self.to { 1440 - self.from + self.to } else { self.to - self.from }
    }

    fn mid(&self) -> i64 {
        (self.from + self.duration() / 2) % 1440
    }

    fn includes(&self, m: i64) -> bool {
        if self.from == 0 && self.to == 0 && m == 1440 {
            return true;
        }
        self.from == m && self.to == m || self.from <= m && m < self.to || self.from > self.to && (self.from <= m || self.to > m)
    }
}

impl fmt::Display for DayPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DayPeriod({:02}:{:02}", self.from / 60, self.from % 60)?;
        if self.from == self.to { f.write_str(")") } else { write!(f, "-{:02}:{:02})", self.to / 60, self.to % 60) }
    }
}

/// `ZoneTextPrinterParser.UNDEFINED`, `STD`, `GENERIC`.
const ZONE_NAME_UNDEFINED: i32 = -1;
const ZONE_NAME_STD: i32 = 0;
const ZONE_NAME_GENERIC: i32 = 2;

/// `java.time.format.Parsed`: what a formatter parsed, resolved.
#[derive(Debug, Clone)]
pub struct Parsed {
    fields: Vec<(Field, i64)>,
    zone: Option<ZoneId>,
    zone_name_type: i32,
    leap_second: bool,
    day_period: Option<DayPeriod>,
    date: Option<LocalDate>,
    time: Option<LocalTime>,
    excess_days: i64,
    strict: bool,
}

impl Parsed {
    fn new() -> Parsed {
        Parsed {
            fields: Vec::new(),
            zone: None,
            zone_name_type: ZONE_NAME_UNDEFINED,
            leap_second: false,
            day_period: None,
            date: None,
            time: None,
            excess_days: 0,
            strict: false,
        }
    }

    fn get(&self, field: Field) -> Option<i64> {
        self.fields.iter().find(|(f, _)| *f == field).map(|(_, v)| *v)
    }

    fn contains(&self, field: Field) -> bool {
        self.get(field).is_some()
    }

    /// `fieldValues.put`: the old value; an existing key keeps its place.
    fn put(&mut self, field: Field, value: i64) -> Option<i64> {
        match self.fields.iter_mut().find(|(f, _)| *f == field) {
            Some(entry) => Some(std::mem::replace(&mut entry.1, value)),
            None => {
                self.fields.push((field, value));
                None
            }
        }
    }

    fn remove(&mut self, field: Field) -> Option<i64> {
        let i = self.fields.iter().position(|(f, _)| *f == field)?;
        Some(self.fields.remove(i).1)
    }

    /// `query(TemporalQueries.localDate())`.
    pub fn local_date(&self) -> Option<LocalDate> {
        self.date
    }

    /// `query(TemporalQueries.localTime())`.
    pub fn local_time(&self) -> Option<LocalTime> {
        self.time
    }

    /// `query(TemporalQueries.zoneId())`.
    pub fn zone(&self) -> Option<&ZoneId> {
        self.zone.as_ref()
    }

    /// The `Parsed.leapSecond` flag (`DateTimeFormatter.parsedLeapSecond()`).
    pub fn leap_second(&self) -> bool {
        self.leap_second
    }

    /// `getLong(field)` by the field's name (`"InstantSeconds"`, `"HourOfAmPm"`, …): an unresolved field, or one the
    /// resolved date or time has.
    pub fn get_long(&self, name: &str) -> Option<i64> {
        let field = ALL_FIELDS.iter().copied().find(|f| f.name() == name)?;
        self.get(field)
            .or_else(|| self.date.and_then(|d| date_field(&d, field).filter(|_| date_supports(field))))
            .or_else(|| self.time.and_then(|t| time_field(&t, field)))
    }

    /// `Instant.from(this)`.
    pub fn to_instant(&self) -> Result<Instant> {
        let fail = |cause: Option<Error>| {
            let e = date_time(format!("Unable to obtain Instant from TemporalAccessor: {self} of type java.time.format.Parsed"));
            match cause {
                Some(c) => e.with_cause(c),
                None => e,
            }
        };
        let (Some(seconds), Some(nanos)) = (self.get_long("InstantSeconds"), self.get_long("NanoOfSecond")) else {
            return Err(fail(None));
        };
        Instant::of_epoch_second(seconds, nanos).map_err(|e| fail(Some(e)))
    }

    fn unable(&self, what: &str) -> Error {
        date_time(format!("Unable to obtain {what} from TemporalAccessor: {self} of type java.time.format.Parsed"))
    }
}

const ALL_FIELDS: [Field; 36] = [
    Field::NanoOfSecond,
    Field::NanoOfDay,
    Field::MicroOfSecond,
    Field::MicroOfDay,
    Field::MilliOfSecond,
    Field::MilliOfDay,
    Field::SecondOfMinute,
    Field::SecondOfDay,
    Field::MinuteOfHour,
    Field::MinuteOfDay,
    Field::HourOfAmPm,
    Field::ClockHourOfAmPm,
    Field::HourOfDay,
    Field::ClockHourOfDay,
    Field::AmPmOfDay,
    Field::DayOfWeek,
    Field::AlignedDayOfWeekInMonth,
    Field::AlignedDayOfWeekInYear,
    Field::DayOfMonth,
    Field::DayOfYear,
    Field::EpochDay,
    Field::AlignedWeekOfMonth,
    Field::AlignedWeekOfYear,
    Field::MonthOfYear,
    Field::ProlepticMonth,
    Field::YearOfEra,
    Field::Year,
    Field::Era,
    Field::InstantSeconds,
    Field::OffsetSeconds,
    Field::QuarterOfYear,
    Field::ModifiedJulianDay,
    Field::WeekBasedYear,
    Field::WeekOfWeekBasedYear,
    Field::WeekOfMonth,
    Field::LocalizedDayOfWeek,
];

/// `LocalDate.isSupported(field)`.
fn date_supports(field: Field) -> bool {
    field.is_date_based() || !field.is_chrono()
}

impl fmt::Display for Parsed {
    /// `toString()`, the fields sorted by name: `{HourOfAmPm=10, MinuteOfHour=13},ISO resolved to 2023-11-14`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fields: Vec<(&str, i64)> = self.fields.iter().map(|(k, v)| (k.name(), *v)).collect();
        fields.sort();
        let map: Vec<String> = fields.iter().map(|(k, v)| format!("{k}={v}")).collect();
        write!(f, "{{{}}},ISO", map.join(", "))?;
        if let Some(zone) = &self.zone {
            write!(f, ",{zone},{}", self.zone_name_type)?;
        }
        match (self.date, self.time) {
            (Some(d), Some(t)) => write!(f, " resolved to {d}T{t}"),
            (Some(d), None) => write!(f, " resolved to {d}"),
            (None, Some(t)) => write!(f, " resolved to {t}"),
            (None, None) => Ok(()),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// DateTimeParseContext

struct Context {
    case_sensitive: bool,
    parsed: Vec<Parsed>,
}

impl Context {
    fn current(&mut self) -> &mut Parsed {
        self.parsed.last_mut().expect("a parsed")
    }

    /// `setParsedField`: `successPos`, or `!errorPos` when the field already had another value.
    fn set_parsed_field(&mut self, field: Field, value: i64, error_pos: usize, success_pos: usize) -> Pos {
        match self.current().put(field, value) {
            Some(old) if old != value => !(error_pos as i64),
            _ => success_pos as i64,
        }
    }

    fn char_equals(&self, a: u16, b: u16) -> bool {
        a == b || !self.case_sensitive && char_equals_ignore_case(a, b)
    }

    /// `subSequenceEquals(cs1, offset1, cs2, offset2, length)`.
    fn sub_sequence_equals(&self, cs1: &[u16], offset1: usize, cs2: &[u16], offset2: usize, length: usize) -> bool {
        if offset1 + length > cs1.len() || offset2 + length > cs2.len() {
            return false;
        }
        (0..length).all(|i| self.char_equals(cs1[offset1 + i], cs2[offset2 + i]))
    }
}

/// `Character.toUpperCase(char)` and `toLowerCase(char)`, simple mappings of one UTF-16 unit.
fn case_map(c: u16, upper: bool) -> u16 {
    let Some(ch) = char::from_u32(c as u32) else { return c };
    let mut mapped = if upper { ch.to_uppercase().collect::<Vec<_>>() } else { ch.to_lowercase().collect::<Vec<_>>() };
    if mapped.len() != 1 || (mapped[0] as u32) > 0xFFFF {
        return c;
    }
    mapped.pop().map_or(c, |m| m as u16)
}

/// `DateTimeParseContext.charEqualsIgnoreCase(c1, c2)`.
fn char_equals_ignore_case(a: u16, b: u16) -> bool {
    a == b || case_map(a, true) == case_map(b, true) || case_map(a, false) == case_map(b, false)
}

fn units(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// `String.charAt(index)`.
fn char_at(text: &[u16], index: usize) -> Result<u16> {
    text.get(index).copied().ok_or_else(|| {
        Error::new("StringIndexOutOfBoundsException", format!("Index {index} out of bounds for length {}", text.len()))
    })
}

fn digit(c: u16) -> Option<i64> {
    (b'0' as u16..=b'9' as u16).contains(&c).then(|| (c - b'0' as u16) as i64)
}

// ---------------------------------------------------------------------------------------------------------------
// DateTimeFormatter.parse

impl DateTimeFormatter {
    /// `parse(text)`: the resolved `Parsed`, or a `DateTimeParseException` with the JDK's text and index.
    pub fn parse(&self, text: &str) -> Result<Parsed> {
        let units = units(text);
        let abbr = if units.len() > 64 { format!("{}...", String::from_utf16_lossy(&units[..64])) } else { text.to_string() };
        let wrap = |e: Error| -> Error {
            if e.simple_class() == "DateTimeParseException" {
                return e;
            }
            Error::new("DateTimeParseException", format!("Text '{abbr}' could not be parsed: {}", e.message())).with_cause(e)
        };
        let mut ctx = Context { case_sensitive: self.case_sensitive, parsed: vec![Parsed::new()] };
        let pos = parse_items(&self.items, &mut ctx, &units, 0).map_err(wrap)?;
        if pos < 0 {
            return Err(Error::new("DateTimeParseException", format!("Text '{abbr}' could not be parsed at index {}", !pos)));
        }
        if (pos as usize) < units.len() {
            return Err(Error::new(
                "DateTimeParseException",
                format!("Text '{abbr}' could not be parsed, unparsed text found at index {pos}"),
            ));
        }
        let mut parsed = ctx.parsed.pop().expect("a parsed");
        if parsed.zone.is_none() {
            parsed.zone = self.zone.clone();
        }
        parsed.resolve(self.strict_resolver).map_err(wrap)?;
        Ok(parsed)
    }

    /// `parse(text, LocalDate::from)`.
    pub fn parse_local_date(&self, text: &str) -> Result<LocalDate> {
        self.parse_query(text, LocalDate::from_parsed)
    }

    /// `parse(text, LocalTime::from)`.
    pub fn parse_local_time(&self, text: &str) -> Result<LocalTime> {
        self.parse_query(text, LocalTime::from_parsed)
    }

    /// `parse(text, LocalDateTime::from)`.
    pub fn parse_local_date_time(&self, text: &str) -> Result<LocalDateTime> {
        self.parse_query(text, LocalDateTime::from_parsed)
    }

    /// `parse(text, query)`: an exception of the query is wrapped as one of parsing.
    pub(crate) fn parse_query<T>(&self, text: &str, query: impl FnOnce(&Parsed) -> Result<T>) -> Result<T> {
        let parsed = self.parse(text)?;
        query(&parsed).map_err(|e| {
            let units = units(text);
            let abbr = if units.len() > 64 { format!("{}...", String::from_utf16_lossy(&units[..64])) } else { text.to_string() };
            Error::new("DateTimeParseException", format!("Text '{abbr}' could not be parsed: {}", e.message())).with_cause(e)
        })
    }
}

impl LocalDate {
    /// `LocalDate.from(parsed)`.
    pub fn from_parsed(parsed: &Parsed) -> Result<LocalDate> {
        parsed.date.ok_or_else(|| parsed.unable("LocalDate"))
    }
}

impl LocalTime {
    /// `LocalTime.from(parsed)`.
    pub fn from_parsed(parsed: &Parsed) -> Result<LocalTime> {
        parsed.time.ok_or_else(|| parsed.unable("LocalTime"))
    }
}

impl LocalDateTime {
    /// `LocalDateTime.from(parsed)`.
    pub fn from_parsed(parsed: &Parsed) -> Result<LocalDateTime> {
        let both = LocalDate::from_parsed(parsed).and_then(|d| Ok(LocalDateTime::of(d, LocalTime::from_parsed(parsed)?)));
        both.map_err(|e| parsed.unable("LocalDateTime").with_cause(e))
    }
}

fn parse_items(items: &[Item], ctx: &mut Context, text: &[u16], position: usize) -> Result<Pos> {
    let mut pos = position as i64;
    for item in items {
        pos = parse_item(item, ctx, text, pos as usize)?;
        if pos < 0 {
            break;
        }
    }
    Ok(pos)
}

fn parse_item(item: &Item, ctx: &mut Context, text: &[u16], position: usize) -> Result<Pos> {
    let len = text.len();
    Ok(match item {
        Item::Literal(lit) if lit.len() == 1 => {
            if position == len {
                return Ok(!(position as i64));
            }
            if ctx.char_equals(text[position], lit[0]) { position as i64 + 1 } else { !(position as i64) }
        }
        Item::Literal(lit) => {
            if !ctx.sub_sequence_equals(text, position, lit, 0, lit.len()) {
                return Ok(!(position as i64));
            }
            (position + lit.len()) as i64
        }
        Item::Number(n) => parse_number(n, ctx, text, position),
        Item::Fraction { min, max, decimal_point, .. } => parse_fraction(*min, *max, *decimal_point, ctx, text, position),
        Item::Text(field, style) => {
            for (t, value) in text_entries(*field, *style) {
                if ctx.sub_sequence_equals(&t, 0, text, position, t.len()) {
                    return Ok(ctx.set_parsed_field(*field, value, position, position + t.len()));
                }
            }
            !(position as i64)
        }
        Item::ZoneText(style, _) => parse_zone(ctx, text, position, Some(*style))?,
        Item::ZoneId => parse_zone(ctx, text, position, None)?,
        Item::Offset(pattern, no_offset) => parse_offset(*pattern, no_offset, ctx, text, position)?,
        Item::LocalizedOffset(style) => parse_localized_offset(*style, ctx, text, position)?,
        Item::DayPeriod(style) => {
            for (t, index) in day_period_entries(*style) {
                if ctx.sub_sequence_equals(&t, 0, text, position, t.len()) {
                    ctx.current().day_period = Some(DayPeriod::of_index(index));
                    return Ok((position + t.len()) as i64);
                }
            }
            !(position as i64)
        }
        Item::Instant => parse_instant(ctx, text, position)?,
        Item::Optional(items) => {
            let copy = ctx.current().clone();
            ctx.parsed.push(copy);
            let pos = parse_items(items, ctx, text, position)?;
            if pos < 0 {
                ctx.parsed.pop();
                return Ok(position as i64);
            }
            let n = ctx.parsed.len();
            ctx.parsed.remove(n - 2);
            pos
        }
        Item::Pad(width, inner) => {
            if position == len {
                return Ok(!(position as i64));
            }
            let end = position + width;
            if end > len {
                return Ok(!(position as i64));
            }
            let mut pos = position;
            while pos < end && ctx.char_equals(text[pos], b' ' as u16) {
                pos += 1;
            }
            let result = parse_item(inner, ctx, &text[..end], pos)?;
            if result != end as i64 {
                return Ok(!((position + pos) as i64));
            }
            result
        }
    })
}

/// `NumberPrinterParser.parse` (strict), with `ReducedPrinterParser.setValue` on the base value 2000.
fn parse_number(n: &Number, ctx: &mut Context, text: &[u16], start: usize) -> Pos {
    let len = text.len();
    let mut position = start;
    if position == len {
        return !(position as i64);
    }
    let (mut negative, mut positive) = (false, false);
    match text[position] {
        0x2B => {
            if !n.sign.parse_strict(true) {
                return !(position as i64);
            }
            positive = true;
            position += 1;
        }
        0x2D => {
            if !n.sign.parse_strict(false) {
                return !(position as i64);
            }
            negative = true;
            position += 1;
        }
        _ => {}
    }
    let eff_min = n.min;
    let min_end = position + eff_min;
    if min_end > len {
        return !(position as i64);
    }
    let mut eff_max = n.max as i64 + n.subsequent.max(0) as i64;
    let mut total: i64 = 0;
    let mut big: Option<BigInt> = None;
    let mut pos = position;
    for pass in 0..2 {
        let max_end = ((pos as i64 + eff_max).min(len as i64)).max(0) as usize;
        while pos < max_end {
            let Some(d) = digit(text[pos]) else {
                if pos < min_end {
                    return !(position as i64);
                }
                break;
            };
            pos += 1;
            if pos - position > 18 {
                big = Some(big.take().unwrap_or_else(|| BigInt::from(total)) * 10 + d);
            } else {
                total = total * 10 + d;
            }
        }
        if n.subsequent > 0 && pass == 0 {
            let parse_len = (pos - position) as i64;
            eff_max = (eff_min as i64).max(parse_len - n.subsequent as i64);
            pos = position;
            total = 0;
            big = None;
        } else {
            break;
        }
    }
    if negative {
        match &mut big {
            Some(b) => {
                if *b == BigInt::from(0) {
                    return !(position as i64 - 1);
                }
                *b = -b.clone();
            }
            None => {
                if total == 0 {
                    return !(position as i64 - 1);
                }
                total = -total;
            }
        }
    } else if n.sign == SignStyle::ExceedsPad {
        let parse_len = pos - position;
        if positive {
            if parse_len <= n.min {
                return !(position as i64 - 1);
            }
        } else if parse_len > n.min {
            return !(position as i64);
        }
    }
    let value = match big {
        Some(mut b) => {
            // `BigInteger.bitLength() > 63`: drop the last digit.
            let magnitude_bits = if b < BigInt::from(0) { (-b.clone() - BigInt::from(1)).bits() } else { b.bits() };
            if magnitude_bits > 63 {
                b /= 10;
                pos -= 1;
            }
            long_value(&b)
        }
        None => total,
    };
    if n.reduced {
        let mut value = value;
        if pos - position == n.min && value >= 0 {
            value += 2000;
            if value < 2000 {
                value += 100;
            }
        }
        return ctx.set_parsed_field(n.field, value, position, pos);
    }
    ctx.set_parsed_field(n.field, value, position, pos)
}

/// `BigInteger.longValue()`: the low 64 bits.
fn long_value(b: &BigInt) -> i64 {
    let bytes = b.to_signed_bytes_le();
    let fill = if b < &BigInt::from(0) { 0xFF } else { 0 };
    let mut low = [fill; 8];
    for (i, byte) in bytes.iter().take(8).enumerate() {
        low[i] = *byte;
    }
    i64::from_le_bytes(low)
}

/// `NanosPrinterParser.parse` (strict).
fn parse_fraction(min: usize, max: usize, decimal_point: bool, ctx: &mut Context, text: &[u16], start: usize) -> Pos {
    let len = text.len();
    let mut position = start;
    if position == len {
        return if min > 0 { !(position as i64) } else { position as i64 };
    }
    if decimal_point {
        if text[position] != b'.' as u16 {
            return if min > 0 { !(position as i64) } else { position as i64 };
        }
        position += 1;
    }
    let min_end = position + min;
    if min_end > len {
        return !(position as i64);
    }
    let max_end = (position + max).min(len);
    let mut total: i64 = 0;
    let mut pos = position;
    while pos < max_end {
        let Some(d) = digit(text[pos]) else {
            if pos < min_end {
                return !(position as i64);
            }
            break;
        };
        pos += 1;
        total = total * 10 + d;
    }
    for _ in 0..9 - (pos - position) {
        total *= 10;
    }
    ctx.set_parsed_field(Field::NanoOfSecond, total, position, pos)
}

/// The texts a strict `TextPrinterParser` tries, longest first, with their values (`en_US`; a narrow text shared by
/// several values stands for the last of them, as `DateTimeTextProvider.LocaleStore` builds it).
fn text_entries(field: Field, style: TextStyle) -> Vec<(Vec<u16>, i64)> {
    let mut entries: Vec<(String, i64)> = match field {
        Field::MonthOfYear => match style {
            TextStyle::Narrow => [("J", 7), ("F", 2), ("M", 5), ("A", 8), ("S", 9), ("O", 10), ("N", 11), ("D", 12)]
                .iter()
                .map(|(t, v)| (t.to_string(), *v))
                .collect(),
            _ => MONTHS.iter().enumerate().map(|(i, m)| (if style == TextStyle::Full { m.to_string() } else { m[..3].to_string() }, i as i64 + 1)).collect(),
        },
        Field::DayOfWeek => match style {
            TextStyle::Narrow => [("M", 1), ("T", 4), ("W", 3), ("F", 5), ("S", 7)].iter().map(|(t, v)| (t.to_string(), *v)).collect(),
            _ => WEEKDAYS.iter().enumerate().map(|(i, d)| (if style == TextStyle::Full { d.to_string() } else { d[..3].to_string() }, i as i64 + 1)).collect(),
        },
        Field::AmPmOfDay => match style {
            TextStyle::Narrow => vec![("a".into(), 0), ("p".into(), 1)],
            _ => vec![("AM".into(), 0), ("PM".into(), 1)],
        },
        Field::Era => match style {
            TextStyle::Full => vec![("Before Christ".into(), 0), ("Anno Domini".into(), 1)],
            TextStyle::Short => vec![("BC".into(), 0), ("AD".into(), 1)],
            TextStyle::Narrow => vec![("B".into(), 0), ("A".into(), 1)],
        },
        Field::QuarterOfYear => (1..=4)
            .map(|q| {
                let t = match style {
                    TextStyle::Full => format!("{q}{} quarter", ["st", "nd", "rd", "th"][q as usize - 1]),
                    TextStyle::Short => format!("Q{q}"),
                    TextStyle::Narrow => q.to_string(),
                };
                (t, q)
            })
            .collect(),
        _ => Vec::new(),
    };
    entries.sort_by_key(|(t, _)| std::cmp::Reverse(t.encode_utf16().count()));
    entries.into_iter().map(|(t, v)| (units(&t), v)).collect()
}

/// The texts of `DayPeriodPrinterParser` (`en`), longest first, with their day period indexes.
fn day_period_entries(style: TextStyle) -> Vec<(Vec<u16>, i64)> {
    let narrow = style == TextStyle::Narrow;
    let mut entries: Vec<(&str, i64)> = vec![
        (if narrow { "a" } else { "AM" }, 0),
        (if narrow { "p" } else { "PM" }, 1),
        (if narrow { "mi" } else { "midnight" }, 2),
        (if narrow { "n" } else { "noon" }, 3),
        ("in the morning", 4),
        ("in the afternoon", 6),
        ("in the evening", 8),
        ("at night", 10),
    ];
    entries.sort_by_key(|(t, _)| std::cmp::Reverse(t.len()));
    entries.into_iter().map(|(t, v)| (units(t), v)).collect()
}

// ---------------------------------------------------------------------------------------------------------------
// Offsets and zones

/// `OffsetIdPrinterParser.parse` (strict).
/// The offset pattern [`parse_instant`] reads, past `OffsetIdPrinterParser.PATTERNS`: as the 8.0.0 image's
/// `Instant.parse` accepts it, `Z`, `+HH`, `+HH:mm`, `+HHmm`, `+HH:mm:ss` and `+HHmmss`.
const INSTANT_OFFSET_ID: usize = 22;

fn parse_offset(pattern: usize, no_offset: &str, ctx: &mut Context, text: &[u16], position: usize) -> Result<Pos> {
    let len = text.len();
    let no_offset = units(no_offset);
    if no_offset.is_empty() {
        if position == len {
            return Ok(ctx.set_parsed_field(Field::OffsetSeconds, 0, position, position));
        }
    } else {
        if position == len {
            return Ok(!(position as i64));
        }
        if ctx.sub_sequence_equals(text, position, &no_offset, 0, no_offset.len()) {
            return Ok(ctx.set_parsed_field(Field::OffsetSeconds, 0, position, position + no_offset.len()));
        }
    }
    let sign = text[position];
    if sign == b'+' as u16 || sign == b'-' as u16 {
        let negative: i64 = if sign == b'-' as u16 { -1 } else { 1 };
        // `Instant.parse`'s offset id: `+HH`, then optional minutes and seconds, all with a colon (`+HH:mm:ss`) or
        // none (`+HHmmss`), as the colon after the hours says.
        let pattern = match pattern {
            INSTANT_OFFSET_ID if text.get(position + 3) == Some(&(b':' as u16)) => 10,
            INSTANT_OFFSET_ID => 9,
            other => other,
        };
        let style = pattern % 11;
        let colon = style > 0 && style % 2 == 0;
        let padded_hour = pattern < 11;
        let mut array = [position as i64 + 1, 0, 0, 0];
        match pattern {
            0 | 11 => parse_hour(text, padded_hour, &mut array),
            1 | 2 | 13 => {
                parse_hour(text, padded_hour, &mut array);
                parse_minute_or_second(text, colon, false, 2, &mut array);
            }
            3 | 4 | 15 => {
                parse_hour(text, padded_hour, &mut array);
                parse_minute_or_second(text, colon, true, 2, &mut array);
            }
            5 | 6 | 17 => {
                parse_hour(text, padded_hour, &mut array);
                parse_minute_or_second(text, colon, true, 2, &mut array);
                parse_minute_or_second(text, colon, false, 3, &mut array);
            }
            7 | 8 | 19 => {
                parse_hour(text, padded_hour, &mut array);
                parse_minute_or_second(text, colon, true, 2, &mut array);
                parse_minute_or_second(text, colon, true, 3, &mut array);
            }
            9 | 10 | 21 => {
                parse_hour(text, padded_hour, &mut array);
                if parse_digits(text, colon, 2, &mut array) {
                    parse_digits(text, colon, 3, &mut array);
                }
            }
            12 => parse_variable_width_digits(text, 1, 4, &mut array),
            14 => parse_variable_width_digits(text, 3, 4, &mut array),
            16 => parse_variable_width_digits(text, 3, 6, &mut array),
            18 => parse_variable_width_digits(text, 5, 6, &mut array),
            _ => parse_variable_width_digits(text, 1, 6, &mut array),
        }
        if array[0] > 0 {
            if array[1] > 23 || array[2] > 59 || array[3] > 59 {
                return Err(date_time("Value out of range: Hour[0-23], Minute[0-59], Second[0-59]"));
            }
            let seconds = negative * (array[1] * 3600 + array[2] * 60 + array[3]);
            return Ok(ctx.set_parsed_field(Field::OffsetSeconds, seconds, position, array[0] as usize));
        }
    }
    if no_offset.is_empty() {
        return Ok(ctx.set_parsed_field(Field::OffsetSeconds, 0, position, position));
    }
    Ok(!(position as i64))
}

fn parse_hour(text: &[u16], padded_hour: bool, array: &mut [i64; 4]) {
    if padded_hour {
        if !parse_digits(text, false, 1, array) {
            array[0] = !array[0];
        }
    } else {
        parse_variable_width_digits(text, 1, 2, array);
    }
}

fn parse_minute_or_second(text: &[u16], colon: bool, mandatory: bool, index: usize, array: &mut [i64; 4]) {
    if !parse_digits(text, colon, index, array) && mandatory {
        array[0] = !array[0];
    }
}

fn parse_digits(text: &[u16], colon: bool, index: usize, array: &mut [i64; 4]) -> bool {
    let mut pos = array[0];
    if pos < 0 {
        return true;
    }
    if colon && index != 1 {
        if pos + 1 > text.len() as i64 || text[pos as usize] != b':' as u16 {
            return false;
        }
        pos += 1;
    }
    if pos + 2 > text.len() as i64 {
        return false;
    }
    let (Some(a), Some(b)) = (digit(text[pos as usize]), digit(text[pos as usize + 1])) else { return false };
    let value = a * 10 + b;
    if value > 59 {
        return false;
    }
    array[index] = value;
    array[0] = pos + 2;
    true
}

fn parse_variable_width_digits(text: &[u16], min_digits: usize, max_digits: usize, array: &mut [i64; 4]) {
    let mut pos = array[0] as usize;
    let mut digits = Vec::new();
    for _ in 0..max_digits {
        if pos + 1 > text.len() {
            break;
        }
        let Some(d) = digit(text[pos]) else { break };
        pos += 1;
        digits.push(d);
    }
    if digits.len() < min_digits {
        array[0] = !array[0];
        return;
    }
    let two = |i: usize| digits[i] * 10 + digits[i + 1];
    match digits.len() {
        1 => array[1] = digits[0],
        2 => array[1] = two(0),
        3 => {
            array[1] = digits[0];
            array[2] = two(1);
        }
        4 => {
            array[1] = two(0);
            array[2] = two(2);
        }
        5 => {
            array[1] = digits[0];
            array[2] = two(1);
            array[3] = two(3);
        }
        _ => {
            array[1] = two(0);
            array[2] = two(2);
            array[3] = two(4);
        }
    }
    array[0] = pos as i64;
}

/// `LocalizedOffsetIdPrinterParser.parse`.
fn parse_localized_offset(style: TextStyle, ctx: &mut Context, text: &[u16], position: usize) -> Result<Pos> {
    let end = text.len();
    let gmt = units("GMT");
    if !ctx.sub_sequence_equals(text, position, &gmt, 0, 3) {
        return Ok(!(position as i64));
    }
    let mut pos = position + 3;
    if pos == end {
        return Ok(ctx.set_parsed_field(Field::OffsetSeconds, 0, position, pos));
    }
    let negative: i64 = match text[pos] {
        0x2B => 1,
        0x2D => -1,
        _ => return Ok(ctx.set_parsed_field(Field::OffsetSeconds, 0, position, pos)),
    };
    pos += 1;
    let get_digit = |pos: usize| -> Result<Option<i64>> { Ok(digit(char_at(text, pos)?)) };
    let h;
    let (mut m, mut s) = (0, 0);
    if style == TextStyle::Full {
        let h1 = get_digit(pos)?;
        let h2 = get_digit(pos + 1)?;
        let c = char_at(text, pos + 2);
        pos += 3;
        let (Some(h1), Some(h2)) = (h1, h2) else { return Ok(!(position as i64)) };
        if c? != b':' as u16 {
            return Ok(!(position as i64));
        }
        h = h1 * 10 + h2;
        let m1 = get_digit(pos)?;
        let m2 = get_digit(pos + 1)?;
        pos += 2;
        let (Some(m1), Some(m2)) = (m1, m2) else { return Ok(!(position as i64)) };
        m = m1 * 10 + m2;
        if pos + 2 < end && text[pos] == b':' as u16
            && let (Some(s1), Some(s2)) = (digit(text[pos + 1]), digit(text[pos + 2]))
        {
            s = s1 * 10 + s2;
            pos += 3;
        }
    } else {
        let Some(h1) = get_digit(pos)? else { return Ok(!(position as i64)) };
        pos += 1;
        let mut hours = h1;
        if pos < end {
            if let Some(h2) = digit(text[pos]) {
                hours = hours * 10 + h2;
                pos += 1;
            }
            if pos + 2 < end && text[pos] == b':' as u16
                && let (Some(m1), Some(m2)) = (digit(text[pos + 1]), digit(text[pos + 2]))
            {
                m = m1 * 10 + m2;
                pos += 3;
                if pos + 2 < end && text[pos] == b':' as u16
                    && let (Some(s1), Some(s2)) = (digit(text[pos + 1]), digit(text[pos + 2]))
                {
                    s = s1 * 10 + s2;
                    pos += 3;
                }
            }
        }
        h = hours;
    }
    Ok(ctx.set_parsed_field(Field::OffsetSeconds, negative * (h * 3600 + m * 60 + s), position, pos))
}

/// `ZoneIdPrinterParser.parse` (and `ZoneTextPrinterParser`'s): the fixed forms (`+01:00`, `UTC`, `GMT+1`, `UT`,
/// `GMT0`, `Z`), then the region ids and zone names the JDK knows.
fn parse_zone(ctx: &mut Context, text: &[u16], position: usize, names: Option<TextStyle>) -> Result<Pos> {
    let len = text.len();
    if position == len {
        return Ok(!(position as i64));
    }
    let next = text[position];
    if next == b'+' as u16 || next == b'-' as u16 {
        return parse_offset_based(ctx, text, position, position, true);
    }
    if len >= position + 2 {
        let next_next = text[position + 1];
        if ctx.char_equals(next, b'U' as u16) && ctx.char_equals(next_next, b'T' as u16) {
            if len >= position + 3 && ctx.char_equals(text[position + 2], b'C' as u16) {
                if len > position + 3 && ctx.char_equals(text[position + 3], 0x2212) {
                    let mut tmp = text[..position + 3].to_vec();
                    tmp.push(b'-' as u16);
                    tmp.extend_from_slice(&text[position + 4..]);
                    return parse_offset_based(ctx, &tmp, position, position + 3, false);
                }
                return parse_offset_based(ctx, text, position, position + 3, false);
            }
            return parse_offset_based(ctx, text, position, position + 2, false);
        }
        if ctx.char_equals(next, b'G' as u16)
            && len >= position + 3
            && ctx.char_equals(next_next, b'M' as u16)
            && ctx.char_equals(text[position + 2], b'T' as u16)
        {
            if len >= position + 4 && ctx.char_equals(text[position + 3], b'0' as u16) {
                ctx.current().zone = Some(ZoneId::fixed_region("GMT0"));
                return Ok(position as i64 + 4);
            }
            return parse_offset_based(ctx, text, position, position + 3, false);
        }
    }
    let tree = match names {
        Some(TextStyle::Full) => &*super::zone::ZONE_TEXT_TREE_FULL,
        Some(TextStyle::Short) => &*super::zone::ZONE_TEXT_TREE_SHORT,
        _ => &*super::zone::REGION_TREE,
    };
    if let Some((id, kind, end)) = tree.matches(text, position) {
        let zone = ZoneId::of(id)?;
        let parsed = ctx.current();
        parsed.zone = Some(zone);
        parsed.zone_name_type = kind;
        return Ok(end as i64);
    }
    if ctx.char_equals(next, b'Z' as u16) {
        ctx.current().zone = Some(ZoneId::of_offset_seconds(0)?);
        return Ok(position as i64 + 1);
    }
    Ok(!(position as i64))
}

/// `ZoneIdPrinterParser.parseOffsetBased`: a prefix (`UTC`, `GMT`, `UT`, or none before a sign) and an offset parsed
/// with `+HH:MM:ss` (no-offset text `Z`, or `0` after a prefix).
fn parse_offset_based(ctx: &mut Context, text: &[u16], prefix_pos: usize, position: usize, sign_only: bool) -> Result<Pos> {
    let prefix = String::from_utf16_lossy(&text[prefix_pos..position]).to_ascii_uppercase();
    if position >= text.len() || text[position] == b'0' as u16 || ctx.char_equals(text[position], b'Z' as u16) {
        ctx.current().zone = Some(ZoneId::prefixed(&prefix, 0));
        return Ok(position as i64);
    }
    let mut copy = Context { case_sensitive: ctx.case_sensitive, parsed: vec![Parsed::new()] };
    let end = parse_offset(6, if sign_only { "Z" } else { "0" }, &mut copy, text, position)?;
    if end < 0 {
        if sign_only {
            return Ok(!(prefix_pos as i64));
        }
        ctx.current().zone = Some(ZoneId::prefixed(&prefix, 0));
        return Ok(position as i64);
    }
    let offset = copy.current().get(Field::OffsetSeconds).unwrap_or(0) as i32;
    match ZoneId::of_offset_seconds(offset) {
        Ok(zone) => {
            ctx.current().zone = Some(if prefix.is_empty() { zone } else { ZoneId::prefixed(&prefix, offset) });
            Ok(end)
        }
        Err(_) => Ok(!(prefix_pos as i64)),
    }
}

// ---------------------------------------------------------------------------------------------------------------
// InstantPrinterParser

/// `InstantPrinterParser.parse` with `fractionalDigits` -2 (`ISO_INSTANT`): `ISO_LOCAL_DATE`, `T`, `HH:mm:ss`, an
/// optional fraction of up to nine digits and an offset id, in a context of its own.
fn parse_instant(ctx: &mut Context, text: &[u16], position: usize) -> Result<Pos> {
    const SECONDS_PER_10000_YEARS: i64 = 146_097 * 25 * 86_400;
    let n = |field, min, max, sign| Item::Number(Number { field, min, max, sign, subsequent: 0, reduced: false, outer: None });
    let lit = |c: u8| Item::Literal(vec![c as u16]);
    let items = [
        n(Field::Year, 4, 10, SignStyle::ExceedsPad),
        lit(b'-'),
        n(Field::MonthOfYear, 2, 2, SignStyle::NotNegative),
        lit(b'-'),
        n(Field::DayOfMonth, 2, 2, SignStyle::NotNegative),
        lit(b'T'),
        n(Field::HourOfDay, 2, 2, SignStyle::NotNegative),
        lit(b':'),
        n(Field::MinuteOfHour, 2, 2, SignStyle::NotNegative),
        lit(b':'),
        n(Field::SecondOfMinute, 2, 2, SignStyle::NotNegative),
        Item::Fraction { min: 0, max: 9, decimal_point: true, subsequent: 0 },
        Item::Offset(INSTANT_OFFSET_ID, "Z"),
    ];
    let mut inner = Context { case_sensitive: ctx.case_sensitive, parsed: vec![Parsed::new()] };
    let pos = parse_items(&items, &mut inner, text, position)?;
    if pos < 0 {
        return Ok(pos);
    }
    let p = inner.current();
    let year_parsed = p.get(Field::Year).unwrap_or(0);
    let month = p.get(Field::MonthOfYear).unwrap_or(0);
    let day = p.get(Field::DayOfMonth).unwrap_or(0);
    let mut hour = p.get(Field::HourOfDay).unwrap_or(0);
    let min = p.get(Field::MinuteOfHour).unwrap_or(0);
    let mut sec = p.get(Field::SecondOfMinute).unwrap_or(0);
    let nano = p.get(Field::NanoOfSecond).unwrap_or(0);
    let offset = p.get(Field::OffsetSeconds).unwrap_or(0);
    let mut days = 0;
    if hour == 24 && min == 0 && sec == 0 && nano == 0 {
        hour = 0;
        days = 1;
    } else if hour == 23 && min == 59 && sec == 60 {
        ctx.current().leap_second = true;
        sec = 59;
    }
    let year = (year_parsed as i32) % 10_000;
    let instant_secs = (|| -> Result<i64> {
        let date = LocalDate::of(year, month as u32, day as u32)?;
        let time = LocalTime::of(hour as u32, min as u32, sec as u32, 0)?;
        let date = date.plus_days(days)?;
        ZoneId::of_offset_seconds(offset as i32)?;
        let secs = LocalDateTime::of(date, time).to_epoch_second() - offset;
        let cycles = (year_parsed / 10_000).checked_mul(SECONDS_PER_10000_YEARS);
        cycles.map(|c| secs.wrapping_add(c)).ok_or_else(|| Error::new("ArithmeticException", "long overflow"))
    })();
    let Ok(instant_secs) = instant_secs else { return Ok(!(position as i64)) };
    let success = ctx.set_parsed_field(Field::InstantSeconds, instant_secs, position, pos as usize);
    if success < 0 {
        return Ok(success);
    }
    Ok(ctx.set_parsed_field(Field::NanoOfSecond, nano, position, success as usize))
}

// ---------------------------------------------------------------------------------------------------------------
// Parsed.resolve

fn checked_int(field: Field, value: i64) -> Result<i32> {
    field.range().check_int(value, field)
}

fn check(field: Field, value: i64) -> Result<i64> {
    field.range().check(value, field)
}

/// `Math.toIntExact(value)`.
fn to_int_exact(value: i64) -> Result<i32> {
    i32::try_from(value).map_err(|_| Error::new("ArithmeticException", "integer overflow"))
}

/// `LocalDate.ofYearDay(year, dayOfYear)`.
fn of_year_day(year: i32, day_of_year: i64) -> Result<LocalDate> {
    check(Field::Year, year as i64)?;
    check(Field::DayOfYear, day_of_year)?;
    let leap = LocalDate::of(year, 1, 1)?.is_leap_year();
    if day_of_year == 366 && !leap {
        return Err(date_time(format!("Invalid date 'DayOfYear 366' as '{year}' is not a leap year")));
    }
    LocalDate::of(year, 1, 1)?.plus_days(day_of_year - 1)
}

/// `with(TemporalAdjusters.nextOrSame(dayOfWeek))`.
fn next_or_same(date: LocalDate, dow: i64) -> Result<LocalDate> {
    let cal = date.day_of_week() as i64;
    if cal == dow {
        return Ok(date);
    }
    let diff = cal - dow;
    date.plus_days(if diff >= 0 { 7 - diff } else { -diff })
}

impl Parsed {
    fn resolve(&mut self, strict: bool) -> Result<()> {
        self.strict = strict;
        self.resolve_fields()?;
        self.resolve_time_lenient()?;
        self.cross_check()?;
        self.resolve_period()?;
        self.resolve_fractional();
        self.resolve_instant()
    }

    fn resolve_fields(&mut self) -> Result<()> {
        self.resolve_chrono_fields()?;
        if self.fields.is_empty() {
            return Ok(());
        }
        let mut changed = 0;
        'outer: while changed < 50 {
            for i in 0..self.fields.len() {
                let target = self.fields[i].0;
                match self.resolve_field(target)? {
                    Some(date) => {
                        self.update_check_conflict_date(date)?;
                        changed += 1;
                        continue 'outer;
                    }
                    None if !self.contains(target) => {
                        changed += 1;
                        continue 'outer;
                    }
                    None => {}
                }
            }
            break;
        }
        if changed == 50 {
            return Err(date_time("One of the parsed fields has an incorrectly implemented resolve method"));
        }
        if changed > 0 {
            self.resolve_chrono_fields()?;
        }
        Ok(())
    }

    fn resolve_chrono_fields(&mut self) -> Result<()> {
        self.resolve_instant_fields()?;
        let date = self.resolve_date()?;
        if let Some(date) = date {
            self.update_check_conflict_date(date)?;
        }
        self.resolve_time_fields()
    }

    /// `TemporalField.resolve` of the fields that are not `ChronoField`s.
    fn resolve_field(&mut self, field: Field) -> Result<Option<LocalDate>> {
        let Some(value) = self.get(field) else { return Ok(None) };
        match field {
            Field::ModifiedJulianDay => {
                self.remove(field);
                check(field, value)?;
                LocalDate::of_epoch_day(value - 40_587).map(Some)
            }
            Field::LocalizedDayOfWeek => {
                to_int_exact(value)?;
                let checked = checked_int(field, value)? as i64;
                let iso = (FIRST_DOW - 1 + (checked - 1)).rem_euclid(7) + 1;
                self.remove(field);
                self.put(Field::DayOfWeek, iso);
                Ok(None)
            }
            Field::WeekOfMonth | Field::WeekOfWeekBasedYear | Field::WeekBasedYear => {
                let new_value = to_int_exact(value)? as i64;
                let Some(iso_dow) = self.get(Field::DayOfWeek) else { return Ok(None) };
                let iso_dow = checked_int(Field::DayOfWeek, iso_dow)? as i64;
                let dow = localized_dow_of(iso_dow);
                if let Some(year) = self.get(Field::Year) {
                    let year = checked_int(Field::Year, year)?;
                    if field == Field::WeekOfMonth && let Some(month) = self.get(Field::MonthOfYear) {
                        return self.resolve_week_of_month(year, month, new_value, dow).map(Some);
                    }
                } else if field != Field::WeekOfMonth && self.contains(Field::WeekBasedYear) && self.contains(Field::WeekOfWeekBasedYear) {
                    return self.resolve_week_based_year(dow).map(Some);
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn resolve_week_of_month(&mut self, year: i32, month: i64, wom: i64, local_dow: i64) -> Result<LocalDate> {
        let month_valid = checked_int(Field::MonthOfYear, month)?;
        let date = LocalDate::of(year, month_valid as u32, 1)?;
        let wom = checked_int(Field::WeekOfMonth, wom)? as i64;
        let weeks = wom - week_of_month(&date);
        let days = local_dow - localized_dow_of(date.day_of_week() as i64);
        let date = date.plus_days(weeks * 7 + days)?;
        if self.strict && date.month_value() as i64 != month {
            return Err(date_time("Strict mode rejected resolved date as it is in a different month"));
        }
        self.remove(Field::WeekOfMonth);
        self.remove(Field::Year);
        self.remove(Field::MonthOfYear);
        self.remove(Field::DayOfWeek);
        Ok(date)
    }

    fn resolve_week_based_year(&mut self, local_dow: i64) -> Result<LocalDate> {
        let yowby = checked_int(Field::WeekBasedYear, self.get(Field::WeekBasedYear).unwrap_or(0))?;
        let range = range_week_of_week_based_year(LocalDate::of(yowby, 7, 2)?)?;
        let wowby = range.check_int(self.get(Field::WeekOfWeekBasedYear).unwrap_or(0), Field::WeekOfWeekBasedYear)? as i64;
        let date = of_week_based_year(yowby, wowby, local_dow)?;
        if self.strict && date_field(&date, Field::WeekBasedYear) != Some(yowby as i64) {
            return Err(date_time("Strict mode rejected resolved date as it is in a different week-based-year"));
        }
        self.remove(Field::WeekBasedYear);
        self.remove(Field::WeekOfWeekBasedYear);
        self.remove(Field::DayOfWeek);
        Ok(date)
    }

    fn update_check_conflict_date(&mut self, date: LocalDate) -> Result<()> {
        match self.date {
            Some(d) if d != date => Err(date_time(format!("Conflict found: Fields resolved to two different dates: {d} {date}"))),
            Some(_) => Ok(()),
            None => {
                self.date = Some(date);
                Ok(())
            }
        }
    }

    /// `updateCheckConflict(targetField, changeField, changeValue)`.
    fn update_check_conflict_field(&mut self, target: Field, change: Field, value: i64) -> Result<()> {
        match self.put(change, value) {
            Some(old) if old != value => Err(date_time(format!(
                "Conflict found: {} {old} differs from {} {value} while resolving  {}",
                change.name(),
                change.name(),
                target.name()
            ))),
            _ => Ok(()),
        }
    }

    fn update_check_conflict_time(&mut self, time: LocalTime, excess_days: i64) -> Result<()> {
        match self.time {
            Some(t) => {
                if t != time {
                    return Err(date_time(format!("Conflict found: Fields resolved to different times: {t} {time}")));
                }
                if self.excess_days != 0 && excess_days != 0 && self.excess_days != excess_days {
                    return Err(date_time(format!(
                        "Conflict found: Fields resolved to different excess periods: P{}D P{excess_days}D",
                        self.excess_days
                    )));
                }
                self.excess_days = excess_days;
            }
            None => {
                self.time = Some(time);
                self.excess_days = excess_days;
            }
        }
        Ok(())
    }

    fn resolve_instant_fields(&mut self) -> Result<()> {
        let Some(seconds) = self.get(Field::InstantSeconds) else { return Ok(()) };
        let offset = match &self.zone {
            Some(zone) => {
                let instant = Instant::of_epoch_second(seconds, 0)?;
                zone.offset_at(instant.epoch_second())
            }
            None => match self.get(Field::OffsetSeconds) {
                Some(o) => {
                    ZoneId::of_offset_seconds(o as i32)?;
                    Instant::of_epoch_second(seconds, 0)?;
                    o as i32
                }
                None => return Ok(()),
            },
        };
        let local = seconds + offset as i64;
        let date = LocalDate::of_epoch_day(local.div_euclid(86_400))?;
        self.update_check_conflict_date(date)?;
        self.update_check_conflict_field(Field::InstantSeconds, Field::SecondOfDay, local.rem_euclid(86_400))?;
        self.update_check_conflict_field(Field::InstantSeconds, Field::OffsetSeconds, offset as i64)
    }

    /// `AbstractChronology.addFieldValue`.
    fn add_field_value(&mut self, field: Field, value: i64) -> Result<()> {
        if let Some(old) = self.get(field) && old != value {
            return Err(date_time(format!("Conflict found: {} {old} differs from {} {value}", field.name(), field.name())));
        }
        self.put(field, value);
        Ok(())
    }

    /// `IsoChronology.resolveDate(fieldValues, resolverStyle)`.
    fn resolve_date(&mut self) -> Result<Option<LocalDate>> {
        if let Some(epoch_day) = self.remove(Field::EpochDay) {
            return LocalDate::of_epoch_day(epoch_day).map(Some);
        }
        if let Some(p) = self.remove(Field::ProlepticMonth) {
            check(Field::ProlepticMonth, p)?;
            self.add_field_value(Field::MonthOfYear, p.rem_euclid(12) + 1)?;
            self.add_field_value(Field::Year, p.div_euclid(12))?;
        }
        self.resolve_year_of_era()?;
        if self.contains(Field::Year) {
            if self.contains(Field::MonthOfYear) {
                if self.contains(Field::DayOfMonth) {
                    return self.resolve_ymd().map(Some);
                }
                if self.contains(Field::AlignedWeekOfMonth) {
                    if self.contains(Field::AlignedDayOfWeekInMonth) {
                        return self.resolve_ym_aligned(false).map(Some);
                    }
                    if self.contains(Field::DayOfWeek) {
                        return self.resolve_ym_aligned(true).map(Some);
                    }
                }
            }
            if self.contains(Field::DayOfYear) {
                let y = checked_int(Field::Year, self.remove(Field::Year).unwrap_or(0))?;
                let doy = checked_int(Field::DayOfYear, self.remove(Field::DayOfYear).unwrap_or(0))?;
                return of_year_day(y, doy as i64).map(Some);
            }
            if self.contains(Field::AlignedWeekOfYear) {
                if self.contains(Field::AlignedDayOfWeekInYear) {
                    return self.resolve_y_aligned(false).map(Some);
                }
                if self.contains(Field::DayOfWeek) {
                    return self.resolve_y_aligned(true).map(Some);
                }
            }
        }
        Ok(None)
    }

    /// `IsoChronology.resolveYearOfEra`.
    fn resolve_year_of_era(&mut self) -> Result<()> {
        if let Some(yoe) = self.remove(Field::YearOfEra) {
            check(Field::YearOfEra, yoe)?;
            match self.remove(Field::Era) {
                None => {
                    let year = self.get(Field::Year);
                    if self.strict {
                        match year {
                            Some(y) => self.add_field_value(Field::Year, if y > 0 { yoe } else { 1 - yoe })?,
                            None => {
                                self.put(Field::YearOfEra, yoe);
                            }
                        }
                    } else {
                        let y = match year {
                            None => yoe,
                            Some(y) if y > 0 => yoe,
                            Some(_) => 1 - yoe,
                        };
                        self.add_field_value(Field::Year, y)?;
                    }
                }
                Some(1) => self.add_field_value(Field::Year, yoe)?,
                Some(0) => self.add_field_value(Field::Year, 1 - yoe)?,
                Some(era) => return Err(date_time(format!("Invalid value for era: {era}"))),
            }
        } else if let Some(era) = self.get(Field::Era) {
            check(Field::Era, era)?;
        }
        Ok(())
    }

    /// `IsoChronology.resolveYMD`.
    fn resolve_ymd(&mut self) -> Result<LocalDate> {
        let y = checked_int(Field::Year, self.remove(Field::Year).unwrap_or(0))?;
        let moy = checked_int(Field::MonthOfYear, self.remove(Field::MonthOfYear).unwrap_or(0))?;
        let mut dom = checked_int(Field::DayOfMonth, self.remove(Field::DayOfMonth).unwrap_or(0))?;
        if !self.strict {
            if matches!(moy, 4 | 6 | 9 | 11) {
                dom = dom.min(30);
            } else if moy == 2 {
                dom = dom.min(if LocalDate::of(y, 1, 1)?.is_leap_year() { 29 } else { 28 });
            }
        }
        LocalDate::of(y, moy as u32, dom as u32)
    }

    /// `resolveYMAA` and `resolveYMAD`.
    fn resolve_ym_aligned(&mut self, day_of_week: bool) -> Result<LocalDate> {
        let y = checked_int(Field::Year, self.remove(Field::Year).unwrap_or(0))?;
        let moy = checked_int(Field::MonthOfYear, self.remove(Field::MonthOfYear).unwrap_or(0))?;
        let aw = checked_int(Field::AlignedWeekOfMonth, self.remove(Field::AlignedWeekOfMonth).unwrap_or(0))? as i64;
        let date = if day_of_week {
            let dow = checked_int(Field::DayOfWeek, self.remove(Field::DayOfWeek).unwrap_or(0))? as i64;
            next_or_same(LocalDate::of(y, moy as u32, 1)?.plus_days((aw - 1) * 7)?, dow)?
        } else {
            let ad = checked_int(Field::AlignedDayOfWeekInMonth, self.remove(Field::AlignedDayOfWeekInMonth).unwrap_or(0))? as i64;
            LocalDate::of(y, moy as u32, 1)?.plus_days((aw - 1) * 7 + (ad - 1))?
        };
        if self.strict && date.month_value() as i32 != moy {
            return Err(date_time("Strict mode rejected resolved date as it is in a different month"));
        }
        Ok(date)
    }

    /// `resolveYAA` and `resolveYAD`.
    fn resolve_y_aligned(&mut self, day_of_week: bool) -> Result<LocalDate> {
        let y = checked_int(Field::Year, self.remove(Field::Year).unwrap_or(0))?;
        let aw = checked_int(Field::AlignedWeekOfYear, self.remove(Field::AlignedWeekOfYear).unwrap_or(0))? as i64;
        let date = if day_of_week {
            let dow = checked_int(Field::DayOfWeek, self.remove(Field::DayOfWeek).unwrap_or(0))? as i64;
            next_or_same(of_year_day(y, 1)?.plus_days((aw - 1) * 7)?, dow)?
        } else {
            let ad = checked_int(Field::AlignedDayOfWeekInYear, self.remove(Field::AlignedDayOfWeekInYear).unwrap_or(0))? as i64;
            of_year_day(y, 1)?.plus_days((aw - 1) * 7 + (ad - 1))?
        };
        if self.strict && date.year() != y {
            return Err(date_time("Strict mode rejected resolved date as it is in a different year"));
        }
        Ok(date)
    }

    /// `Parsed.resolveTimeFields`.
    fn resolve_time_fields(&mut self) -> Result<()> {
        if let Some(ch) = self.remove(Field::ClockHourOfDay) {
            if self.strict || ch != 0 {
                check(Field::ClockHourOfDay, ch)?;
            }
            self.update_check_conflict_field(Field::ClockHourOfDay, Field::HourOfDay, if ch == 24 { 0 } else { ch })?;
        }
        if let Some(ch) = self.remove(Field::ClockHourOfAmPm) {
            if self.strict || ch != 0 {
                check(Field::ClockHourOfAmPm, ch)?;
            }
            self.update_check_conflict_field(Field::ClockHourOfAmPm, Field::HourOfAmPm, if ch == 12 { 0 } else { ch })?;
        }
        if self.contains(Field::AmPmOfDay) && self.contains(Field::HourOfAmPm) {
            let ap = self.remove(Field::AmPmOfDay).unwrap_or(0);
            let hap = self.remove(Field::HourOfAmPm).unwrap_or(0);
            check(Field::AmPmOfDay, ap)?;
            check(Field::HourOfAmPm, hap)?;
            self.update_check_conflict_field(Field::AmPmOfDay, Field::HourOfDay, ap * 12 + hap)?;
        }
        if let Some(nod) = self.remove(Field::NanoOfDay) {
            check(Field::NanoOfDay, nod)?;
            self.update_check_conflict_field(Field::NanoOfDay, Field::HourOfDay, nod / 3_600_000_000_000)?;
            self.update_check_conflict_field(Field::NanoOfDay, Field::MinuteOfHour, (nod / 60_000_000_000) % 60)?;
            self.update_check_conflict_field(Field::NanoOfDay, Field::SecondOfMinute, (nod / 1_000_000_000) % 60)?;
            self.update_check_conflict_field(Field::NanoOfDay, Field::NanoOfSecond, nod % 1_000_000_000)?;
        }
        if let Some(cod) = self.remove(Field::MicroOfDay) {
            check(Field::MicroOfDay, cod)?;
            self.update_check_conflict_field(Field::MicroOfDay, Field::SecondOfDay, cod / 1_000_000)?;
            self.update_check_conflict_field(Field::MicroOfDay, Field::MicroOfSecond, cod % 1_000_000)?;
        }
        if let Some(lod) = self.remove(Field::MilliOfDay) {
            check(Field::MilliOfDay, lod)?;
            self.update_check_conflict_field(Field::MilliOfDay, Field::SecondOfDay, lod / 1000)?;
            self.update_check_conflict_field(Field::MilliOfDay, Field::MilliOfSecond, lod % 1000)?;
        }
        if let Some(sod) = self.remove(Field::SecondOfDay) {
            check(Field::SecondOfDay, sod)?;
            self.update_check_conflict_field(Field::SecondOfDay, Field::HourOfDay, sod / 3600)?;
            self.update_check_conflict_field(Field::SecondOfDay, Field::MinuteOfHour, (sod / 60) % 60)?;
            self.update_check_conflict_field(Field::SecondOfDay, Field::SecondOfMinute, sod % 60)?;
        }
        if let Some(mod_) = self.remove(Field::MinuteOfDay) {
            check(Field::MinuteOfDay, mod_)?;
            self.update_check_conflict_field(Field::MinuteOfDay, Field::HourOfDay, mod_ / 60)?;
            self.update_check_conflict_field(Field::MinuteOfDay, Field::MinuteOfHour, mod_ % 60)?;
        }
        if let Some(mut nos) = self.get(Field::NanoOfSecond) {
            check(Field::NanoOfSecond, nos)?;
            if let Some(cos) = self.remove(Field::MicroOfSecond) {
                check(Field::MicroOfSecond, cos)?;
                nos = cos * 1000 + (nos % 1000);
                self.update_check_conflict_field(Field::MicroOfSecond, Field::NanoOfSecond, nos)?;
            }
            if let Some(los) = self.remove(Field::MilliOfSecond) {
                check(Field::MilliOfSecond, los)?;
                self.update_check_conflict_field(Field::MilliOfSecond, Field::NanoOfSecond, los * 1_000_000 + (nos % 1_000_000))?;
            }
        }
        if let Some(dp) = self.day_period && let Some(hoap) = self.remove(Field::HourOfAmPm) {
            check(Field::HourOfAmPm, hoap)?;
            let moh = self.get(Field::MinuteOfHour).map_or(0, |m| m.rem_euclid(60));
            let excess = if dp.includes((hoap.rem_euclid(12) + 12) * 60 + moh) { 12 } else { 0 };
            self.update_check_conflict_field(Field::HourOfAmPm, Field::HourOfDay, hoap + excess)?;
            self.day_period = None;
        }
        if self.contains(Field::HourOfDay) && self.contains(Field::MinuteOfHour) && self.contains(Field::SecondOfMinute) && self.contains(Field::NanoOfSecond) {
            let hod = self.remove(Field::HourOfDay).unwrap_or(0);
            let moh = self.remove(Field::MinuteOfHour).unwrap_or(0);
            let som = self.remove(Field::SecondOfMinute).unwrap_or(0);
            let nos = self.remove(Field::NanoOfSecond).unwrap_or(0);
            self.resolve_time(hod, moh, som, nos)?;
        }
        Ok(())
    }

    /// `Parsed.resolveTimeLenient`.
    fn resolve_time_lenient(&mut self) -> Result<()> {
        if self.time.is_none() {
            if let Some(los) = self.remove(Field::MilliOfSecond) {
                if let Some(cos) = self.get(Field::MicroOfSecond) {
                    let cos = los * 1000 + cos % 1000;
                    self.update_check_conflict_field(Field::MilliOfSecond, Field::MicroOfSecond, cos)?;
                    self.remove(Field::MicroOfSecond);
                    self.put(Field::NanoOfSecond, cos * 1000);
                } else {
                    self.put(Field::NanoOfSecond, los * 1_000_000);
                }
            } else if let Some(cos) = self.remove(Field::MicroOfSecond) {
                self.put(Field::NanoOfSecond, cos * 1000);
            }
            let no_time_field = ![Field::HourOfDay, Field::MinuteOfHour, Field::SecondOfMinute, Field::NanoOfSecond].iter().any(|f| self.contains(*f));
            if no_time_field && !self.strict {
                if let Some(dp) = self.day_period {
                    let mid = dp.mid();
                    self.resolve_time(mid / 60, mid % 60, 0, 0)?;
                    self.day_period = None;
                } else if let Some(ap) = self.remove(Field::AmPmOfDay) {
                    check(Field::AmPmOfDay, ap)?;
                    self.resolve_time(ap * 12 + 6, 0, 0, 0)?;
                }
            }
            if let Some(hod) = self.get(Field::HourOfDay) {
                let moh = self.get(Field::MinuteOfHour);
                let som = self.get(Field::SecondOfMinute);
                let nos = self.get(Field::NanoOfSecond);
                if moh.is_none() && (som.is_some() || nos.is_some()) || moh.is_some() && som.is_none() && nos.is_some() {
                    return Ok(());
                }
                let moh = moh.unwrap_or(0);
                if let Some(dp) = self.day_period && !dp.includes(hod * 60 + moh) {
                    return Err(date_time(format!("Conflict found: Resolved time {hod:02}:{moh:02} conflicts with {dp}")));
                }
                self.resolve_time(hod, moh, som.unwrap_or(0), nos.unwrap_or(0))?;
                for f in [Field::HourOfDay, Field::MinuteOfHour, Field::SecondOfMinute, Field::NanoOfSecond] {
                    self.remove(f);
                }
            }
        }
        for i in 0..self.fields.len() {
            let (field, value) = self.fields[i];
            if field.is_chrono() && field.is_time_based() {
                check(field, value)?;
            }
        }
        Ok(())
    }

    fn resolve_time(&mut self, hod: i64, moh: i64, som: i64, nos: i64) -> Result<()> {
        let moh = checked_int(Field::MinuteOfHour, moh)?;
        let nos = checked_int(Field::NanoOfSecond, nos)?;
        if !self.strict && hod == 24 && moh == 0 && som == 0 && nos == 0 {
            return self.update_check_conflict_time(LocalTime::MIDNIGHT, 1);
        }
        let hod = checked_int(Field::HourOfDay, hod)?;
        let som = checked_int(Field::SecondOfMinute, som)?;
        self.update_check_conflict_time(LocalTime::of(hod as u32, moh as u32, som as u32, nos as u32)?, 0)
    }

    fn resolve_period(&mut self) -> Result<()> {
        if let (Some(date), Some(_)) = (self.date, self.time) && self.excess_days != 0 {
            self.date = Some(date.plus_days(self.excess_days)?);
            self.excess_days = 0;
        }
        Ok(())
    }

    fn resolve_fractional(&mut self) {
        if self.time.is_none() && [Field::InstantSeconds, Field::SecondOfDay, Field::SecondOfMinute].iter().any(|f| self.contains(*f)) {
            if let Some(nos) = self.get(Field::NanoOfSecond) {
                self.put(Field::MicroOfSecond, nos / 1000);
                self.put(Field::MilliOfSecond, nos / 1_000_000);
            } else {
                self.put(Field::NanoOfSecond, 0);
                self.put(Field::MicroOfSecond, 0);
                self.put(Field::MilliOfSecond, 0);
            }
        }
    }

    fn resolve_instant(&mut self) -> Result<()> {
        let (Some(date), Some(time)) = (self.date, self.time) else { return Ok(()) };
        if self.contains(Field::InstantSeconds) {
            return Ok(());
        }
        let local = LocalDateTime::of(date, time).to_epoch_second();
        if let Some(offset) = self.get(Field::OffsetSeconds) {
            ZoneId::of_offset_seconds(offset as i32)?;
            self.put(Field::InstantSeconds, local - offset);
        } else if let Some(zone) = &self.zone {
            let later = self.zone_name_type == ZONE_NAME_STD || self.zone_name_type == ZONE_NAME_GENERIC;
            let offset = zone.offset_for_local(local, later);
            self.put(Field::InstantSeconds, local - offset as i64);
        }
        Ok(())
    }

    fn cross_check(&mut self) -> Result<()> {
        if let Some(date) = self.date {
            self.cross_check_with(Some(date), None)?;
        }
        if let Some(time) = self.time {
            self.cross_check_with(None, Some(time))?;
            if let Some(date) = self.date && !self.fields.is_empty() {
                self.cross_check_with(Some(date), Some(time))?;
            }
        }
        Ok(())
    }

    fn cross_check_with(&mut self, date: Option<LocalDate>, time: Option<LocalTime>) -> Result<()> {
        let target = match (date, time) {
            (Some(d), Some(t)) => format!("{d}T{t}"),
            (Some(d), None) => d.to_string(),
            (None, Some(t)) => t.to_string(),
            (None, None) => return Ok(()),
        };
        let mut i = 0;
        while i < self.fields.len() {
            let (field, val2) = self.fields[i];
            let from_date = date.filter(|_| date_supports(field)).and_then(|d| date_field(&d, field));
            let from_time = time.filter(|_| field.is_chrono() && field.is_time_based()).and_then(|t| time_field(&t, field));
            if let Some(val1) = from_date.or(from_time) {
                if val1 != val2 {
                    return Err(date_time(format!(
                        "Conflict found: Field {} {val1} differs from {} {val2} derived from {target}",
                        field.name(),
                        field.name()
                    )));
                }
                self.fields.remove(i);
            } else {
                i += 1;
            }
        }
        Ok(())
    }
}

/// `WeekFields.getFirstDayOfWeek().getValue()` of `en_US` (Sunday).
const FIRST_DOW: i64 = 7;

/// `ComputedDayOfField.ofWeekBasedYear(chrono, yowby, wowby, dow)`.
fn of_week_based_year(yowby: i32, wowby: i64, dow: i64) -> Result<LocalDate> {
    let date = LocalDate::of(yowby, 1, 1)?;
    let ldow = localized_dow_of(date.day_of_week() as i64);
    let offset = start_of_week_offset(1, ldow);
    let year_len = date.length_of_year() as i64;
    let new_year_week = compute_week(offset, year_len + MINIMAL_DAYS);
    let wowby = wowby.min(new_year_week - 1);
    let days = -offset + (dow - 1) + (wowby - 1) * 7;
    date.plus_days(days)
}

/// `ComputedDayOfField.rangeWeekOfWeekBasedYear(temporal)`.
fn range_week_of_week_based_year(date: LocalDate) -> Result<super::formatter::ValueRange> {
    let dow = localized_dow_of(date.day_of_week() as i64);
    let doy = date.day_of_year() as i64;
    let offset = start_of_week_offset(doy, dow);
    let week = compute_week(offset, doy);
    if week == 0 {
        return range_week_of_week_based_year(date.minus_days(doy + 7)?);
    }
    let year_len = date.length_of_year() as i64;
    let new_year_week = compute_week(offset, year_len + MINIMAL_DAYS);
    if week >= new_year_week {
        return range_week_of_week_based_year(date.plus_days(year_len - doy + 1 + 7)?);
    }
    Ok(super::formatter::ValueRange::of(1, new_year_week - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(pattern: &str, text: &str) -> String {
        match DateTimeFormatter::of_pattern(pattern).and_then(|f| f.parse(text)) {
            Ok(p) => p.to_string(),
            Err(e) => format!("ERR {}: {}", e.simple_class(), e.message()),
        }
    }

    #[test]
    fn patterns() {
        assert_eq!(parse("yyyyMMdd", "20231114"), "{},ISO resolved to 2023-11-14");
        assert_eq!(parse("Hmm", "930"), "{},ISO resolved to 09:30");
        assert_eq!(parse("yyyyMMddHHmmssSSS", "20231114221320123"), "{},ISO resolved to 2023-11-14T22:13:20.123");
        assert_eq!(
            parse("yyyy MM.dd mm:hh:ss", "2023 11.14 13:10:20"),
            "{HourOfAmPm=10, MicroOfSecond=0, MilliOfSecond=0, MinuteOfHour=13, NanoOfSecond=0, SecondOfMinute=20},ISO resolved to 2023-11-14"
        );
        assert_eq!(parse("uuuu-MM-dd", "2023-02-30"), "{},ISO resolved to 2023-02-28");
        assert_eq!(
            parse("yyyy-MM-dd", "-2023-02-28"),
            "ERR DateTimeParseException: Text '-2023-02-28' could not be parsed: Invalid value for YearOfEra (valid values 1 - 999999999/1000000000): -2023"
        );
        assert_eq!(parse("yyyy-MM-dd", "+12023-02-28"), "{},ISO resolved to +12023-02-28");
        assert_eq!(parse("yy", "12"), "{Year=2012},ISO");
        assert_eq!(
            parse("yyyy-DDD", "2023-366"),
            "ERR DateTimeParseException: Text '2023-366' could not be parsed: Invalid date 'DayOfYear 366' as '2023' is not a leap year"
        );
        assert_eq!(parse("YYYY-ww-e", "2023-46-3"), "{},ISO resolved to 2023-11-14");
        assert_eq!(parse("MMMMM", "J"), "{MonthOfYear=7},ISO");
        assert_eq!(parse("h B", "11 at night"), "{},ISO resolved to 23:00");
        assert_eq!(
            parse("HH B", "03 at night"),
            "ERR DateTimeParseException: Text '03 at night' could not be parsed: Conflict found: Resolved time 03:00 conflicts with DayPeriod(21:00-24:00)"
        );
        assert_eq!(parse("ZZZZ", "GMT+01"), "ERR DateTimeParseException: Text 'GMT+01' could not be parsed: Index 6 out of bounds for length 6");
        assert_eq!(parse("O", "GMT+1:3"), "ERR DateTimeParseException: Text 'GMT+1:3' could not be parsed, unparsed text found at index 5");
        assert_eq!(
            parse("xx", "+2400"),
            "ERR DateTimeParseException: Text '+2400' could not be parsed: Value out of range: Hour[0-23], Minute[0-59], Second[0-59]"
        );
        assert_eq!(parse("XXX", "+01:60"), "ERR DateTimeParseException: Text '+01:60' could not be parsed at index 0");
        assert_eq!(parse("VV", "UTC+01:00"), "{},ISO,UTC+01:00,-1");
        assert_eq!(parse("VV", "GMT0"), "{},ISO,GMT0,-1");
        assert_eq!(parse("VV", "Z"), "{},ISO,Z,-1");
        assert_eq!(parse("z", "UT"), "{},ISO,UT,-1");
        assert_eq!(parse("VV", "+01:00"), "{},ISO,+01:00,-1");
    }
}
