//! `java.time.ZoneId`, `ZoneOffset` and `ZoneRules` on the time-zone database of MockServer 8.0.0's image (Temurin
//! 26.0.2, tzdb 2026b, with the en_US zone names), read from `tzdb.bin` (written by `scripts/tz-tables.sh`); and the
//! JVM's default zone, GMT unless set ([`set_default_zone`], [`ZoneId::host`]).

use std::fmt;
use std::sync::{LazyLock, RwLock};

use crate::{Error, Result};

// ---------------------------------------------------------------------------------------------------------------
// The database

/// `ZoneOffsetTransitionRule`.
#[derive(Debug, Clone, Copy)]
struct TransitionRule {
    month: u32,
    dom: i32,
    /// ISO day of week, 0 for none.
    dow: u32,
    /// Second of day of the local time, 86400 when `midnightEndOfDay`.
    second_of_day: i64,
    end_of_day: bool,
    /// UTC 0, WALL 1, STANDARD 2.
    time_definition: u8,
    standard: i32,
    before: i32,
    after: i32,
}

/// `java.time.zone.ZoneRules`.
#[derive(Debug)]
pub(crate) struct ZoneRules {
    standard_transitions: Vec<i64>,
    standard_offsets: Vec<i32>,
    savings_transitions: Vec<i64>,
    wall_offsets: Vec<i32>,
    /// `savingsLocalTransitions`, as local seconds since 1970-01-01T00:00.
    local_transitions: Vec<i64>,
    last_rules: Vec<TransitionRule>,
}

/// A region's names in `en_US`.
#[derive(Debug, Default)]
struct Names {
    /// What `DateTimeFormatter` prints: standard long, short, daylight long, short, generic long, short.
    formatter: [Option<u16>; 6],
    /// `TimeZone.getDisplayName`: standard long, short, daylight long, short.
    time_zone: [Option<u16>; 4],
}

struct Region {
    id: u16,
    rules: u16,
    names: Names,
}

struct Database {
    strings: Vec<String>,
    rules: Vec<ZoneRules>,
    regions: Vec<Region>,
    /// `TimeZoneNameUtility.getZoneStrings` rows of regions: id, `ZoneName.toZid`, names 1 to 6.
    zone_strings: Vec<(u16, u16, [Option<u16>; 6])>,
    /// The other regions (not `Etc`, not `GMT`) and their display names 1 to 6.
    custom: Vec<(u16, [Option<u16>; 6])>,
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> &[u8] {
        let s = &self.bytes[self.pos..self.pos + n];
        self.pos += n;
        s
    }
    fn u8(&mut self) -> u8 {
        self.take(1)[0]
    }
    fn u16(&mut self) -> u16 {
        u16::from_be_bytes(self.take(2).try_into().expect("2 bytes"))
    }
    fn i32(&mut self) -> i32 {
        i32::from_be_bytes(self.take(4).try_into().expect("4 bytes"))
    }
    fn i64(&mut self) -> i64 {
        i64::from_be_bytes(self.take(8).try_into().expect("8 bytes"))
    }
    fn string(&mut self) -> String {
        let n = self.u16() as usize;
        String::from_utf8(self.take(n).to_vec()).expect("UTF-8")
    }
    fn reference(&mut self) -> Option<u16> {
        Some(self.u16()).filter(|r| *r != 0xFFFF)
    }
    fn refs<const N: usize>(&mut self) -> [Option<u16>; N] {
        std::array::from_fn(|_| self.reference())
    }
}

static DATABASE: LazyLock<Database> = LazyLock::new(|| {
    let mut r = Reader { bytes: include_bytes!("tzdb.bin"), pos: 0 };
    assert_eq!(r.take(4), b"DFTZ");
    assert_eq!(r.u8(), 1);
    let _version = r.string();
    let rules = (0..r.u16())
        .map(|_| {
            let n = r.i32() as usize;
            let standard_transitions: Vec<i64> = (0..n).map(|_| r.i64()).collect();
            let standard_offsets: Vec<i32> = (0..=n).map(|_| r.i32()).collect();
            let m = r.i32() as usize;
            let savings_transitions: Vec<i64> = (0..m).map(|_| r.i64()).collect();
            let wall_offsets: Vec<i32> = (0..=m).map(|_| r.i32()).collect();
            let last_rules = (0..r.u8())
                .map(|_| TransitionRule {
                    month: r.u8() as u32,
                    dom: r.u8() as i8 as i32,
                    dow: r.u8() as u32,
                    second_of_day: r.i32() as i64,
                    end_of_day: r.u8() == 1,
                    time_definition: r.u8(),
                    standard: r.i32(),
                    before: r.i32(),
                    after: r.i32(),
                })
                .collect();
            let mut local_transitions = Vec::with_capacity(m * 2);
            for (i, t) in savings_transitions.iter().enumerate() {
                let (before, after) = (wall_offsets[i] as i64, wall_offsets[i + 1] as i64);
                if after > before {
                    local_transitions.extend([t + before, t + after]);
                } else {
                    local_transitions.extend([t + after, t + before]);
                }
            }
            ZoneRules { standard_transitions, standard_offsets, savings_transitions, wall_offsets, local_transitions, last_rules }
        })
        .collect();
    let strings = (0..r.u16()).map(|_| r.string()).collect();
    let regions = (0..r.u16())
        .map(|_| {
            let id = r.u16();
            let rules = r.u16();
            let formatter = r.refs();
            let time_zone = r.refs();
            Region { id, rules, names: Names { formatter, time_zone } }
        })
        .collect();
    let zone_strings = (0..r.u16())
        .map(|_| {
            let id = r.u16();
            let zid = r.u16();
            (id, zid, r.refs())
        })
        .collect();
    let custom = (0..r.u16())
        .map(|_| {
            let id = r.u16();
            (id, r.refs())
        })
        .collect();
    Database { strings, rules, regions, zone_strings, custom }
});

impl Database {
    fn string(&self, r: Option<u16>) -> Option<&str> {
        r.map(|i| self.strings[i as usize].as_str())
    }

    fn region(&self, id: &str) -> Option<usize> {
        self.regions.binary_search_by(|r| self.strings[r.id as usize].as_str().cmp(id)).ok()
    }
}

/// A transition: its instant and the offsets before and after.
#[derive(Debug, Clone, Copy)]
struct Transition {
    epoch_second: i64,
    before: i32,
    after: i32,
}

impl Transition {
    fn is_gap(&self) -> bool {
        self.after > self.before
    }
    fn local_before(&self) -> i64 {
        self.epoch_second + self.before as i64
    }
    fn local_after(&self) -> i64 {
        self.epoch_second + self.after as i64
    }
}

/// The offsets a local date-time can have in a zone ([`ZoneId::local_offsets`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalOffsets {
    /// One valid offset.
    Single(i32),
    /// The local time is skipped (no valid offset): the offsets before and after the transition.
    Gap {
        /// The offset before the transition, in seconds.
        before: i32,
        /// The offset after the transition, in seconds.
        after: i32,
    },
    /// The local time happens twice: the earlier offset (before) and the later one (after).
    Overlap {
        /// The offset before the transition, in seconds.
        before: i32,
        /// The offset after the transition, in seconds.
        after: i32,
    },
}

/// What a local date-time maps to: one offset, or a transition (a gap or an overlap).
enum OffsetInfo {
    Offset(i32),
    Transition(Transition),
}

/// `ZoneInfoFile.UTC1900`: 1900-01-01T00:00:00Z.
const UTC1900: i64 = -2_208_988_800;

const DAYS_0000_TO_1970: i64 = 146_097 * 5 - (30 * 365 + 7);

impl TransitionRule {
    /// `createTransition(year)`.
    fn create(&self, year: i32) -> Transition {
        let length = super::month_length(year as i64, self.month) as i64;
        let dow = self.dow as i64;
        let mut date = if self.dom < 0 {
            let d = super::days_from_civil(year as i64, self.month as i64, length + 1 + self.dom as i64);
            let cal = super::day_of_week(d) as i64;
            // `previousOrSame(dow)`
            if dow == 0 || cal == dow {
                d
            } else {
                let diff = dow - cal;
                d - if diff >= 0 { 7 - diff } else { -diff }
            }
        } else {
            let d = super::days_from_civil(year as i64, self.month as i64, self.dom as i64);
            let cal = super::day_of_week(d) as i64;
            // `nextOrSame(dow)`
            if dow == 0 || cal == dow {
                d
            } else {
                let diff = cal - dow;
                d + if diff >= 0 { 7 - diff } else { -diff }
            }
        };
        let mut second = self.second_of_day;
        if self.end_of_day {
            date += 1;
            second = 0;
        }
        let mut local = date * 86_400 + second;
        local += match self.time_definition {
            0 => self.before as i64,
            2 => (self.before - self.standard) as i64,
            _ => 0,
        };
        Transition { epoch_second: local - self.before as i64, before: self.before, after: self.after }
    }
}

impl ZoneRules {
    fn transitions_of_year(&self, year: i32) -> Vec<Transition> {
        self.last_rules.iter().map(|r| r.create(year)).collect()
    }

    /// `getOffset(instant)`.
    fn offset(&self, epoch_second: i64) -> i32 {
        if self.savings_transitions.is_empty() {
            return self.wall_offsets[0];
        }
        let last = *self.savings_transitions.last().expect("a transition");
        if !self.last_rules.is_empty() && epoch_second > last {
            let year = find_year(epoch_second, *self.wall_offsets.last().expect("an offset"));
            let mut after = 0;
            for t in self.transitions_of_year(year) {
                if epoch_second < t.epoch_second {
                    return t.before;
                }
                after = t.after;
            }
            return after;
        }
        let index = match self.savings_transitions.binary_search(&epoch_second) {
            Ok(i) => i as i64,
            Err(i) => i as i64 - 1,
        };
        self.wall_offsets[(index + 1) as usize]
    }

    /// `getStandardOffset(instant)`.
    fn standard_offset(&self, epoch_second: i64) -> i32 {
        if self.standard_transitions.is_empty() {
            return self.standard_offsets[0];
        }
        let index = match self.standard_transitions.binary_search(&epoch_second) {
            Ok(i) => i as i64,
            Err(i) => i as i64 - 1,
        };
        self.standard_offsets[(index + 1) as usize]
    }

    /// `getOffsetInfo(localDateTime)`, the local date-time as seconds since 1970-01-01T00:00.
    fn offset_info(&self, local: i64) -> OffsetInfo {
        if self.local_transitions.is_empty() {
            return OffsetInfo::Offset(self.wall_offsets[0]);
        }
        let last = *self.local_transitions.last().expect("a transition");
        if !self.last_rules.is_empty() && local > last {
            let year = super::civil_from_days(local.div_euclid(86_400)).0 as i32;
            let mut info = OffsetInfo::Offset(0);
            for t in self.transitions_of_year(year) {
                info = find_offset_info(local, t);
                match info {
                    OffsetInfo::Transition(_) => return info,
                    OffsetInfo::Offset(o) if o == t.before => return info,
                    _ => {}
                }
            }
            return info;
        }
        let mut index = match self.local_transitions.binary_search(&local) {
            Ok(i) => i as i64,
            Err(0) => return OffsetInfo::Offset(self.wall_offsets[0]),
            Err(i) => i as i64 - 1,
        };
        let lt = &self.local_transitions;
        if lt.binary_search(&local).is_ok() && (index as usize) < lt.len() - 1 && lt[index as usize] == lt[index as usize + 1] {
            index += 1;
        }
        if index & 1 == 0 {
            let i = index as usize;
            let (before, after) = (self.wall_offsets[i / 2], self.wall_offsets[i / 2 + 1]);
            let local_transition = if after > before { lt[i] } else { lt[i + 1] };
            OffsetInfo::Transition(Transition { epoch_second: local_transition - before as i64, before, after })
        } else {
            OffsetInfo::Offset(self.wall_offsets[index as usize / 2 + 1])
        }
    }
}

/// `ZoneRules.findOffsetInfo(dt, trans)`.
fn find_offset_info(local: i64, t: Transition) -> OffsetInfo {
    if t.is_gap() {
        if local < t.local_before() {
            OffsetInfo::Offset(t.before)
        } else if local < t.local_after() {
            OffsetInfo::Transition(t)
        } else {
            OffsetInfo::Offset(t.after)
        }
    } else if local >= t.local_before() {
        OffsetInfo::Offset(t.after)
    } else if local < t.local_after() {
        OffsetInfo::Offset(t.before)
    } else {
        OffsetInfo::Transition(t)
    }
}

/// `ZoneRules.findYear(epochSecond, offset)`.
fn find_year(epoch_second: i64, offset: i32) -> i32 {
    const DAYS_PER_CYCLE: i64 = 146_097;
    let local = epoch_second + offset as i64;
    let mut zero_day = local.div_euclid(86_400) + DAYS_0000_TO_1970 - 60;
    let mut adjust = 0;
    if zero_day < 0 {
        let cycles = (zero_day + 1) / DAYS_PER_CYCLE - 1;
        adjust = cycles * 400;
        zero_day += -cycles * DAYS_PER_CYCLE;
    }
    let mut year = (400 * zero_day + 591) / DAYS_PER_CYCLE;
    let mut doy = zero_day - (365 * year + year / 4 - year / 100 + year / 400);
    if doy < 0 {
        year -= 1;
        doy = zero_day - (365 * year + year / 4 - year / 100 + year / 400);
    }
    year += adjust;
    if doy >= 306 {
        year += 1;
    }
    year.min(999_999_999) as i32
}

// ---------------------------------------------------------------------------------------------------------------
// ZoneId

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Kind {
    /// A `ZoneOffset`, or a region with a fixed offset (`UTC`, `GMT+01:00`).
    Fixed(i32),
    /// A region of the database, by index.
    Region(usize),
}

/// A zone: a `ZoneOffset` (its id `Z` or `+01:00`), or a `ZoneRegion` (`Europe/Paris`, `UTC`, `UTC+01:00`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ZoneId {
    id: String,
    kind: Kind,
    region: bool,
}

/// `ZoneOffset.MAX_SECONDS`.
const MAX_OFFSET_SECONDS: i32 = 18 * 3600;

fn date_time(message: impl Into<String>) -> Error {
    Error::new("DateTimeException", message)
}

impl ZoneId {
    /// `ZoneId.of("UTC")`.
    pub fn utc() -> ZoneId {
        ZoneId { id: "UTC".into(), kind: Kind::Fixed(0), region: true }
    }

    /// `ZoneOffset.ofTotalSeconds(seconds)`.
    pub fn of_offset_seconds(seconds: i32) -> Result<ZoneId> {
        if !(-MAX_OFFSET_SECONDS..=MAX_OFFSET_SECONDS).contains(&seconds) {
            return Err(date_time("Zone offset not in valid range: -18:00 to +18:00"));
        }
        Ok(ZoneId { id: offset_id(seconds), kind: Kind::Fixed(seconds), region: false })
    }

    /// `ZoneId.ofOffset(prefix, offset)` for `GMT`, `UTC` and `UT`: a region named by the prefix, with the offset's
    /// id appended unless it is zero.
    pub(crate) fn prefixed(prefix: &str, seconds: i32) -> ZoneId {
        if prefix.is_empty() {
            return ZoneId { id: offset_id(seconds), kind: Kind::Fixed(seconds), region: false };
        }
        let id = if seconds == 0 { prefix.to_string() } else { format!("{prefix}{}", offset_id(seconds)) };
        ZoneId { id, kind: Kind::Fixed(seconds), region: true }
    }

    /// A region of the database by its exact id (`GMT0`).
    pub(crate) fn fixed_region(id: &str) -> ZoneId {
        ZoneId::region(id).unwrap_or_else(|| ZoneId { id: id.to_string(), kind: Kind::Fixed(0), region: true })
    }

    fn region(id: &str) -> Option<ZoneId> {
        DATABASE.region(id).map(|i| ZoneId { id: id.to_string(), kind: Kind::Region(i), region: true })
    }

    /// `ZoneId.of(id)`: `Z`, an offset (`+01`, `-05:30`, `+01:02:03`), `UTC`/`GMT`/`UT` with or without an offset,
    /// or a region id of the database.
    pub fn of(id: &str) -> Result<ZoneId> {
        if id.len() <= 1 || id.starts_with('+') || id.starts_with('-') {
            return ZoneId::offset_of(id);
        }
        for prefix in ["UTC", "GMT", "UT"] {
            if let Some(rest) = id.strip_prefix(prefix) {
                if rest.is_empty() {
                    return Ok(ZoneId::prefixed(prefix, 0));
                }
                if rest.starts_with('+') || rest.starts_with('-') {
                    let offset = ZoneId::offset_of(rest).map_err(|e| {
                        date_time(format!("Invalid ID for offset-based ZoneId: {id}")).with_cause(e)
                    })?;
                    let seconds = offset.fixed_offset().unwrap_or(0);
                    return Ok(ZoneId::prefixed(prefix, seconds));
                }
                break;
            }
        }
        let bytes = id.as_bytes();
        let valid = bytes.len() >= 2
            && bytes[0].is_ascii_alphabetic()
            && bytes[1..].iter().all(|c| c.is_ascii_alphanumeric() || b"~/._+-".contains(c));
        if !valid {
            return Err(date_time(format!("Invalid ID for region-based ZoneId, invalid format: {id}")));
        }
        ZoneId::region(id).ok_or_else(|| Error::new("ZoneRulesException", format!("Unknown time-zone ID: {id}")))
    }

    /// `ZoneOffset.of(offsetId)`.
    fn offset_of(id: &str) -> Result<ZoneId> {
        if id == "Z" {
            return ZoneId::of_offset_seconds(0);
        }
        let invalid = || date_time(format!("Invalid ID for ZoneOffset, invalid format: {id}"));
        let b = id.as_bytes();
        let num = |i: usize| -> Result<i32> {
            let (a, c) = (*b.get(i).ok_or_else(invalid)?, *b.get(i + 1).ok_or_else(invalid)?);
            if !a.is_ascii_digit() || !c.is_ascii_digit() {
                return Err(invalid());
            }
            Ok(((a - b'0') * 10 + (c - b'0')) as i32)
        };
        let colon = |i: usize| -> Result<()> { if b.get(i) == Some(&b':') { Ok(()) } else { Err(invalid()) } };
        let (h, m, s) = match b.len() {
            2 if b[1].is_ascii_digit() => ((b[1] - b'0') as i32, 0, 0),
            3 => (num(1)?, 0, 0),
            5 => (num(1)?, num(3)?, 0),
            6 => {
                colon(3)?;
                (num(1)?, num(4)?, 0)
            }
            7 => (num(1)?, num(3)?, num(5)?),
            9 => {
                colon(3)?;
                colon(6)?;
                (num(1)?, num(4)?, num(7)?)
            }
            _ => return Err(invalid()),
        };
        let sign = match b[0] {
            b'+' => 1,
            b'-' => -1,
            _ => return Err(date_time(format!("Invalid ID for ZoneOffset, plus/minus not found when expected: {id}"))),
        };
        if h > 18 {
            return Err(date_time(format!("Zone offset hours not in valid range: value {} is not in the range -18 to 18", sign * h)));
        }
        if m > 59 {
            return Err(date_time(format!("Zone offset minutes not in valid range: value {} is not in the range -59 to 59", sign * m)));
        }
        if s > 59 {
            return Err(date_time(format!("Zone offset seconds not in valid range: value {} is not in the range -59 to 59", sign * s)));
        }
        if h == 18 && (m > 0 || s > 0) {
            return Err(date_time("Zone offset not in valid range: -18:00 to +18:00"));
        }
        ZoneId::of_offset_seconds(sign * (h * 3600 + m * 60 + s))
    }

    /// The zone the JVM would take as its default on this host: the `TZ` environment variable (without a leading
    /// `:`), otherwise the zone `/etc/localtime` links to; `GMT` when neither names a zone the database knows.
    pub fn host() -> ZoneId {
        let from_tz = std::env::var("TZ").ok().filter(|v| !v.is_empty()).map(|v| v.trim_start_matches(':').to_string());
        let id = from_tz.or_else(|| {
            let link = std::fs::read_link("/etc/localtime").ok()?;
            let path = link.to_string_lossy().into_owned();
            path.find("zoneinfo/").map(|i| path[i + 9..].to_string())
        });
        id.and_then(|id| ZoneId::of(&id).ok()).unwrap_or_else(|| ZoneId::prefixed("GMT", 0))
    }

    /// `getId()`.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Whether this is a `ZoneOffset`.
    pub fn is_offset(&self) -> bool {
        !self.region
    }

    fn fixed_offset(&self) -> Option<i32> {
        match self.kind {
            Kind::Fixed(o) => Some(o),
            Kind::Region(_) => None,
        }
    }

    fn rules(&self) -> Option<&'static ZoneRules> {
        match self.kind {
            Kind::Region(i) => Some(&DATABASE.rules[DATABASE.regions[i].rules as usize]),
            Kind::Fixed(_) => None,
        }
    }

    /// The offset in seconds at an instant (`getRules().getOffset(instant)`).
    pub fn offset_at(&self, epoch_second: i64) -> i32 {
        match (self.fixed_offset(), self.rules()) {
            (Some(o), _) => o,
            (_, Some(r)) => r.offset(epoch_second),
            _ => 0,
        }
    }

    /// `getRules().getStandardOffset(instant)`.
    pub fn standard_offset_at(&self, epoch_second: i64) -> i32 {
        match (self.fixed_offset(), self.rules()) {
            (Some(o), _) => o,
            (_, Some(r)) => r.standard_offset(epoch_second),
            _ => 0,
        }
    }

    /// `getRules().isDaylightSavings(instant)`.
    pub fn is_daylight_savings(&self, epoch_second: i64) -> bool {
        self.standard_offset_at(epoch_second) != self.offset_at(epoch_second)
    }

    /// The offset `ZonedDateTime.ofLocal(localDateTime, zone, null)` chooses for a local date-time given as seconds
    /// since 1970-01-01T00:00: in a gap the one before it (the time moves forward by the gap), in an overlap the
    /// earlier one (`later` for `withLaterOffsetAtOverlap`).
    pub(crate) fn offset_for_local(&self, local_epoch_second: i64, later: bool) -> i32 {
        let Some(rules) = self.rules() else { return self.fixed_offset().unwrap_or(0) };
        match rules.offset_info(local_epoch_second) {
            OffsetInfo::Offset(o) => o,
            OffsetInfo::Transition(t) if t.is_gap() => t.before,
            OffsetInfo::Transition(t) => if later { t.after } else { t.before },
        }
    }

    /// `getRules().getValidOffsets(ldt)` and `getTransition(ldt)` for a local date-time given as seconds since
    /// 1970-01-01T00:00: one valid offset, or the transition it falls in (no valid offset in a gap, two in an
    /// overlap).
    pub fn local_offsets(&self, local_epoch_second: i64) -> LocalOffsets {
        let Some(rules) = self.rules() else { return LocalOffsets::Single(self.fixed_offset().unwrap_or(0)) };
        match rules.offset_info(local_epoch_second) {
            OffsetInfo::Offset(o) => LocalOffsets::Single(o),
            OffsetInfo::Transition(t) if t.is_gap() => LocalOffsets::Gap { before: t.before, after: t.after },
            OffsetInfo::Transition(t) => LocalOffsets::Overlap { before: t.before, after: t.after },
        }
    }

    /// `getRules().isFixedOffset()`: an offset, or a region whose rules never change (`UTC`, `Etc/GMT+5`).
    pub fn is_fixed_offset(&self) -> bool {
        match self.rules() {
            None => true,
            Some(r) => r.savings_transitions.is_empty() && r.last_rules.is_empty(),
        }
    }

    /// What `java.util.TimeZone` (`sun.util.calendar.ZoneInfo`, which `Date`, `Timestamp` and `SimpleDateFormat`
    /// use) says at an instant: the offset and whether it is daylight time. `ZoneInfoFile` keeps the transitions
    /// from 1900 on: before that, the zone's last standard offset applies, without daylight time.
    pub(crate) fn util_offset_at(&self, epoch_second: i64) -> (i32, bool) {
        let Some(rules) = self.rules() else { return (self.fixed_offset().unwrap_or(0), false) };
        if epoch_second < UTC1900 && !rules.savings_transitions.is_empty() {
            return (*rules.standard_offsets.last().expect("an offset"), false);
        }
        let offset = rules.offset(epoch_second);
        (offset, offset != rules.standard_offset(epoch_second))
    }

    /// `ZoneInfo.getOffsetsByWall(wall)`, the wall time as seconds since 1970-01-01T00:00: a transition counts from
    /// its local time after it, so a time in a gap takes the offset before it and one in an overlap the offset after
    /// it.
    pub(crate) fn util_offset_by_wall(&self, local: i64) -> i32 {
        let Some(rules) = self.rules() else { return self.fixed_offset().unwrap_or(0) };
        let offset = match rules.offset_info(local) {
            OffsetInfo::Offset(o) => o,
            OffsetInfo::Transition(t) if t.is_gap() => t.before,
            OffsetInfo::Transition(t) => t.after,
        };
        if local - (offset as i64) < UTC1900 && !rules.savings_transitions.is_empty() {
            return *rules.standard_offsets.last().expect("an offset");
        }
        offset
    }

    /// Whether a local date-time is in a gap or an overlap (`getRules().getTransition(ldt) != null`).
    pub(crate) fn has_transition_at_local(&self, local_epoch_second: i64) -> bool {
        self.rules().is_some_and(|r| matches!(r.offset_info(local_epoch_second), OffsetInfo::Transition(_)))
    }

    /// The name `DateTimeFormatter` prints for this region: `kind` 0 standard, 1 daylight, 2 generic; `full` for
    /// `zzzz`/`vvvv`. `None` when the JDK has none (the id is printed).
    pub(crate) fn formatter_name(&self, kind: usize, full: bool) -> Option<&'static str> {
        let i = self.names_index()?;
        let names = &DATABASE.regions[i].names.formatter;
        let pick = |k: usize| names[k * 2 + usize::from(!full)];
        if names[..4].iter().all(Option::is_none) {
            return None;
        }
        match kind {
            2 => Some(DATABASE.string(pick(2)).unwrap_or(&DATABASE.strings[DATABASE.regions[i].id as usize])),
            k => DATABASE.string(pick(k)),
        }
    }

    /// The database entry whose names this zone has: its region, or the region of the same id (`UTC`, `GMT`).
    fn names_index(&self) -> Option<usize> {
        match self.kind {
            Kind::Region(i) => Some(i),
            Kind::Fixed(_) if self.region => DATABASE.region(&self.id),
            Kind::Fixed(_) => None,
        }
    }

    /// `TimeZone.getDisplayName(daylight, style, Locale.US)`; a fixed zone prints as `GMT+01:00` (`UTC` as `UTC`).
    pub(crate) fn time_zone_name(&self, daylight: bool, long: bool) -> String {
        if let Some(i) = self.names_index() {
            let names = &DATABASE.regions[i].names.time_zone;
            let name = names[usize::from(daylight) * 2 + usize::from(!long)];
            return DATABASE.string(name).unwrap_or(&self.id).to_string();
        }
        match self.kind {
            Kind::Region(_) => self.id.clone(),
            Kind::Fixed(o) => {
                if o == 0 && long {
                    return "Greenwich Mean Time".into();
                }
                if o == 0 {
                    return "GMT".into();
                }
                let abs = o.unsigned_abs() / 60;
                format!("GMT{}{:02}:{:02}", if o < 0 { '-' } else { '+' }, abs / 60, abs % 60)
            }
        }
    }
}

/// `ZoneOffset.getId()`: `Z`, or `+hh:mm` with `:ss` when the seconds are not zero.
pub(crate) fn offset_id(seconds: i32) -> String {
    if seconds == 0 {
        return "Z".into();
    }
    let abs = seconds.unsigned_abs();
    let (h, m, s) = (abs / 3600, abs / 60 % 60, abs % 60);
    let mut id = format!("{}{h:02}:{m:02}", if seconds < 0 { '-' } else { '+' });
    if s != 0 {
        id.push_str(&format!(":{s:02}"));
    }
    id
}

impl fmt::Display for ZoneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.id)
    }
}

// ---------------------------------------------------------------------------------------------------------------
// The default zone

static DEFAULT_ZONE: LazyLock<RwLock<ZoneId>> = LazyLock::new(|| RwLock::new(ZoneId::prefixed("GMT", 0)));

thread_local! {
    static THREAD_ZONE: std::cell::RefCell<Option<ZoneId>> = const { std::cell::RefCell::new(None) };
}

/// The JVM's default zone (`ZoneId.systemDefault()`, `TimeZone.getDefault()`): `GMT`, as in MockServer 8.0.0's image
/// (its JVM runs without `-Duser.timezone` or `TZ` and takes `GMT`), unless
/// [`set_default_zone`] set another, or [`with_default_zone`] on this thread.
pub fn default_zone() -> ZoneId {
    if let Some(z) = THREAD_ZONE.with(|z| z.borrow().clone()) {
        return z;
    }
    DEFAULT_ZONE.read().expect("default zone").clone()
}

/// Sets the process's default zone (as `-Duser.timezone` or `TimeZone.setDefault` do), e.g. to [`ZoneId::host`].
pub fn set_default_zone(zone: ZoneId) {
    *DEFAULT_ZONE.write().expect("default zone") = zone;
}

/// Runs `f` with the default zone of this thread set to `zone`; other threads keep theirs.
pub fn with_default_zone<T>(zone: ZoneId, f: impl FnOnce() -> T) -> T {
    struct Restore(Option<ZoneId>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let old = self.0.take();
            THREAD_ZONE.with(|z| *z.borrow_mut() = old);
        }
    }
    let _restore = Restore(THREAD_ZONE.with(|z| z.borrow_mut().replace(zone)));
    f()
}

// ---------------------------------------------------------------------------------------------------------------
// DateTimeFormatterBuilder.PrefixTree (case sensitive)

/// A node of `PrefixTree`: its key (UTF-16), value (a zone id and a name type) and children.
#[derive(Debug, Default)]
pub(crate) struct PrefixTree {
    key: Vec<u16>,
    value: Option<(String, i32)>,
    children: Vec<PrefixTree>,
}

impl PrefixTree {
    fn add(&mut self, k: &[u16], v: &str, t: i32) {
        let prefix = k.iter().zip(&self.key).take_while(|(a, b)| a == b).count();
        if prefix == self.key.len() {
            if prefix < k.len() {
                let sub = &k[prefix..];
                if let Some(c) = self.children.iter_mut().find(|c| c.key.first() == sub.first()) {
                    c.add(sub, v, t);
                    return;
                }
                self.children.insert(0, PrefixTree { key: sub.to_vec(), value: Some((v.to_string(), t)), children: Vec::new() });
                return;
            }
            self.value = Some((v.to_string(), t));
            return;
        }
        let n1 = PrefixTree { key: self.key[prefix..].to_vec(), value: self.value.take(), children: std::mem::take(&mut self.children) };
        self.key.truncate(prefix);
        self.children = vec![n1];
        if prefix < k.len() {
            self.children.push(PrefixTree { key: k[prefix..].to_vec(), value: Some((v.to_string(), t)), children: Vec::new() });
        } else {
            self.value = Some((v.to_string(), t));
        }
    }

    /// `match(text, pos)`: the node the text leads to and the position after it, or `None` when the text does not
    /// start with this node's key.
    fn find(&self, text: &[u16], off: usize) -> Option<(&PrefixTree, usize)> {
        if !text[off..].starts_with(&self.key) {
            return None;
        }
        let off = off + self.key.len();
        if off != text.len()
            && let Some(c) = self.children.iter().find(|c| c.key.first() == Some(&text[off]))
            && let Some(found) = c.find(text, off)
        {
            return Some(found);
        }
        Some((self, off))
    }

    /// The zone id and name type the tree matches at `pos`, and the position after the match.
    pub(crate) fn matches(&self, text: &[u16], pos: usize) -> Option<(&str, i32, usize)> {
        let (node, end) = self.find(text, pos)?;
        node.value.as_ref().map(|(v, t)| (v.as_str(), *t, end))
    }
}

fn units(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// `ZoneIdPrinterParser`'s tree of the region ids.
pub(crate) static REGION_TREE: LazyLock<PrefixTree> = LazyLock::new(|| {
    let mut tree = PrefixTree::default();
    for r in &DATABASE.regions {
        let id = &DATABASE.strings[r.id as usize];
        tree.add(&units(id), id, -1);
    }
    tree
});

/// `ZoneTextPrinterParser.getTree` for `en_US`: `full` for `zzzz`/`vvvv`.
fn zone_text_tree(full: bool) -> PrefixTree {
    let db = &*DATABASE;
    let mut tree = PrefixTree::default();
    for (id, zid, names) in &db.zone_strings {
        let id = &db.strings[*id as usize];
        tree.add(&units(id), id, -1);
        let zid = &db.strings[*zid as usize];
        let mut i = if full { 1 } else { 2 };
        while i <= 6 {
            if let Some(name) = db.string(names[i - 1]) {
                tree.add(&units(name), zid, ((i - 1) / 2) as i32);
            }
            i += 2;
        }
    }
    for (cid, names) in &db.custom {
        let cid = &db.strings[*cid as usize];
        let mut i = if full { 1 } else { 2 };
        while i <= 6 {
            if let Some(name) = db.string(names[i - 1]).filter(|n| !n.is_empty()) {
                tree.add(&units(name), cid, ((i - 1) / 2) as i32);
            }
            i += 2;
        }
    }
    tree
}

pub(crate) static ZONE_TEXT_TREE_FULL: LazyLock<PrefixTree> = LazyLock::new(|| zone_text_tree(true));
pub(crate) static ZONE_TEXT_TREE_SHORT: LazyLock<PrefixTree> = LazyLock::new(|| zone_text_tree(false));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules() {
        let ny = ZoneId::of("America/New_York").unwrap();
        // 2023-07-01T00:00:00Z and 2023-01-01T00:00:00Z
        assert_eq!(ny.offset_at(1_688_169_600), -4 * 3600);
        assert_eq!(ny.offset_at(1_672_531_200), -5 * 3600);
        assert!(ny.is_daylight_savings(1_688_169_600));
        // 2100-07-01, through the last rules
        assert_eq!(ny.offset_at(4_118_083_200), -4 * 3600);
        // 2023-03-12T02:30 is in the gap: the offset before it; 2023-11-05T01:30 overlaps: the earlier one
        assert_eq!(ny.offset_for_local(1_678_588_200, false), -5 * 3600);
        assert_eq!(ny.offset_for_local(1_699_147_800, false), -4 * 3600);
        assert_eq!(ny.offset_for_local(1_699_147_800, true), -5 * 3600);
        assert_eq!(ny.formatter_name(0, false), Some("EST"));
        assert_eq!(ny.formatter_name(2, true), Some("Eastern Time"));
        assert_eq!(ny.time_zone_name(true, true), "Eastern Daylight Time");
        assert_eq!(ZoneId::of("Europe/Nowhere").unwrap_err().message(), "Unknown time-zone ID: Europe/Nowhere");
        assert_eq!(ZoneId::of("UTC+01:30").unwrap().id(), "UTC+01:30");
        assert_eq!(ZoneId::of("+5").unwrap().id(), "+05:00");
        assert_eq!(REGION_TREE.matches(&units("Europe/Parisx"), 0).map(|m| (m.0, m.2)), Some(("Europe/Paris", 12)));
        let full = ZONE_TEXT_TREE_FULL.matches(&units("Eastern Standard Time"), 0).map(|m| (m.0.to_string(), m.1));
        assert_eq!(full, Some(("America/New_York".to_string(), 0)));
    }
}
