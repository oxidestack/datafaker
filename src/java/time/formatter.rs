//! `java.time.format.DateTimeFormatter.ofPattern(pattern)` in the default FORMAT locale `en_US` (Sunday-first weeks,
//! one minimal day in the first week), optionally `withZone(ZoneId.systemDefault())` (`GMT`): the printer-parsers
//! `DateTimeFormatterBuilder.appendPattern` builds (adjacent value parsing included) and formatting. Parsing is in
//! `parse.rs`.
//!
//! Every pattern letter is supported.

use super::zone::{ZoneId, offset_id};
use super::{LocalDate, LocalDateTime, LocalTime, MONTHS, WEEKDAYS};
use crate::{Error, Result};

fn illegal(message: impl Into<String>) -> Error {
    Error::illegal_argument(message)
}

fn date_time(message: impl Into<String>) -> Error {
    Error::new("DateTimeException", message)
}

fn unsupported(name: &str) -> Error {
    Error::new("UnsupportedTemporalTypeException", format!("Unsupported field: {name}"))
}

/// What is being formatted: the fields it has, and the zone and offset that can be read from it.
#[derive(Debug, Clone)]
pub(crate) struct Temporal {
    date: Option<LocalDate>,
    time: Option<LocalTime>,
    zone: Option<ZoneId>,
    offset: Option<i32>,
    text: String,
}

impl Temporal {
    pub(crate) fn date(date: LocalDate) -> Temporal {
        Temporal { date: Some(date), time: None, zone: None, offset: None, text: date.to_string() }
    }

    pub(crate) fn time(time: LocalTime) -> Temporal {
        Temporal { date: None, time: Some(time), zone: None, offset: None, text: time.to_string() }
    }

    pub(crate) fn date_time(dt: LocalDateTime) -> Temporal {
        Temporal { date: Some(dt.date()), time: Some(dt.time()), zone: None, offset: None, text: dt.to_string() }
    }

    /// An `Instant` at a zone (a `ZonedDateTime`).
    pub(crate) fn zoned(instant: super::Instant, zone: &ZoneId) -> Temporal {
        let offset = zone.offset_at(instant.epoch_second());
        let dt = LocalDateTime::of_instant_offset(instant, offset);
        let text = format!("{dt}{}{}", offset_id(offset), if zone.is_offset() { String::new() } else { format!("[{zone}]") });
        Temporal { date: Some(dt.date()), time: Some(dt.time()), zone: Some(zone.clone()), offset: Some(offset), text }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Fields

/// A `TemporalField`: the `ChronoField`s in their declaration order, then `IsoFields.QUARTER_OF_YEAR`,
/// `JulianFields.MODIFIED_JULIAN_DAY` and the fields of `WeekFields.of(en_US)` (`WeekFields[SUNDAY,1]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Field {
    NanoOfSecond,
    NanoOfDay,
    MicroOfSecond,
    MicroOfDay,
    MilliOfSecond,
    MilliOfDay,
    SecondOfMinute,
    SecondOfDay,
    MinuteOfHour,
    MinuteOfDay,
    HourOfAmPm,
    ClockHourOfAmPm,
    HourOfDay,
    ClockHourOfDay,
    AmPmOfDay,
    DayOfWeek,
    AlignedDayOfWeekInMonth,
    AlignedDayOfWeekInYear,
    DayOfMonth,
    DayOfYear,
    EpochDay,
    AlignedWeekOfMonth,
    AlignedWeekOfYear,
    MonthOfYear,
    ProlepticMonth,
    YearOfEra,
    Year,
    Era,
    InstantSeconds,
    OffsetSeconds,
    QuarterOfYear,
    ModifiedJulianDay,
    WeekBasedYear,
    WeekOfWeekBasedYear,
    WeekOfMonth,
    LocalizedDayOfWeek,
}

/// `java.time.temporal.ValueRange`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ValueRange {
    pub min: i64,
    pub largest_min: i64,
    pub smallest_max: i64,
    pub max: i64,
}

impl ValueRange {
    pub(crate) const fn of(min: i64, max: i64) -> ValueRange {
        ValueRange { min, largest_min: min, smallest_max: max, max }
    }

    const fn of3(min: i64, smallest_max: i64, max: i64) -> ValueRange {
        ValueRange { min, largest_min: min, smallest_max, max }
    }

    pub(crate) fn is_valid_value(&self, value: i64) -> bool {
        (self.min..=self.max).contains(&value)
    }

    fn is_int_value(&self) -> bool {
        self.min >= i32::MIN as i64 && self.max <= i32::MAX as i64
    }

    /// `checkValidValue(value, field)`.
    pub(crate) fn check(&self, value: i64, field: Field) -> Result<i64> {
        if !self.is_valid_value(value) {
            return Err(date_time(format!("Invalid value for {} (valid values {self}): {value}", field.name())));
        }
        Ok(value)
    }

    /// `checkValidIntValue(value, field)`.
    pub(crate) fn check_int(&self, value: i64, field: Field) -> Result<i32> {
        if !(self.is_int_value() && self.is_valid_value(value)) {
            return Err(date_time(format!("Invalid value for {} (valid values {self}): {value}", field.name())));
        }
        Ok(value as i32)
    }
}

impl std::fmt::Display for ValueRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.min)?;
        if self.min != self.largest_min {
            write!(f, "/{}", self.largest_min)?;
        }
        write!(f, " - {}", self.smallest_max)?;
        if self.smallest_max != self.max {
            write!(f, "/{}", self.max)?;
        }
        Ok(())
    }
}

const YEAR_RANGE: ValueRange = ValueRange::of(-999_999_999, 999_999_999);

impl Field {
    /// `toString()`.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Field::NanoOfSecond => "NanoOfSecond",
            Field::NanoOfDay => "NanoOfDay",
            Field::MicroOfSecond => "MicroOfSecond",
            Field::MicroOfDay => "MicroOfDay",
            Field::MilliOfSecond => "MilliOfSecond",
            Field::MilliOfDay => "MilliOfDay",
            Field::SecondOfMinute => "SecondOfMinute",
            Field::SecondOfDay => "SecondOfDay",
            Field::MinuteOfHour => "MinuteOfHour",
            Field::MinuteOfDay => "MinuteOfDay",
            Field::HourOfAmPm => "HourOfAmPm",
            Field::ClockHourOfAmPm => "ClockHourOfAmPm",
            Field::HourOfDay => "HourOfDay",
            Field::ClockHourOfDay => "ClockHourOfDay",
            Field::AmPmOfDay => "AmPmOfDay",
            Field::DayOfWeek => "DayOfWeek",
            Field::AlignedDayOfWeekInMonth => "AlignedDayOfWeekInMonth",
            Field::AlignedDayOfWeekInYear => "AlignedDayOfWeekInYear",
            Field::DayOfMonth => "DayOfMonth",
            Field::DayOfYear => "DayOfYear",
            Field::EpochDay => "EpochDay",
            Field::AlignedWeekOfMonth => "AlignedWeekOfMonth",
            Field::AlignedWeekOfYear => "AlignedWeekOfYear",
            Field::MonthOfYear => "MonthOfYear",
            Field::ProlepticMonth => "ProlepticMonth",
            Field::YearOfEra => "YearOfEra",
            Field::Year => "Year",
            Field::Era => "Era",
            Field::InstantSeconds => "InstantSeconds",
            Field::OffsetSeconds => "OffsetSeconds",
            Field::QuarterOfYear => "QuarterOfYear",
            Field::ModifiedJulianDay => "ModifiedJulianDay",
            Field::WeekBasedYear => "WeekBasedYear[WeekFields[SUNDAY,1]]",
            Field::WeekOfWeekBasedYear => "WeekOfWeekBasedYear[WeekFields[SUNDAY,1]]",
            Field::WeekOfMonth => "WeekOfMonth[WeekFields[SUNDAY,1]]",
            Field::LocalizedDayOfWeek => "DayOfWeek[WeekFields[SUNDAY,1]]",
        }
    }

    /// `range()`.
    pub(crate) fn range(self) -> ValueRange {
        match self {
            Field::NanoOfSecond => ValueRange::of(0, 999_999_999),
            Field::NanoOfDay => ValueRange::of(0, 86_400 * 1_000_000_000 - 1),
            Field::MicroOfSecond => ValueRange::of(0, 999_999),
            Field::MicroOfDay => ValueRange::of(0, 86_400 * 1_000_000 - 1),
            Field::MilliOfSecond => ValueRange::of(0, 999),
            Field::MilliOfDay => ValueRange::of(0, 86_400 * 1000 - 1),
            Field::SecondOfMinute | Field::MinuteOfHour => ValueRange::of(0, 59),
            Field::SecondOfDay => ValueRange::of(0, 86_399),
            Field::MinuteOfDay => ValueRange::of(0, 1439),
            Field::HourOfAmPm => ValueRange::of(0, 11),
            Field::ClockHourOfAmPm => ValueRange::of(1, 12),
            Field::HourOfDay => ValueRange::of(0, 23),
            Field::ClockHourOfDay => ValueRange::of(1, 24),
            Field::AmPmOfDay | Field::Era => ValueRange::of(0, 1),
            Field::DayOfWeek | Field::AlignedDayOfWeekInMonth | Field::AlignedDayOfWeekInYear | Field::LocalizedDayOfWeek => {
                ValueRange::of(1, 7)
            }
            Field::DayOfMonth => ValueRange::of3(1, 28, 31),
            Field::DayOfYear => ValueRange::of3(1, 365, 366),
            Field::EpochDay => ValueRange::of(-365_243_219_162, 365_241_780_471),
            Field::AlignedWeekOfMonth => ValueRange::of3(1, 4, 5),
            Field::AlignedWeekOfYear => ValueRange::of(1, 53),
            Field::MonthOfYear => ValueRange::of(1, 12),
            Field::ProlepticMonth => ValueRange::of(-999_999_999 * 12, 999_999_999 * 12 + 11),
            Field::YearOfEra => ValueRange::of3(1, 999_999_999, 1_000_000_000),
            Field::Year | Field::WeekBasedYear => YEAR_RANGE,
            Field::InstantSeconds => ValueRange::of(-31_557_014_167_219_200, 31_556_889_864_403_199),
            Field::OffsetSeconds => ValueRange::of(-18 * 3600, 18 * 3600),
            Field::QuarterOfYear => ValueRange::of(1, 4),
            Field::ModifiedJulianDay => ValueRange::of(-365_243_219_162 + 40_587, 365_241_780_471 + 40_587),
            Field::WeekOfWeekBasedYear => ValueRange::of3(1, 52, 53),
            Field::WeekOfMonth => ValueRange { min: 0, largest_min: 1, smallest_max: 4, max: 6 },
        }
    }

    /// Whether this is a `ChronoField`.
    pub(crate) fn is_chrono(self) -> bool {
        self <= Field::OffsetSeconds
    }

    /// `ChronoField.isTimeBased()`.
    pub(crate) fn is_time_based(self) -> bool {
        self < Field::DayOfWeek
    }

    /// `ChronoField.isDateBased()`.
    pub(crate) fn is_date_based(self) -> bool {
        (Field::DayOfWeek..=Field::Era).contains(&self)
    }

    fn of_letter(c: char) -> Option<Field> {
        Some(match c {
            'G' => Field::Era,
            'y' => Field::YearOfEra,
            'u' => Field::Year,
            'Q' | 'q' => Field::QuarterOfYear,
            'M' | 'L' => Field::MonthOfYear,
            'D' => Field::DayOfYear,
            'd' => Field::DayOfMonth,
            'F' => Field::AlignedWeekOfMonth,
            'E' | 'c' | 'e' => Field::DayOfWeek,
            'a' => Field::AmPmOfDay,
            'H' => Field::HourOfDay,
            'k' => Field::ClockHourOfDay,
            'K' => Field::HourOfAmPm,
            'h' => Field::ClockHourOfAmPm,
            'm' => Field::MinuteOfHour,
            's' => Field::SecondOfMinute,
            'S' | 'n' => Field::NanoOfSecond,
            'A' => Field::MilliOfDay,
            'N' => Field::NanoOfDay,
            'g' => Field::ModifiedJulianDay,
            _ => return None,
        })
    }
}

/// WeekFields of `en_US`.
pub(crate) const FIRST_DAY_OF_WEEK: i64 = 7;
pub(crate) const MINIMAL_DAYS: i64 = 1;

pub(crate) fn start_of_week_offset(day: i64, dow: i64) -> i64 {
    let week_start = (day - dow).rem_euclid(7);
    if week_start + 1 > MINIMAL_DAYS { 7 - week_start } else { -week_start }
}

pub(crate) fn compute_week(offset: i64, day: i64) -> i64 {
    (7 + offset + (day - 1)) / 7
}

/// The localized day of week of an ISO one (Sunday 1 to Saturday 7).
pub(crate) fn localized_dow_of(iso_dow: i64) -> i64 {
    (iso_dow - FIRST_DAY_OF_WEEK).rem_euclid(7) + 1
}

fn localized_dow(date: &LocalDate) -> i64 {
    localized_dow_of(date.day_of_week() as i64)
}

fn week_of_week_based_year(date: &LocalDate) -> i64 {
    let dow = localized_dow(date);
    let doy = date.day_of_year() as i64;
    let offset = start_of_week_offset(doy, dow);
    let mut week = compute_week(offset, doy);
    if week == 0 {
        return week_of_week_based_year(&date.minus_days(doy).expect("in range"));
    }
    if week > 50 {
        let new_year_week = compute_week(offset, date.length_of_year() as i64 + MINIMAL_DAYS);
        if week >= new_year_week {
            week = week - new_year_week + 1;
        }
    }
    week
}

fn week_based_year(date: &LocalDate) -> i64 {
    let dow = localized_dow(date);
    let year = date.year() as i64;
    let doy = date.day_of_year() as i64;
    let offset = start_of_week_offset(doy, dow);
    let week = compute_week(offset, doy);
    if week == 0 {
        return year - 1;
    }
    let new_year_week = compute_week(offset, date.length_of_year() as i64 + MINIMAL_DAYS);
    if week >= new_year_week { year + 1 } else { year }
}

/// `localizedWeekOfMonth(date)`.
pub(crate) fn week_of_month(date: &LocalDate) -> i64 {
    let dom = date.day_of_month() as i64;
    compute_week(start_of_week_offset(dom, localized_dow(date)), dom)
}

/// `LocalDate.getLong(field)` (and what the other fields read from a date); `None` when the field is not supported.
pub(crate) fn date_field(d: &LocalDate, field: Field) -> Option<i64> {
    let year = d.year() as i64;
    let (dom, doy) = (d.day_of_month() as i64, d.day_of_year() as i64);
    Some(match field {
        Field::DayOfWeek => d.day_of_week() as i64,
        Field::AlignedDayOfWeekInMonth => (dom - 1) % 7 + 1,
        Field::AlignedDayOfWeekInYear => (doy - 1) % 7 + 1,
        Field::DayOfMonth => dom,
        Field::DayOfYear => doy,
        Field::EpochDay => d.to_epoch_day(),
        Field::AlignedWeekOfMonth => (dom - 1) / 7 + 1,
        Field::AlignedWeekOfYear => (doy - 1) / 7 + 1,
        Field::MonthOfYear => d.month_value() as i64,
        Field::ProlepticMonth => year * 12 + d.month_value() as i64 - 1,
        Field::YearOfEra => if year >= 1 { year } else { 1 - year },
        Field::Year => year,
        Field::Era => i64::from(year >= 1),
        Field::QuarterOfYear => (d.month_value() as i64 - 1) / 3 + 1,
        Field::ModifiedJulianDay => d.to_epoch_day() + 40_587,
        Field::WeekBasedYear => week_based_year(d),
        Field::WeekOfWeekBasedYear => week_of_week_based_year(d),
        Field::WeekOfMonth => week_of_month(d),
        Field::LocalizedDayOfWeek => localized_dow(d),
        _ => return None,
    })
}

/// `LocalTime.getLong(field)`; `None` when the field is not supported.
pub(crate) fn time_field(t: &LocalTime, field: Field) -> Option<i64> {
    let nod = t.to_nano_of_day();
    let hour = t.hour() as i64;
    Some(match field {
        Field::NanoOfSecond => t.nano() as i64,
        Field::NanoOfDay => nod,
        Field::MicroOfSecond => t.nano() as i64 / 1000,
        Field::MicroOfDay => nod / 1000,
        Field::MilliOfSecond => t.nano() as i64 / 1_000_000,
        Field::MilliOfDay => nod / 1_000_000,
        Field::SecondOfMinute => t.second() as i64,
        Field::SecondOfDay => nod / 1_000_000_000,
        Field::MinuteOfHour => t.minute() as i64,
        Field::MinuteOfDay => hour * 60 + t.minute() as i64,
        Field::HourOfAmPm => hour % 12,
        Field::ClockHourOfAmPm => if hour % 12 == 0 { 12 } else { hour % 12 },
        Field::HourOfDay => hour,
        Field::ClockHourOfDay => if hour == 0 { 24 } else { hour },
        Field::AmPmOfDay => hour / 12,
        _ => return None,
    })
}

fn is_week_or_iso_field(field: Field) -> bool {
    field > Field::OffsetSeconds
}

impl Temporal {
    fn supports(&self, field: Field) -> bool {
        if field.is_date_based() || is_week_or_iso_field(field) {
            self.date.is_some()
        } else if field == Field::OffsetSeconds {
            self.offset.is_some()
        } else if field == Field::InstantSeconds {
            self.offset.is_some() && self.date.is_some() && self.time.is_some()
        } else {
            self.time.is_some()
        }
    }

    fn get(&self, field: Field) -> Result<i64> {
        if is_week_or_iso_field(field) && field != Field::QuarterOfYear && field != Field::ModifiedJulianDay {
            let date = self.date.ok_or_else(|| unsupported("DayOfWeek"))?;
            return Ok(date_field(&date, field).expect("a week field"));
        }
        if field.is_date_based() || is_week_or_iso_field(field) {
            let d = self.date.ok_or_else(|| unsupported(field.name()))?;
            return Ok(date_field(&d, field).expect("a date field"));
        }
        if field == Field::OffsetSeconds {
            return self.offset.map(i64::from).ok_or_else(|| unsupported(field.name()));
        }
        let t = self.time.ok_or_else(|| unsupported(field.name()))?;
        time_field(&t, field).ok_or_else(|| unsupported(field.name()))
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Printer-parsers

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignStyle {
    Normal,
    NotNegative,
    ExceedsPad,
}

impl SignStyle {
    /// `SignStyle.parse(positive, strict, fixedWidth)` in strict mode.
    pub(crate) fn parse_strict(self, positive: bool) -> bool {
        match self {
            SignStyle::Normal => !positive,
            SignStyle::ExceedsPad => true,
            SignStyle::NotNegative => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextStyle {
    Short,
    Full,
    Narrow,
}

/// `NumberPrinterParser` and its subclasses `ReducedPrinterParser` (two digits on the base value 2000) and
/// `WeekBasedFieldPrinterParser` (`outer`: the widths its builder sees, for adjacent value parsing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Number {
    pub field: Field,
    pub min: usize,
    pub max: usize,
    pub sign: SignStyle,
    /// `subsequentWidth`: -1 for fixed width, otherwise the width of the adjacent fixed-width values after it.
    pub subsequent: i32,
    pub reduced: bool,
    pub outer: Option<(usize, usize)>,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Item {
    /// `CharLiteralPrinterParser` or `StringLiteralPrinterParser`, as UTF-16 units.
    Literal(Vec<u16>),
    Number(Number),
    Text(Field, TextStyle),
    /// `NanosPrinterParser`: the fraction of the second.
    Fraction { min: usize, max: usize, decimal_point: bool, subsequent: i32 },
    ZoneText(TextStyle, bool),
    ZoneId,
    /// `OffsetIdPrinterParser`: the index of its pattern in `PATTERNS` (`+HH` 0 … `+H:mm:ss` 21) and the text of
    /// a zero offset.
    Offset(usize, &'static str),
    LocalizedOffset(TextStyle),
    /// `B`: `en` day periods.
    DayPeriod(TextStyle),
    /// `InstantPrinterParser` (`ISO_INSTANT`).
    Instant,
    Optional(Vec<Item>),
    Pad(usize, Box<Item>),
}

impl Item {
    /// The widths and whether the sign style is `NOT_NEGATIVE`, as `appendValue` sees a value printer-parser.
    fn adjacency(&self) -> (usize, usize, bool) {
        match self {
            Item::Number(n) => match n.outer {
                Some((min, max)) => (min, max, true),
                None => (n.min, n.max, n.sign == SignStyle::NotNegative),
            },
            Item::Fraction { min, max, .. } => (*min, *max, true),
            _ => unreachable!("not a value printer-parser"),
        }
    }

    fn subsequent_mut(&mut self) -> &mut i32 {
        match self {
            Item::Number(n) => &mut n.subsequent,
            Item::Fraction { subsequent, .. } => subsequent,
            _ => unreachable!("not a value printer-parser"),
        }
    }

    fn with_fixed_width(mut self) -> Item {
        *self.subsequent_mut() = -1;
        self
    }
}

/// A compiled `DateTimeFormatter`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeFormatter {
    pub(crate) items: Vec<Item>,
    pub(crate) zone: Option<ZoneId>,
    pub(crate) case_sensitive: bool,
    /// `ResolverStyle.STRICT` (otherwise `SMART`).
    pub(crate) strict_resolver: bool,
}

fn value(field: Field, min: usize, max: usize, sign: SignStyle) -> Result<Item> {
    if min == max && sign == SignStyle::NotNegative {
        if !(1..=19).contains(&min) {
            return Err(illegal(format!("The width must be from 1 to 19 inclusive but was {min}")));
        }
    } else {
        if !(1..=19).contains(&min) {
            return Err(illegal(format!("The minimum width must be from 1 to 19 inclusive but was {min}")));
        }
        if !(1..=19).contains(&max) {
            return Err(illegal(format!("The maximum width must be from 1 to 19 inclusive but was {max}")));
        }
        if max < min {
            return Err(illegal(format!("The maximum width must exceed or equal the minimum width but {max} < {min}")));
        }
    }
    Ok(number(field, min, max, sign))
}

fn number(field: Field, min: usize, max: usize, sign: SignStyle) -> Item {
    Item::Number(Number { field, min, max, sign, subsequent: 0, reduced: false, outer: None })
}

fn reduced(field: Field) -> Item {
    Item::Number(Number { field, min: 2, max: 2, sign: SignStyle::NotNegative, subsequent: 0, reduced: true, outer: None })
}

fn too_many(c: char) -> Error {
    illegal(format!("Too many pattern letters: {c}"))
}

fn text_style(count: usize, c: char) -> Result<TextStyle> {
    match count {
        1..=3 => Ok(TextStyle::Short),
        4 => Ok(TextStyle::Full),
        5 => Ok(TextStyle::Narrow),
        _ => Err(too_many(c)),
    }
}

/// One level of `DateTimeFormatterBuilder`: the main one or an open optional section.
#[derive(Default)]
struct Level {
    items: Vec<Item>,
    /// `valueParserIndex`.
    value_parser: Option<usize>,
}

/// `DateTimeFormatterBuilder` as `appendPattern` drives it.
struct Builder {
    stack: Vec<Level>,
    pad: Option<usize>,
}

impl Builder {
    fn active(&mut self) -> &mut Level {
        self.stack.last_mut().expect("root")
    }

    /// `appendInternal(pp)`: pads it if `padNext` was called.
    fn append_internal(&mut self, item: Item) -> usize {
        let item = match self.pad.take() {
            Some(width) => Item::Pad(width, Box::new(item)),
            None => item,
        };
        let active = self.active();
        active.items.push(item);
        active.value_parser = None;
        active.items.len() - 1
    }

    /// `appendValue(NumberPrinterParser)`: adjacent value parsing.
    fn append_value(&mut self, item: Item) -> Result<()> {
        let Some(base) = self.active().value_parser else {
            let index = self.append_internal(item);
            self.active().value_parser = Some(index);
            return Ok(());
        };
        if let Item::Pad(..) = self.active().items[base] {
            return Err(Error::new(
                "ClassCastException",
                "class java.time.format.DateTimeFormatterBuilder$PadPrinterParserDecorator cannot be cast to class \
                 java.time.format.DateTimeFormatterBuilder$NumberPrinterParser (java.time.format.DateTimeFormatterBuilder$\
                 PadPrinterParserDecorator and java.time.format.DateTimeFormatterBuilder$NumberPrinterParser are in module \
                 java.base of loader 'bootstrap')",
            ));
        }
        let (min, max, not_negative) = item.adjacency();
        if min == max && not_negative {
            *self.active().items[base].subsequent_mut() += max as i32;
            self.append_internal(item.with_fixed_width());
            self.active().value_parser = Some(base);
        } else {
            *self.active().items[base].subsequent_mut() = -1;
            let index = self.append_internal(item);
            self.active().value_parser = Some(index);
        }
        Ok(())
    }

    /// `optionalEnd()`.
    fn optional_end(&mut self) {
        let section = self.stack.pop().expect("open section");
        if !section.items.is_empty() {
            self.append_internal(Item::Optional(section.items));
        }
    }

    /// `DateTimeFormatterBuilder.parseField`.
    fn parse_field(&mut self, c: char, count: usize, field: Field) -> Result<()> {
        match c {
            'u' | 'y' => {
                if count == 2 {
                    self.append_value(reduced(field))
                } else if count < 4 {
                    self.append_value(value(field, count, 19, SignStyle::Normal)?)
                } else {
                    self.append_value(value(field, count, 19, SignStyle::ExceedsPad)?)
                }
            }
            'c' | 'L' | 'q' | 'M' | 'Q' | 'E' | 'e' => {
                if c == 'c' && count == 1 {
                    return self.append_value(week_based('c', count, count, count));
                }
                if c == 'c' && count == 2 {
                    return Err(illegal("Invalid pattern \"cc\""));
                }
                match count {
                    1 | 2 if c == 'e' => self.append_value(week_based('e', count, count, count)),
                    1 | 2 if c == 'E' => {
                        self.append_internal(Item::Text(field, TextStyle::Short));
                        Ok(())
                    }
                    1 => self.append_value(value(field, 1, 19, SignStyle::Normal)?),
                    2 => self.append_value(value(field, 2, 2, SignStyle::NotNegative)?),
                    3..=5 => {
                        self.append_internal(Item::Text(field, text_style(count, c)?));
                        Ok(())
                    }
                    _ => Err(too_many(c)),
                }
            }
            'a' => {
                if count != 1 {
                    return Err(too_many(c));
                }
                self.append_internal(Item::Text(field, TextStyle::Short));
                Ok(())
            }
            'G' => {
                self.append_internal(Item::Text(field, text_style(count, c)?));
                Ok(())
            }
            'S' => {
                if count > 9 {
                    return Err(illegal(format!("Minimum width must be from 0 to 9 inclusive but was {count}")));
                }
                self.append_value(Item::Fraction { min: count, max: count, decimal_point: false, subsequent: 0 })
            }
            'F' => if count == 1 { self.append_value(value(field, 1, 19, SignStyle::Normal)?) } else { Err(too_many(c)) },
            'd' | 'h' | 'H' | 'k' | 'K' | 'm' | 's' => match count {
                1 => self.append_value(value(field, 1, 19, SignStyle::Normal)?),
                2 => self.append_value(value(field, 2, 2, SignStyle::NotNegative)?),
                _ => Err(too_many(c)),
            },
            'D' => match count {
                1 => self.append_value(value(field, 1, 19, SignStyle::Normal)?),
                2 | 3 => self.append_value(value(field, count, 3, SignStyle::NotNegative)?),
                _ => Err(too_many(c)),
            },
            'g' => self.append_value(value(field, count, 19, SignStyle::Normal)?),
            // 'A', 'n', 'N'
            _ => self.append_value(value(field, count, 19, SignStyle::NotNegative)?),
        }
    }
}

/// `WeekBasedFieldPrinterParser(chr, count, minWidth, maxWidth)` and the printer-parser it delegates to.
fn week_based(c: char, count: usize, min: usize, max: usize) -> Item {
    let n = match c {
        'Y' if count == 2 => {
            Number { field: Field::WeekBasedYear, min: 2, max: 2, sign: SignStyle::NotNegative, subsequent: 0, reduced: true, outer: None }
        }
        'Y' => {
            let sign = if count < 4 { SignStyle::Normal } else { SignStyle::ExceedsPad };
            Number { field: Field::WeekBasedYear, min: count, max: 19, sign, subsequent: 0, reduced: false, outer: None }
        }
        _ => {
            let field = match c {
                'w' => Field::WeekOfWeekBasedYear,
                'W' => Field::WeekOfMonth,
                _ => Field::LocalizedDayOfWeek,
            };
            Number { field, min, max, sign: SignStyle::NotNegative, subsequent: 0, reduced: false, outer: None }
        }
    };
    Item::Number(Number { outer: Some((min, max)), ..n })
}

impl DateTimeFormatter {
    /// `DateTimeFormatter.ofPattern(pattern)`.
    pub fn of_pattern(pattern: &str) -> Result<DateTimeFormatter> {
        let chars: Vec<char> = pattern.chars().collect();
        let mut b = Builder { stack: vec![Level::default()], pad: None };
        let mut pos = 0;
        while pos < chars.len() {
            let mut cur = chars[pos];
            if cur.is_ascii_alphabetic() {
                let mut start = pos;
                pos += 1;
                while pos < chars.len() && chars[pos] == cur {
                    pos += 1;
                }
                let mut count = pos - start;
                if cur == 'p' {
                    let mut width = 0;
                    if pos < chars.len() && chars[pos].is_ascii_alphabetic() {
                        cur = chars[pos];
                        width = count;
                        start = pos;
                        pos += 1;
                        while pos < chars.len() && chars[pos] == cur {
                            pos += 1;
                        }
                        count = pos - start;
                    }
                    if width == 0 {
                        return Err(illegal(format!("Pad letter 'p' must be followed by valid pad pattern: {pattern}")));
                    }
                    // `padNext(width)`.
                    b.pad = Some(width);
                    b.active().value_parser = None;
                }
                if let Some(field) = Field::of_letter(cur) {
                    b.parse_field(cur, count, field)?;
                    continue;
                }
                let item = match cur {
                    'z' => match count {
                        1..=3 => Item::ZoneText(TextStyle::Short, false),
                        4 => Item::ZoneText(TextStyle::Full, false),
                        _ => return Err(too_many(cur)),
                    },
                    'V' => {
                        if count != 2 {
                            return Err(illegal(format!("Pattern letter count must be 2: {cur}")));
                        }
                        Item::ZoneId
                    }
                    'v' => match count {
                        1 => Item::ZoneText(TextStyle::Short, true),
                        4 => Item::ZoneText(TextStyle::Full, true),
                        _ => return Err(illegal(format!("Wrong number of pattern letters: {cur}"))),
                    },
                    'Z' => match count {
                        1..=3 => Item::Offset(3, "+0000"),
                        4 => Item::LocalizedOffset(TextStyle::Full),
                        5 => Item::Offset(6, "Z"),
                        _ => return Err(too_many(cur)),
                    },
                    'O' => match count {
                        1 => Item::LocalizedOffset(TextStyle::Short),
                        4 => Item::LocalizedOffset(TextStyle::Full),
                        _ => return Err(illegal(format!("Pattern letter count must be 1 or 4: {cur}"))),
                    },
                    'X' => {
                        if count > 5 {
                            return Err(too_many(cur));
                        }
                        Item::Offset(count + usize::from(count != 1), "Z")
                    }
                    'x' => {
                        if count > 5 {
                            return Err(too_many(cur));
                        }
                        let zero = if count == 1 { "+00" } else if count % 2 == 0 { "+0000" } else { "+00:00" };
                        Item::Offset(count + usize::from(count != 1), zero)
                    }
                    'W' => {
                        if count > 1 {
                            return Err(too_many(cur));
                        }
                        b.append_value(week_based(cur, count, count, count))?;
                        continue;
                    }
                    'w' => {
                        if count > 2 {
                            return Err(too_many(cur));
                        }
                        b.append_value(week_based(cur, count, count, 2))?;
                        continue;
                    }
                    'Y' => {
                        b.append_value(week_based(cur, count, count, if count == 2 { 2 } else { 19 }))?;
                        continue;
                    }
                    'B' => match count {
                        1 => Item::DayPeriod(TextStyle::Short),
                        4 => Item::DayPeriod(TextStyle::Full),
                        5 => Item::DayPeriod(TextStyle::Narrow),
                        _ => return Err(illegal(format!("Wrong number of pattern letters: {cur}"))),
                    },
                    _ => return Err(illegal(format!("Unknown pattern letter: {cur}"))),
                };
                b.append_internal(item);
            } else if cur == '\'' {
                let start = pos;
                pos += 1;
                while pos < chars.len() {
                    if chars[pos] == '\'' {
                        if pos + 1 < chars.len() && chars[pos + 1] == '\'' {
                            pos += 1;
                        } else {
                            break;
                        }
                    }
                    pos += 1;
                }
                if pos >= chars.len() {
                    return Err(illegal(format!("Pattern ends with an incomplete string literal: {pattern}")));
                }
                let s: String = chars[start + 1..pos].iter().collect();
                let literal = if s.is_empty() { "'".to_string() } else { s.replace("''", "'") };
                b.append_internal(Item::Literal(literal.encode_utf16().collect()));
                pos += 1;
            } else if cur == '[' {
                // `optionalStart()`.
                b.active().value_parser = None;
                b.stack.push(Level::default());
                pos += 1;
            } else if cur == ']' {
                if b.stack.len() == 1 {
                    return Err(illegal("Pattern invalid as it contains ] without previous ["));
                }
                b.optional_end();
                pos += 1;
            } else if matches!(cur, '{' | '}' | '#') {
                return Err(illegal(format!("Pattern includes reserved character: '{cur}'")));
            } else {
                // `appendLiteral(char)`, one UTF-16 unit at a time.
                let mut units = [0u16; 2];
                for unit in cur.encode_utf16(&mut units) {
                    b.append_internal(Item::Literal(vec![*unit]));
                }
                pos += 1;
            }
        }
        while b.stack.len() > 1 {
            b.optional_end();
        }
        let items = b.stack.pop().expect("root").items;
        Ok(DateTimeFormatter { items, zone: None, case_sensitive: true, strict_resolver: false })
    }

    /// `DateTimeFormatter.ISO_INSTANT`: case insensitive, the STRICT resolver.
    pub(crate) fn iso_instant() -> DateTimeFormatter {
        DateTimeFormatter { items: vec![Item::Instant], zone: None, case_sensitive: false, strict_resolver: true }
    }

    /// `withResolverStyle(ResolverStyle.STRICT)`.
    pub fn with_strict_resolver(mut self) -> DateTimeFormatter {
        self.strict_resolver = true;
        self
    }

    /// `withZone(ZoneId.systemDefault())`.
    pub fn with_default_zone(self) -> DateTimeFormatter {
        self.with_zone(super::default_zone())
    }

    /// `withZone(zone)`.
    pub fn with_zone(mut self, zone: ZoneId) -> DateTimeFormatter {
        self.zone = Some(zone);
        self
    }

    /// `format(instant)`; the formatter needs its zone (`with_default_zone`), as an `Instant` has no date fields.
    pub fn format_instant(&self, instant: super::Instant) -> Result<String> {
        let Some(zone) = &self.zone else { return Err(unsupported("YearOfEra")) };
        self.format(&Temporal::zoned(instant, zone))
    }

    /// `format(temporal)`.
    pub(crate) fn format(&self, temporal: &Temporal) -> Result<String> {
        let mut t = temporal.clone();
        if let Some(zone) = &self.zone && t.zone.is_none() {
            t.zone = Some(zone.clone());
        }
        let mut buf = String::new();
        format_items(&self.items, &t, 0, &mut buf)?;
        Ok(buf)
    }
}

/// Formats `items`; `Ok(false)` when an optional section lacks a value.
fn format_items(items: &[Item], t: &Temporal, optional: usize, buf: &mut String) -> Result<bool> {
    for item in items {
        if !format_item(item, t, optional, buf)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn format_item(item: &Item, t: &Temporal, optional: usize, buf: &mut String) -> Result<bool> {
    let get = |field: Field| -> Result<Option<i64>> {
        if optional > 0 && !t.supports(field) {
            return Ok(None);
        }
        t.get(field).map(Some)
    };
    let no_zone = |buf: &mut String| -> Result<bool> {
        let _ = buf;
        if optional > 0 {
            return Ok(false);
        }
        Err(date_time(format!("Unable to extract ZoneId from temporal {}", t.text)))
    };
    match item {
        Item::Literal(s) => buf.push_str(&String::from_utf16_lossy(s)),
        Item::Optional(section) => {
            let len = buf.len();
            if !format_items(section, t, optional + 1, buf)? {
                buf.truncate(len);
            }
        }
        Item::Pad(width, inner) => {
            let pre = buf.len();
            if !format_item(inner, t, optional, buf)? {
                return Ok(false);
            }
            let len = buf[pre..].encode_utf16().count();
            if len > *width {
                return Err(date_time(format!("Cannot print as output of {len} characters exceeds pad width of {width}")));
            }
            buf.insert_str(pre, &" ".repeat(width - len));
        }
        Item::Number(n) if n.reduced => {
            let Some(value) = get(n.field)? else { return Ok(false) };
            buf.push_str(&format!("{:02}", value.unsigned_abs() % 100));
        }
        Item::Number(Number { field, min, max, sign, .. }) => {
            let Some(value) = get(*field)? else { return Ok(false) };
            let digits = value.unsigned_abs().to_string();
            if digits.len() > *max {
                return Err(date_time(format!(
                    "Field {} cannot be printed as the value {value} exceeds the maximum print width of {max}",
                    field.name()
                )));
            }
            if value >= 0 {
                if *sign == SignStyle::ExceedsPad && *min < 19 && digits.len() > *min {
                    buf.push('+');
                }
            } else {
                if *sign == SignStyle::NotNegative {
                    return Err(date_time(format!(
                        "Field {} cannot be printed as the value {value} cannot be negative according to the SignStyle",
                        field.name()
                    )));
                }
                buf.push('-');
            }
            for _ in digits.len()..*min {
                buf.push('0');
            }
            buf.push_str(&digits);
        }
        Item::Fraction { min, max, decimal_point, .. } => {
            let Some(value) = get(Field::NanoOfSecond)? else { return Ok(false) };
            let mut val = value;
            let size = val.to_string().len();
            if val == 0 || size < 10 - max {
                let width = if val == 0 { *min } else { *max };
                if width > 0 {
                    if *decimal_point {
                        buf.push('.');
                    }
                    buf.push_str(&"0".repeat(width));
                }
            } else {
                if *decimal_point {
                    buf.push('.');
                }
                buf.push_str(&"0".repeat(9 - size));
                val /= 10_i64.pow(9 - *max as u32);
                let mut i = *max;
                while i > *min && val % 10 == 0 {
                    val /= 10;
                    i -= 1;
                }
                buf.push_str(&val.to_string());
            }
        }
        Item::Text(field, style) => {
            let Some(value) = get(*field)? else { return Ok(false) };
            buf.push_str(&text(*field, *style, value));
        }
        Item::ZoneText(style, generic) => {
            let Some(zone) = &t.zone else { return no_zone(buf) };
            if zone.is_offset() {
                buf.push_str(zone.id());
            } else {
                buf.push_str(&zone_name(zone, *style, *generic, t));
            }
        }
        Item::ZoneId => {
            let Some(zone) = &t.zone else { return no_zone(buf) };
            buf.push_str(zone.id());
        }
        Item::Offset(pattern, no_offset) => {
            let Some(total) = offset_value(t, optional)? else { return Ok(false) };
            format_offset(*pattern, no_offset, total, buf);
        }
        Item::LocalizedOffset(style) => {
            let Some(total) = offset_value(t, optional)? else { return Ok(false) };
            buf.push_str("GMT");
            if total != 0 {
                let abs = total.unsigned_abs();
                let (h, m, s) = (abs / 3600 % 100, abs / 60 % 60, abs % 60);
                buf.push(if total < 0 { '-' } else { '+' });
                if *style == TextStyle::Full {
                    buf.push_str(&format!("{h:02}:{m:02}"));
                    if s != 0 {
                        buf.push_str(&format!(":{s:02}"));
                    }
                } else {
                    buf.push_str(&h.to_string());
                    if m != 0 || s != 0 {
                        buf.push_str(&format!(":{m:02}"));
                        if s != 0 {
                            buf.push_str(&format!(":{s:02}"));
                        }
                    }
                }
            }
        }
        Item::DayPeriod(style) => {
            let Some(hour) = get(Field::HourOfDay)? else { return Ok(false) };
            let minute = t.time.map_or(0, |t| t.minute()) as i64;
            let narrow = *style == TextStyle::Narrow;
            buf.push_str(match hour * 60 + minute {
                0 => if narrow { "mi" } else { "midnight" },
                720 => if narrow { "n" } else { "noon" },
                m if m < 720 => "in the morning",
                m if m < 1080 => "in the afternoon",
                m if m < 1260 => "in the evening",
                _ => "at night",
            });
        }
        Item::Instant => {
            let (Some(date), Some(time), Some(offset)) = (t.date, t.time, t.offset) else {
                return Err(unsupported("InstantSeconds"));
            };
            let instant = LocalDateTime::of(date, time).to_instant_at(offset);
            buf.push_str(&instant.to_string());
        }
    }
    Ok(true)
}

/// The offset of the temporal, `None` in an optional section that lacks it.
fn offset_value(t: &Temporal, optional: usize) -> Result<Option<i32>> {
    match t.offset {
        Some(o) => Ok(Some(o)),
        None if optional > 0 => Ok(None),
        None => Err(unsupported("OffsetSeconds")),
    }
}

/// `OffsetIdPrinterParser.format`.
fn format_offset(pattern: usize, no_offset: &str, total: i32, buf: &mut String) {
    if total == 0 {
        buf.push_str(no_offset);
        return;
    }
    let style = pattern % 11;
    let padded_hour = pattern < 11;
    let colon = style > 0 && style.is_multiple_of(2);
    let abs = total.unsigned_abs();
    let (h, m, s) = (abs / 3600 % 100, abs / 60 % 60, abs % 60);
    let start = buf.len();
    let mut output = h;
    buf.push(if total < 0 { '-' } else { '+' });
    if padded_hour || h >= 10 {
        buf.push_str(&format!("{h:02}"));
    } else {
        buf.push_str(&h.to_string());
    }
    let pad = |buf: &mut String, v: u32| buf.push_str(&format!("{}{v:02}", if colon { ":" } else { "" }));
    if (3..=8).contains(&style) || (style >= 9 && s > 0) || (style >= 1 && m > 0) {
        pad(buf, m);
        output += m;
        if style == 7 || style == 8 || (style >= 5 && s > 0) {
            pad(buf, s);
            output += s;
        }
    }
    if output == 0 {
        buf.truncate(start);
        buf.push_str(no_offset);
    }
}

/// `ZoneTextPrinterParser.format`: the `en_US` name of a region zone, daylight or standard as the temporal's instant
/// (or its local date-time, outside a transition) says, otherwise generic; the id when the JDK has no name.
fn zone_name(zone: &ZoneId, style: TextStyle, generic: bool, t: &Temporal) -> String {
    let mut kind = 2;
    if !generic {
        if let (Some(date), Some(time), Some(offset)) = (t.date, t.time, t.offset) {
            let instant = LocalDateTime::of(date, time).to_instant_at(offset);
            kind = usize::from(zone.is_daylight_savings(instant.epoch_second()));
        } else if let (Some(date), Some(time)) = (t.date, t.time) {
            let local = LocalDateTime::of(date, time).to_epoch_second();
            if !zone.has_transition_at_local(local) {
                let offset = zone.offset_for_local(local, false);
                kind = usize::from(zone.is_daylight_savings(local - offset as i64));
            }
        }
    }
    zone.formatter_name(kind, style == TextStyle::Full).map_or_else(|| zone.id().to_string(), str::to_string)
}

/// `en_US` text of a field value.
fn text(field: Field, style: TextStyle, value: i64) -> String {
    let pick = |full: &str, short_len: usize| -> String {
        match style {
            TextStyle::Full => full.to_string(),
            TextStyle::Short => full.chars().take(short_len).collect(),
            TextStyle::Narrow => full.chars().take(1).collect(),
        }
    };
    match field {
        Field::MonthOfYear => pick(MONTHS[value as usize - 1], 3),
        Field::DayOfWeek => pick(WEEKDAYS[value as usize - 1], 3),
        Field::AmPmOfDay => match style {
            TextStyle::Narrow => (if value == 0 { "a" } else { "p" }).to_string(),
            _ => (if value == 0 { "AM" } else { "PM" }).to_string(),
        },
        Field::Era => match (style, value) {
            (TextStyle::Full, 1) => "Anno Domini".into(),
            (TextStyle::Full, _) => "Before Christ".into(),
            (TextStyle::Short, 1) => "AD".into(),
            (TextStyle::Short, _) => "BC".into(),
            (TextStyle::Narrow, 1) => "A".into(),
            (TextStyle::Narrow, _) => "B".into(),
        },
        Field::QuarterOfYear => match style {
            TextStyle::Full => format!("{}{} quarter", value, ["st", "nd", "rd", "th"][value as usize - 1]),
            TextStyle::Short => format!("Q{value}"),
            TextStyle::Narrow => value.to_string(),
        },
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::java::time::Instant;

    fn fmt(pattern: &str, instant: Instant) -> String {
        match DateTimeFormatter::of_pattern(pattern).and_then(|f| f.with_default_zone().format_instant(instant)) {
            Ok(s) => s,
            Err(e) => format!("ERR {}: {}", e.simple_class(), e.message()),
        }
    }

    #[test]
    fn patterns() {
        let i = Instant::of_epoch_milli(1_700_000_000_123);
        assert_eq!(fmt("yyyy-MM-dd HH:mm:ss.SSS", i), "2023-11-14 22:13:20.123");
        assert_eq!(fmt("yyyyMMddHHmmss", i), "20231114221320");
        assert_eq!(fmt("yy DDD hh:mm:ss a", i), "23 318 10:13:20 PM");
        assert_eq!(fmt("EEEE, MMMM d uuuu G", i), "Tuesday, November 14 2023 AD");
        assert_eq!(fmt("EEE MMM QQQ QQQQ", i), "Tue Nov Q4 4th quarter");
        assert_eq!(fmt("'at' h 'o''clock' ''", i), "at 10 o'clock '");
        assert_eq!(fmt("z zzzz VV X xxx Z ZZZZ O", i), "GMT Greenwich Mean Time GMT Z +00:00 +0000 GMT GMT");
        assert_eq!(fmt("Y w W e c F", i), "2023 46 3 3 3 2");
        assert_eq!(fmt("[HH[:mm]] ppH pppH B", i), "22:13 22  22 at night");
        assert_eq!(fmt("ddd", i), "ERR IllegalArgumentException: Too many pattern letters: d");
        assert_eq!(fmt("b", i), "ERR IllegalArgumentException: Unknown pattern letter: b");
        assert_eq!(fmt("#", i), "ERR IllegalArgumentException: Pattern includes reserved character: '#'");
        assert_eq!(fmt("ppHmm", i).split(':').next(), Some("ERR ClassCastException"));
        let date = LocalDate::of(2024, 1, 2).unwrap();
        let f = DateTimeFormatter::of_pattern("yyyy-MM-dd HH").unwrap();
        assert_eq!(date.format(&f).unwrap_err().message(), "Unsupported field: HourOfDay");
        let f = DateTimeFormatter::of_pattern("HH:mm z").unwrap();
        assert_eq!(LocalTime::of(1, 2, 0, 0).unwrap().format(&f).unwrap_err().message(), "Unable to extract ZoneId from temporal 01:02");
    }

    #[test]
    fn adjacent_value_parsing() {
        let f = DateTimeFormatter::of_pattern("yyyyMMddHHmm").unwrap();
        let widths: Vec<i32> = f.items.iter().map(|i| if let Item::Number(n) = i { n.subsequent } else { 0 }).collect();
        assert_eq!(widths, [8, -1, -1, -1, -1]);
        let f = DateTimeFormatter::of_pattern("Hmm:ssSSS").unwrap();
        let widths: Vec<i32> = f.items.iter().map(|i| match i {
            Item::Number(n) => n.subsequent,
            Item::Fraction { subsequent, .. } => *subsequent,
            _ => 0,
        }).collect();
        assert_eq!(widths, [2, -1, 0, 3, -1]);
    }
}
