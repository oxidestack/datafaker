//! The part of `java.time`, `java.util.Date`, `java.sql.Timestamp` and `java.util.concurrent.TimeUnit` that
//! Datafaker's date providers use: values, arithmetic, `toString()` forms, `DateTimeFormatter.ofPattern` and
//! `SimpleDateFormat`.
//!
//! The JVM's default zone (`ZoneId.systemDefault()`, `TimeZone.getDefault()`) is `GMT` here, as in MockServer
//! 8.0.0's image, unless [`set_default_zone`] (or [`with_default_zone`] on a thread) sets another one of the JDK's
//! time-zone database (`zone.rs`); the default FORMAT locale is `en_US`.

pub mod formatter;
mod parse;
pub mod simple_date_format;
pub mod zone;

use std::fmt;

use super::Duration;
use crate::{Error, Result};

pub use formatter::DateTimeFormatter;
pub use parse::Parsed;
pub use simple_date_format::SimpleDateFormat;
pub use zone::{LocalOffsets, ZoneId, default_zone, set_default_zone, with_default_zone};

const NANOS_PER_SECOND: i64 = 1_000_000_000;
const NANOS_PER_DAY: i64 = 86_400 * NANOS_PER_SECOND;
const SECONDS_PER_DAY: i64 = 86_400;

fn long_overflow() -> Error {
    Error::new("ArithmeticException", "long overflow")
}

fn date_time(message: impl Into<String>) -> Error {
    Error::new("DateTimeException", message)
}

fn unsupported(message: impl Into<String>) -> Error {
    Error::new("UnsupportedTemporalTypeException", message)
}

/// `Math.addExact(long, long)`.
fn add_exact(a: i64, b: i64) -> Result<i64> {
    a.checked_add(b).ok_or_else(long_overflow)
}

/// `Math.multiplyExact(long, long)`.
fn multiply_exact(a: i64, b: i64) -> Result<i64> {
    a.checked_mul(b).ok_or_else(long_overflow)
}

thread_local! {
    /// The fixed clock of [`with_fixed_clock`] on this thread, if any.
    static FIXED_CLOCK: std::cell::Cell<Option<(i64, i32)>> = const { std::cell::Cell::new(None) };
}

/// Runs `f` with the clock of this thread fixed at `instant` (as `Clock.fixed(instant, UTC)`): `Instant.now()`,
/// `LocalDate.now()` and the rest read `instant` until `f` returns. For tests and the replay of vectors recorded on
/// a given date; other threads keep the system clock.
pub fn with_fixed_clock<T>(instant: Instant, f: impl FnOnce() -> T) -> T {
    struct Restore(Option<(i64, i32)>);
    impl Drop for Restore {
        fn drop(&mut self) {
            FIXED_CLOCK.with(|c| c.set(self.0));
        }
    }
    let _restore = Restore(FIXED_CLOCK.with(|c| c.replace(Some((instant.seconds, instant.nanos)))));
    f()
}

/// The current time from the system clock, at the microsecond resolution the JDK reads on macOS and Linux.
fn system_now() -> (i64, i32) {
    if let Some(fixed) = FIXED_CLOCK.with(|c| c.get()) {
        return fixed;
    }
    let now = std::time::SystemTime::now();
    let (seconds, nanos) = match now.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => (d.as_secs() as i64, d.subsec_nanos() as i64),
        Err(e) => {
            let d = e.duration();
            let total = -(d.as_nanos() as i128);
            (total.div_euclid(NANOS_PER_SECOND as i128) as i64, total.rem_euclid(NANOS_PER_SECOND as i128) as i64)
        }
    };
    (seconds, (nanos / 1000 * 1000) as i32)
}

/// `System.currentTimeMillis()`.
pub fn current_time_millis() -> i64 {
    let (seconds, nanos) = system_now();
    seconds * 1000 + nanos as i64 / 1_000_000
}

// ---------------------------------------------------------------------------------------------------------------
// Units

/// `java.util.concurrent.TimeUnit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeUnit {
    /// `NANOSECONDS`.
    Nanoseconds,
    /// `MICROSECONDS`.
    Microseconds,
    /// `MILLISECONDS`.
    Milliseconds,
    /// `SECONDS`.
    Seconds,
    /// `MINUTES`.
    Minutes,
    /// `HOURS`.
    Hours,
    /// `DAYS`.
    Days,
}

impl TimeUnit {
    /// The constants' names, in declaration order.
    pub const NAMES: &'static [&'static str] = &["NANOSECONDS", "MICROSECONDS", "MILLISECONDS", "SECONDS", "MINUTES", "HOURS", "DAYS"];
    const ALL: [TimeUnit; 7] = [
        TimeUnit::Nanoseconds,
        TimeUnit::Microseconds,
        TimeUnit::Milliseconds,
        TimeUnit::Seconds,
        TimeUnit::Minutes,
        TimeUnit::Hours,
        TimeUnit::Days,
    ];

    /// `TimeUnit.valueOf(name)`.
    pub fn value_of(name: &str) -> Option<TimeUnit> {
        TimeUnit::NAMES.iter().position(|n| *n == name).map(|i| TimeUnit::ALL[i])
    }

    /// `name()`.
    pub fn name(self) -> &'static str {
        TimeUnit::NAMES[self as usize]
    }

    /// `toMillis(duration)`, saturating at `Long.MIN_VALUE`/`Long.MAX_VALUE`.
    pub fn to_millis(self, duration: i64) -> i64 {
        match self {
            TimeUnit::Nanoseconds => duration / 1_000_000,
            TimeUnit::Microseconds => duration / 1000,
            TimeUnit::Milliseconds => duration,
            other => {
                let ratio = match other {
                    TimeUnit::Seconds => 1000,
                    TimeUnit::Minutes => 60_000,
                    TimeUnit::Hours => 3_600_000,
                    _ => 86_400_000,
                };
                let max = i64::MAX / ratio;
                if duration > max {
                    i64::MAX
                } else if duration < -max {
                    i64::MIN
                } else {
                    duration * ratio
                }
            }
        }
    }

    /// `toChronoUnit()`.
    pub fn to_chrono_unit(self) -> ChronoUnit {
        match self {
            TimeUnit::Nanoseconds => ChronoUnit::Nanos,
            TimeUnit::Microseconds => ChronoUnit::Micros,
            TimeUnit::Milliseconds => ChronoUnit::Millis,
            TimeUnit::Seconds => ChronoUnit::Seconds,
            TimeUnit::Minutes => ChronoUnit::Minutes,
            TimeUnit::Hours => ChronoUnit::Hours,
            TimeUnit::Days => ChronoUnit::Days,
        }
    }
}

/// `java.time.temporal.ChronoUnit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChronoUnit {
    /// `NANOS`.
    Nanos,
    /// `MICROS`.
    Micros,
    /// `MILLIS`.
    Millis,
    /// `SECONDS`.
    Seconds,
    /// `MINUTES`.
    Minutes,
    /// `HOURS`.
    Hours,
    /// `HALF_DAYS`.
    HalfDays,
    /// `DAYS`.
    Days,
    /// `WEEKS`.
    Weeks,
    /// `MONTHS`.
    Months,
    /// `YEARS`.
    Years,
    /// `DECADES`.
    Decades,
    /// `CENTURIES`.
    Centuries,
    /// `MILLENNIA`.
    Millennia,
    /// `ERAS`.
    Eras,
    /// `FOREVER`.
    Forever,
}

impl ChronoUnit {
    /// The constants' names, in declaration order.
    pub const NAMES: &'static [&'static str] = &[
        "NANOS", "MICROS", "MILLIS", "SECONDS", "MINUTES", "HOURS", "HALF_DAYS", "DAYS", "WEEKS", "MONTHS", "YEARS", "DECADES",
        "CENTURIES", "MILLENNIA", "ERAS", "FOREVER",
    ];
    const ALL: [ChronoUnit; 16] = [
        ChronoUnit::Nanos,
        ChronoUnit::Micros,
        ChronoUnit::Millis,
        ChronoUnit::Seconds,
        ChronoUnit::Minutes,
        ChronoUnit::Hours,
        ChronoUnit::HalfDays,
        ChronoUnit::Days,
        ChronoUnit::Weeks,
        ChronoUnit::Months,
        ChronoUnit::Years,
        ChronoUnit::Decades,
        ChronoUnit::Centuries,
        ChronoUnit::Millennia,
        ChronoUnit::Eras,
        ChronoUnit::Forever,
    ];
    const DISPLAY: [&'static str; 16] = [
        "Nanos", "Micros", "Millis", "Seconds", "Minutes", "Hours", "HalfDays", "Days", "Weeks", "Months", "Years", "Decades",
        "Centuries", "Millennia", "Eras", "Forever",
    ];

    /// `ChronoUnit.valueOf(name)`.
    pub fn value_of(name: &str) -> Option<ChronoUnit> {
        ChronoUnit::NAMES.iter().position(|n| *n == name).map(|i| ChronoUnit::ALL[i])
    }

    /// `name()`.
    pub fn name(self) -> &'static str {
        ChronoUnit::NAMES[self as usize]
    }

    /// `isDurationEstimated()`: days and longer.
    pub fn is_duration_estimated(self) -> bool {
        self >= ChronoUnit::Days
    }

    /// `getDuration().getSeconds()` for the units up to half days.
    fn seconds(self) -> i64 {
        match self {
            ChronoUnit::Minutes => 60,
            ChronoUnit::Hours => 3600,
            ChronoUnit::HalfDays => 43_200,
            _ => SECONDS_PER_DAY,
        }
    }
}

impl fmt::Display for ChronoUnit {
    /// `toString()`: `Nanos`, `HalfDays`, …
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(ChronoUnit::DISPLAY[*self as usize])
    }
}

impl Duration {
    /// `Duration.of(amount, unit)`.
    pub fn of(amount: i64, unit: ChronoUnit) -> Result<Duration> {
        if unit == ChronoUnit::Days {
            return Ok(Duration::of_seconds(multiply_exact(amount, SECONDS_PER_DAY)?));
        }
        if unit.is_duration_estimated() {
            return Err(unsupported("Unit must not have an estimated duration"));
        }
        if amount == 0 {
            return Ok(Duration::ZERO);
        }
        match unit {
            ChronoUnit::Nanos => Duration::of_seconds_and_nanos(0, amount),
            ChronoUnit::Micros => {
                let seconds = (amount / 1_000_000_000) * 1000;
                Duration::of_seconds_and_nanos(seconds, (amount % 1_000_000_000) * 1000)
            }
            ChronoUnit::Millis => Duration::of_seconds_and_nanos(amount / 1000, (amount % 1000) * 1_000_000),
            ChronoUnit::Seconds => Ok(Duration::of_seconds(amount)),
            other => Ok(Duration::of_seconds(multiply_exact(other.seconds(), amount)?)),
        }
    }
}

/// `NumberPrinterParser.parse` in strict mode, from `position`: the value and the position after it, or the error
/// index. `exceeds_pad` is `SignStyle.EXCEEDS_PAD` (the year), otherwise `NOT_NEGATIVE` with a fixed width.
fn iso_value(text: &[u16], position: usize, min_width: usize, max_width: usize, exceeds_pad: bool) -> std::result::Result<(i64, usize), usize> {
    let mut position = position;
    if position == text.len() {
        return Err(position);
    }
    let (mut negative, mut positive) = (false, false);
    match text[position] {
        0x2B | 0x2D if !exceeds_pad => return Err(position),
        0x2B => positive = true,
        0x2D => negative = true,
        _ => {}
    }
    if positive || negative {
        position += 1;
    }
    let min_end = position + min_width;
    if min_end > text.len() {
        return Err(position);
    }
    let max_end = (position + max_width).min(text.len());
    let mut total: i64 = 0;
    let mut pos = position;
    while pos < max_end {
        let digit = text[pos].wrapping_sub(b'0' as u16);
        if digit > 9 {
            break;
        }
        total = total * 10 + digit as i64;
        pos += 1;
    }
    if pos < min_end {
        return Err(position);
    }
    if negative {
        if total == 0 {
            return Err(position - 1);
        }
        total = -total;
    } else if exceeds_pad {
        let len = pos - position;
        if positive && len <= min_width {
            return Err(position - 1);
        }
        if !positive && len > min_width {
            return Err(position);
        }
    }
    Ok((total, pos))
}

/// `CharLiteralPrinterParser.parse`: the position after `c`, or the error index.
fn iso_literal(text: &[u16], position: usize, c: char) -> std::result::Result<usize, usize> {
    if text.get(position) == Some(&(c as u16)) { Ok(position + 1) } else { Err(position) }
}

// ---------------------------------------------------------------------------------------------------------------
// Calendar arithmetic

/// Days since 1970-01-01 of a proleptic Gregorian date.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Days since 1970-01-01 of a proleptic Julian date (`JulianCalendar.getFixedDate`).
fn julian_epoch_day(year: i64, month: i64, day: i64) -> i64 {
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    day + (153 * m + 2) / 5 + 365 * y + y.div_euclid(4) - 32_083 - 2_440_588
}

/// The proleptic Gregorian date of a day since 1970-01-01.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(month <= 2), month, day)
}

fn is_leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn month_length(year: i64, month: u32) -> u32 {
    match month {
        2 => 28 + u32::from(is_leap(year)),
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// ISO day of week, Monday 1 to Sunday 7.
fn day_of_week(epoch_day: i64) -> u32 {
    ((epoch_day + 3).rem_euclid(7) + 1) as u32
}

const YEAR_MIN: i64 = -999_999_999;
const YEAR_MAX: i64 = 999_999_999;

fn check_year(year: i64) -> Result<i32> {
    if !(YEAR_MIN..=YEAR_MAX).contains(&year) {
        return Err(date_time(format!("Invalid value for Year (valid values {YEAR_MIN} - {YEAR_MAX}): {year}")));
    }
    Ok(year as i32)
}

/// A date of the calendar `java.util.GregorianCalendar` uses: Julian before 1582-10-15, Gregorian from then on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HybridDate {
    /// The year of era (1 for both 1 AD and 1 BC).
    pub year_of_era: i64,
    /// 1 AD, 0 BC.
    pub era: u32,
    /// The proleptic year (0 is 1 BC), for week and leap arithmetic.
    pub year: i64,
    pub month: u32,
    pub day: u32,
    pub julian: bool,
}

/// Epoch day of 1582-10-15, the first Gregorian day.
const GREGORIAN_CUTOVER_DAY: i64 = -141_427;

impl HybridDate {
    pub(crate) fn of_epoch_day(epoch_day: i64) -> HybridDate {
        let (year, month, day, julian) = if epoch_day >= GREGORIAN_CUTOVER_DAY {
            let (y, m, d) = civil_from_days(epoch_day);
            (y, m, d, false)
        } else {
            // Julian calendar from the Julian day number.
            let c = epoch_day + 2_440_588 + 32_082;
            let d = (4 * c + 3).div_euclid(1461);
            let e = c - (1461 * d).div_euclid(4);
            let m = (5 * e + 2).div_euclid(153);
            let day = (e - (153 * m + 2).div_euclid(5) + 1) as u32;
            let month = (m + 3 - 12 * (m / 10)) as u32;
            (d - 4800 + m / 10, month, day, true)
        };
        let (year_of_era, era) = if year >= 1 { (year, 1) } else { (1 - year, 0) };
        HybridDate { year_of_era, era, year, month, day, julian }
    }

    /// Epoch day of a date given in the hybrid calendar (proleptic year).
    pub(crate) fn epoch_day_of(year: i64, month: u32, day: u32) -> i64 {
        let gregorian = days_from_civil(year, month as i64, day as i64);
        if gregorian >= GREGORIAN_CUTOVER_DAY {
            return gregorian;
        }
        let a = (14 - month as i64) / 12;
        let y = year + 4800 - a;
        let m = month as i64 + 12 * a - 3;
        let jdn = day as i64 + (153 * m + 2) / 5 + 365 * y + y.div_euclid(4) - 32_083;
        jdn - 2_440_588
    }

    pub(crate) fn is_leap(&self) -> bool {
        if self.julian { self.year.rem_euclid(4) == 0 } else { is_leap(self.year) }
    }

    /// Day of year in this date's own calendar year.
    pub(crate) fn day_of_year(&self, epoch_day: i64) -> i64 {
        epoch_day - HybridDate::epoch_day_of(self.year, 1, 1) + 1
    }
}

// ---------------------------------------------------------------------------------------------------------------
// LocalDate

/// `java.time.LocalDate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalDate {
    year: i32,
    month: u32,
    day: u32,
}

impl LocalDate {
    /// `LocalDate.of(year, month, day)`.
    pub fn of(year: i32, month: u32, day: u32) -> Result<LocalDate> {
        check_year(year as i64)?;
        if !(1..=12).contains(&month) {
            return Err(date_time(format!("Invalid value for MonthOfYear (valid values 1 - 12): {month}")));
        }
        if !(1..=31).contains(&day) {
            return Err(date_time(format!("Invalid value for DayOfMonth (valid values 1 - 28/31): {day}")));
        }
        if day > month_length(year as i64, month) {
            return Err(date_time(if day == 29 {
                format!("Invalid date 'February 29' as '{year}' is not a leap year")
            } else {
                format!("Invalid date '{} {day}'", MONTHS[month as usize - 1].to_uppercase())
            }));
        }
        Ok(LocalDate { year, month, day })
    }

    /// `LocalDate.ofEpochDay(epochDay)`.
    pub fn of_epoch_day(epoch_day: i64) -> Result<LocalDate> {
        const MIN: i64 = -365_243_219_162;
        const MAX: i64 = 365_241_780_471;
        if !(MIN..=MAX).contains(&epoch_day) {
            return Err(date_time(format!("Invalid value for EpochDay (valid values {MIN} - {MAX}): {epoch_day}")));
        }
        let (year, month, day) = civil_from_days(epoch_day);
        Ok(LocalDate { year: year as i32, month, day })
    }

    /// `LocalDate.now()` in the default zone (`GMT`).
    pub fn now() -> LocalDate {
        LocalDateTime::of_instant(Instant::now()).date()
    }

    /// `LocalDate.parse(text)`: `DateTimeFormatter.ISO_LOCAL_DATE` (strict), `DateTimeParseException`s with the
    /// JDK's texts and indexes. The year is 4 digits, or a sign and up to 10 (`+` only beyond 4, never `-0000`);
    /// month and day 2 digits each; a value out of range fails resolving (`could not be parsed: Invalid …`).
    pub fn parse(text: &str) -> Result<LocalDate> {
        let units: Vec<u16> = text.encode_utf16().collect();
        let fail = |detail: String| {
            let abbr = if units.len() > 64 { format!("{}...", String::from_utf16_lossy(&units[..64])) } else { text.to_string() };
            Error::new("DateTimeParseException", format!("Text '{abbr}' could not be parsed{detail}"))
        };
        let at = |index: usize| fail(format!(" at index {index}"));
        let (year, pos) = iso_value(&units, 0, 4, 10, true).map_err(at)?;
        let pos = iso_literal(&units, pos, '-').map_err(at)?;
        let (month, pos) = iso_value(&units, pos, 2, 2, false).map_err(at)?;
        let pos = iso_literal(&units, pos, '-').map_err(at)?;
        let (day, pos) = iso_value(&units, pos, 2, 2, false).map_err(at)?;
        if pos < units.len() {
            return Err(fail(format!(", unparsed text found at index {pos}")));
        }
        // `IsoChronology.resolveYMD` (STRICT): the year, month and day ranges, then `LocalDate.of`.
        let resolved = check_year(year).and_then(|year| LocalDate::of(year, month as u32, day as u32));
        resolved.map_err(|e| fail(format!(": {}", e.message())))
    }

    /// `LocalDate.getYear()`.
    pub fn year(&self) -> i32 {
        self.year
    }

    /// `LocalDate.getMonthValue()`: 1 to 12.
    pub fn month_value(&self) -> u32 {
        self.month
    }

    /// `LocalDate.getDayOfMonth()`.
    pub fn day_of_month(&self) -> u32 {
        self.day
    }

    /// `getDayOfWeek().getValue()`: Monday 1 to Sunday 7.
    pub fn day_of_week(&self) -> u32 {
        day_of_week(self.to_epoch_day())
    }

    /// `getDayOfYear()`.
    pub fn day_of_year(&self) -> u32 {
        (self.to_epoch_day() - days_from_civil(self.year as i64, 1, 1) + 1) as u32
    }

    /// `LocalDate.isLeapYear()`.
    pub fn is_leap_year(&self) -> bool {
        is_leap(self.year as i64)
    }

    /// `LocalDate.lengthOfMonth()`.
    pub fn length_of_month(&self) -> u32 {
        month_length(self.year as i64, self.month)
    }

    /// `LocalDate.lengthOfYear()`.
    pub fn length_of_year(&self) -> u32 {
        365 + u32::from(self.is_leap_year())
    }

    /// `toEpochDay()`.
    pub fn to_epoch_day(&self) -> i64 {
        days_from_civil(self.year as i64, self.month as i64, self.day as i64)
    }

    fn resolve_previous_valid(year: i64, month: u32, day: u32) -> Result<LocalDate> {
        let year = check_year(year)?;
        Ok(LocalDate { year, month, day: day.min(month_length(year as i64, month)) })
    }

    /// `plusYears(years)`.
    pub fn plus_years(&self, years: i64) -> Result<LocalDate> {
        if years == 0 {
            return Ok(*self);
        }
        let year = (self.year as i64).checked_add(years).unwrap_or(i64::MAX);
        LocalDate::resolve_previous_valid(year, self.month, self.day)
    }

    /// `minusYears(years)`.
    pub fn minus_years(&self, years: i64) -> Result<LocalDate> {
        if years == i64::MIN { self.plus_years(i64::MAX)?.plus_years(1) } else { self.plus_years(-years) }
    }

    /// `plusMonths(months)`.
    pub fn plus_months(&self, months: i64) -> Result<LocalDate> {
        if months == 0 {
            return Ok(*self);
        }
        let count = self.year as i64 * 12 + (self.month as i64 - 1);
        let calc = count.checked_add(months).ok_or_else(long_overflow)?;
        LocalDate::resolve_previous_valid(calc.div_euclid(12), (calc.rem_euclid(12) + 1) as u32, self.day)
    }

    /// `plusDays(days)`.
    pub fn plus_days(&self, days: i64) -> Result<LocalDate> {
        if days == 0 {
            return Ok(*self);
        }
        LocalDate::of_epoch_day(add_exact(self.to_epoch_day(), days)?)
    }

    /// `minusDays(days)`.
    pub fn minus_days(&self, days: i64) -> Result<LocalDate> {
        if days == i64::MIN { self.plus_days(i64::MAX)?.plus_days(1) } else { self.plus_days(-days) }
    }

    /// `plus(period)` (`Period.addTo`).
    pub fn plus_period(&self, period: Period) -> Result<LocalDate> {
        let mut date = *self;
        if period.months == 0 {
            if period.years != 0 {
                date = date.plus_years(period.years as i64)?;
            }
        } else {
            let total = period.to_total_months();
            if total != 0 {
                date = date.plus_months(total)?;
            }
        }
        if period.days != 0 {
            date = date.plus_days(period.days as i64)?;
        }
        Ok(date)
    }

    /// `isBefore(other)`.
    pub fn is_before(&self, other: &LocalDate) -> bool {
        self < other
    }

    /// `isAfter(other)`.
    pub fn is_after(&self, other: &LocalDate) -> bool {
        self > other
    }

    /// `atTime(time)`.
    pub fn at_time(&self, time: LocalTime) -> LocalDateTime {
        LocalDateTime { date: *self, time }
    }

    /// `format(formatter)`.
    pub fn format(&self, formatter: &DateTimeFormatter) -> Result<String> {
        formatter.format(&formatter::Temporal::date(*self))
    }
}

impl fmt::Display for LocalDate {
    /// `toString()`: `2007-12-03`, `-0044-03-15`, `+10000-01-01`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let abs = self.year.unsigned_abs();
        if abs < 1000 {
            if self.year < 0 {
                f.write_str("-")?;
            }
            write!(f, "{abs:04}")?;
        } else {
            if self.year > 9999 {
                f.write_str("+")?;
            }
            write!(f, "{}", self.year)?;
        }
        write!(f, "-{:02}-{:02}", self.month, self.day)
    }
}

// ---------------------------------------------------------------------------------------------------------------
// LocalTime

/// `java.time.LocalTime`, as its nano of day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalTime {
    nano_of_day: i64,
}

impl LocalTime {
    /// `LocalTime.MIDNIGHT`.
    pub const MIDNIGHT: LocalTime = LocalTime { nano_of_day: 0 };

    /// `LocalTime.ofNanoOfDay(nanoOfDay)`.
    pub fn of_nano_of_day(nano_of_day: i64) -> Result<LocalTime> {
        if !(0..NANOS_PER_DAY).contains(&nano_of_day) {
            return Err(date_time(format!("Invalid value for NanoOfDay (valid values 0 - {}): {nano_of_day}", NANOS_PER_DAY - 1)));
        }
        Ok(LocalTime { nano_of_day })
    }

    /// `LocalTime.ofSecondOfDay(secondOfDay)`.
    pub fn of_second_of_day(second_of_day: i64) -> Result<LocalTime> {
        if !(0..SECONDS_PER_DAY).contains(&second_of_day) {
            return Err(date_time(format!("Invalid value for SecondOfDay (valid values 0 - 86399): {second_of_day}")));
        }
        Ok(LocalTime { nano_of_day: second_of_day * NANOS_PER_SECOND })
    }

    /// `LocalTime.of(hour, minute, second, nano)`.
    pub fn of(hour: u32, minute: u32, second: u32, nano: u32) -> Result<LocalTime> {
        if hour > 23 || minute > 59 || second > 59 || nano as i64 >= NANOS_PER_SECOND {
            return Err(date_time(format!("Invalid time {hour}:{minute}:{second}.{nano}")));
        }
        Ok(LocalTime { nano_of_day: ((hour as i64 * 60 + minute as i64) * 60 + second as i64) * NANOS_PER_SECOND + nano as i64 })
    }

    /// `LocalTime.now()` in the default zone (`GMT`).
    pub fn now() -> LocalTime {
        LocalDateTime::of_instant(Instant::now()).time()
    }

    /// `LocalTime.parse(text)`: `HH:mm[:ss[.fraction]]`.
    pub fn parse(text: &str) -> Result<LocalTime> {
        let bad = || Error::new("DateTimeParseException", format!("Text '{text}' could not be parsed at index 0"));
        let (hms, fraction) = match text.split_once('.') {
            Some((a, b)) => (a, Some(b)),
            None => (text, None),
        };
        let parts: Vec<&str> = hms.split(':').collect();
        if !(2..=3).contains(&parts.len()) || parts.iter().any(|p| p.len() != 2) {
            return Err(bad());
        }
        let n = |s: &str| s.parse::<u32>().map_err(|_| bad());
        let nano = match fraction {
            Some(f) if (1..=9).contains(&f.len()) && f.chars().all(|c| c.is_ascii_digit()) => n(&format!("{f:0<9}"))?,
            Some(_) => return Err(bad()),
            None => 0,
        };
        LocalTime::of(n(parts[0])?, n(parts[1])?, if parts.len() == 3 { n(parts[2])? } else { 0 }, nano)
    }

    /// `toNanoOfDay()`.
    pub fn to_nano_of_day(&self) -> i64 {
        self.nano_of_day
    }

    /// `LocalTime.getHour()`.
    pub fn hour(&self) -> u32 {
        (self.nano_of_day / (3600 * NANOS_PER_SECOND)) as u32
    }

    /// `LocalTime.getMinute()`.
    pub fn minute(&self) -> u32 {
        (self.nano_of_day / (60 * NANOS_PER_SECOND) % 60) as u32
    }

    /// `LocalTime.getSecond()`.
    pub fn second(&self) -> u32 {
        (self.nano_of_day / NANOS_PER_SECOND % 60) as u32
    }

    /// `LocalTime.getNano()`.
    pub fn nano(&self) -> u32 {
        (self.nano_of_day % NANOS_PER_SECOND) as u32
    }

    /// `plus(amount, unit)`: wraps around midnight; units longer than half a day are unsupported.
    pub fn plus(&self, amount: i64, unit: ChronoUnit) -> Result<LocalTime> {
        let (per_day, unit_nanos) = match unit {
            ChronoUnit::Nanos => (NANOS_PER_DAY, 1),
            ChronoUnit::Micros => (NANOS_PER_DAY / 1000, 1000),
            ChronoUnit::Millis => (NANOS_PER_DAY / 1_000_000, 1_000_000),
            ChronoUnit::Seconds => (SECONDS_PER_DAY, NANOS_PER_SECOND),
            ChronoUnit::Minutes => (1440, 60 * NANOS_PER_SECOND),
            ChronoUnit::Hours => (24, 3600 * NANOS_PER_SECOND),
            ChronoUnit::HalfDays => (2, 12 * 3600 * NANOS_PER_SECOND),
            other => return Err(unsupported(format!("Unsupported unit: {other}"))),
        };
        let added = (amount % per_day) * unit_nanos;
        Ok(LocalTime { nano_of_day: (self.nano_of_day + added).rem_euclid(NANOS_PER_DAY) })
    }

    /// `minus(amount, unit)`.
    pub fn minus(&self, amount: i64, unit: ChronoUnit) -> Result<LocalTime> {
        if amount == i64::MIN { self.plus(i64::MAX, unit)?.plus(1, unit) } else { self.plus(-amount, unit) }
    }

    /// `plusSeconds(seconds)`.
    pub fn plus_seconds(&self, seconds: i64) -> LocalTime {
        self.plus(seconds, ChronoUnit::Seconds).expect("seconds are supported")
    }

    /// `plusMinutes(minutes)`.
    pub fn plus_minutes(&self, minutes: i64) -> LocalTime {
        self.plus(minutes, ChronoUnit::Minutes).expect("minutes are supported")
    }

    /// `minusSeconds(seconds)`.
    pub fn minus_seconds(&self, seconds: i64) -> LocalTime {
        self.minus(seconds, ChronoUnit::Seconds).expect("seconds are supported")
    }

    /// `LocalTime.isBefore(other)`.
    pub fn is_before(&self, other: &LocalTime) -> bool {
        self < other
    }

    /// `LocalTime.isAfter(other)`.
    pub fn is_after(&self, other: &LocalTime) -> bool {
        self > other
    }

    /// `format(formatter)`.
    pub fn format(&self, formatter: &DateTimeFormatter) -> Result<String> {
        formatter.format(&formatter::Temporal::time(*self))
    }
}

impl fmt::Display for LocalTime {
    /// `toString()`: `HH:mm`, then `:ss` and a fraction in groups of three digits when not zero.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour(), self.minute())?;
        let (second, nano) = (self.second(), self.nano());
        if second > 0 || nano > 0 {
            write!(f, ":{second:02}")?;
            if nano > 0 {
                if nano % 1_000_000 == 0 {
                    write!(f, ".{:03}", nano / 1_000_000)?;
                } else if nano % 1000 == 0 {
                    write!(f, ".{:06}", nano / 1000)?;
                } else {
                    write!(f, ".{nano:09}")?;
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------------
// LocalDateTime

/// `java.time.LocalDateTime`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalDateTime {
    date: LocalDate,
    time: LocalTime,
}

impl LocalDateTime {
    /// `LocalDateTime.of(date, time)`.
    pub fn of(date: LocalDate, time: LocalTime) -> LocalDateTime {
        LocalDateTime { date, time }
    }

    /// `LocalDateTime.ofInstant(instant, ZoneId.systemDefault())`.
    pub fn of_instant(instant: Instant) -> LocalDateTime {
        LocalDateTime::of_instant_offset(instant, default_zone().offset_at(instant.seconds))
    }

    /// `LocalDateTime.ofEpochSecond(instant.epochSecond, instant.nano, offset)`.
    pub(crate) fn of_instant_offset(instant: Instant, offset_seconds: i32) -> LocalDateTime {
        let local = instant.seconds + offset_seconds as i64;
        let days = local.div_euclid(SECONDS_PER_DAY);
        let secs = local.rem_euclid(SECONDS_PER_DAY);
        let (year, month, day) = civil_from_days(days);
        LocalDateTime {
            date: LocalDate { year: year as i32, month, day },
            time: LocalTime { nano_of_day: secs * NANOS_PER_SECOND + instant.nanos as i64 },
        }
    }

    /// `LocalDateTime.now()` in the default zone (`GMT`).
    pub fn now() -> LocalDateTime {
        LocalDateTime::of_instant(Instant::now())
    }

    /// `LocalDateTime.toLocalDate()`.
    pub fn date(&self) -> LocalDate {
        self.date
    }

    /// `LocalDateTime.toLocalTime()`.
    pub fn time(&self) -> LocalTime {
        self.time
    }

    /// Seconds since the epoch at UTC (`toEpochSecond(ZoneOffset.UTC)`).
    pub fn to_epoch_second(&self) -> i64 {
        self.date.to_epoch_day() * SECONDS_PER_DAY + self.time.nano_of_day / NANOS_PER_SECOND
    }

    /// `atZone(systemDefault()).toInstant()`: in a gap the time moves forward by the gap, in an overlap the earlier
    /// offset applies.
    pub fn to_instant(&self) -> Instant {
        self.to_instant_at(default_zone().offset_for_local(self.to_epoch_second(), false))
    }

    /// `toInstant(ZoneOffset.ofTotalSeconds(offset))`.
    pub(crate) fn to_instant_at(self, offset_seconds: i32) -> Instant {
        Instant { seconds: self.to_epoch_second() - offset_seconds as i64, nanos: self.time.nano() as i32 }
    }

    /// `format(formatter)`.
    pub fn format(&self, formatter: &DateTimeFormatter) -> Result<String> {
        formatter.format(&formatter::Temporal::date_time(*self))
    }
}

impl fmt::Display for LocalDateTime {
    /// `toString()`: the date, `T`, the time.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}T{}", self.date, self.time)
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Instant

/// `java.time.Instant`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant {
    seconds: i64,
    nanos: i32,
}

const INSTANT_MIN_SECOND: i64 = -31_557_014_167_219_200;
const INSTANT_MAX_SECOND: i64 = 31_556_889_864_403_199;

impl Instant {
    /// `Instant.EPOCH`.
    pub const EPOCH: Instant = Instant { seconds: 0, nanos: 0 };

    /// `Instant.ofEpochSecond(seconds, nanoAdjustment)`.
    pub fn of_epoch_second(seconds: i64, nano_adjustment: i64) -> Result<Instant> {
        let secs = add_exact(seconds, nano_adjustment.div_euclid(NANOS_PER_SECOND))?;
        let nanos = nano_adjustment.rem_euclid(NANOS_PER_SECOND) as i32;
        if !(INSTANT_MIN_SECOND..=INSTANT_MAX_SECOND).contains(&secs) {
            return Err(date_time("Instant exceeds minimum or maximum instant"));
        }
        Ok(Instant { seconds: secs, nanos })
    }

    /// `Instant.ofEpochMilli(millis)`.
    pub fn of_epoch_milli(millis: i64) -> Instant {
        Instant { seconds: millis.div_euclid(1000), nanos: (millis.rem_euclid(1000) * 1_000_000) as i32 }
    }

    /// `Instant.now()`: the system clock at microsecond resolution.
    pub fn now() -> Instant {
        let (seconds, nanos) = system_now();
        Instant { seconds, nanos }
    }

    /// `Instant.parse(text)`: `DateTimeFormatter.ISO_INSTANT` (case insensitive, STRICT): `ISO_LOCAL_DATE`, `T`,
    /// `HH:mm:ss`, a fraction of up to nine digits and an offset (`Z`, `+01:00`, `-05:30:15`); `24:00:00` is the next
    /// day's midnight and a leap second `23:59:60` reads as `23:59:59`. `DateTimeParseException`s with the JDK's
    /// texts and indexes.
    pub fn parse(text: &str) -> Result<Instant> {
        DateTimeFormatter::iso_instant().parse_query(text, |p| p.to_instant())
    }

    /// `Instant.getEpochSecond()`.
    pub fn epoch_second(&self) -> i64 {
        self.seconds
    }

    /// `Instant.getNano()`: the nanoseconds within the second.
    pub fn nano(&self) -> i32 {
        self.nanos
    }

    /// `toEpochMilli()`.
    pub fn to_epoch_milli(&self) -> Result<i64> {
        if self.seconds < 0 && self.nanos > 0 {
            let millis = multiply_exact(self.seconds + 1, 1000)?;
            add_exact(millis, self.nanos as i64 / 1_000_000 - 1000)
        } else {
            add_exact(multiply_exact(self.seconds, 1000)?, self.nanos as i64 / 1_000_000)
        }
    }

    fn plus_parts(&self, seconds: i64, nanos: i64) -> Result<Instant> {
        if seconds == 0 && nanos == 0 {
            return Ok(*self);
        }
        let epoch_sec = add_exact(add_exact(self.seconds, seconds)?, nanos / NANOS_PER_SECOND)?;
        Instant::of_epoch_second(epoch_sec, self.nanos as i64 + nanos % NANOS_PER_SECOND)
    }

    /// `plusMillis(millis)`.
    pub fn plus_millis(&self, millis: i64) -> Result<Instant> {
        self.plus_parts(millis / 1000, (millis % 1000) * 1_000_000)
    }

    /// `minusMillis(millis)`.
    pub fn minus_millis(&self, millis: i64) -> Result<Instant> {
        if millis == i64::MIN { self.plus_millis(i64::MAX)?.plus_millis(1) } else { self.plus_millis(-millis) }
    }

    /// `plusSeconds(seconds)`.
    pub fn plus_seconds(&self, seconds: i64) -> Result<Instant> {
        self.plus_parts(seconds, 0)
    }

    /// `plus(amount, unit)`: up to days.
    pub fn plus(&self, amount: i64, unit: ChronoUnit) -> Result<Instant> {
        match unit {
            ChronoUnit::Nanos => self.plus_parts(0, amount),
            ChronoUnit::Micros => self.plus_parts(amount / 1_000_000, (amount % 1_000_000) * 1000),
            ChronoUnit::Millis => self.plus_millis(amount),
            ChronoUnit::Seconds => self.plus_seconds(amount),
            ChronoUnit::Minutes | ChronoUnit::Hours | ChronoUnit::HalfDays | ChronoUnit::Days => {
                self.plus_seconds(multiply_exact(amount, unit.seconds())?)
            }
            other => Err(unsupported(format!("Unsupported unit: {other}"))),
        }
    }

    /// `minus(amount, unit)`.
    pub fn minus(&self, amount: i64, unit: ChronoUnit) -> Result<Instant> {
        if amount == i64::MIN { self.plus(i64::MAX, unit)?.plus(1, unit) } else { self.plus(-amount, unit) }
    }

    /// `Instant.isBefore(other)`.
    pub fn is_before(&self, other: &Instant) -> bool {
        self < other
    }

    /// `Instant.isAfter(other)`.
    pub fn is_after(&self, other: &Instant) -> bool {
        self > other
    }
}

impl fmt::Display for Instant {
    /// `toString()` (`DateTimeFormatter.ISO_INSTANT`): `2011-12-03T10:15:30Z`, `+10000-01-01T00:00:00Z`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const SECONDS_PER_10000_YEARS: i64 = 146_097 * 25 * 86_400;
        const SECONDS_0000_TO_1970: i64 = (146_097 * 5 - (30 * 365 + 7)) * 86_400;
        let ldt_text = |seconds: i64, nanos: i32| {
            let ldt = LocalDateTime::of_instant_offset(Instant { seconds, nanos }, 0);
            let mut s = ldt.to_string();
            if ldt.time.second() == 0 && nanos == 0 {
                s.push_str(":00");
            }
            (ldt, s)
        };
        let mut buf = String::new();
        if self.seconds >= -SECONDS_0000_TO_1970 {
            let zero_secs = self.seconds - SECONDS_PER_10000_YEARS + SECONDS_0000_TO_1970;
            let hi = zero_secs.div_euclid(SECONDS_PER_10000_YEARS) + 1;
            let lo = zero_secs.rem_euclid(SECONDS_PER_10000_YEARS);
            if hi > 0 {
                buf.push('+');
                buf.push_str(&hi.to_string());
            }
            buf.push_str(&ldt_text(lo - SECONDS_0000_TO_1970, self.nanos).1);
        } else {
            let zero_secs = self.seconds + SECONDS_0000_TO_1970;
            let hi = zero_secs / SECONDS_PER_10000_YEARS;
            let lo = zero_secs % SECONDS_PER_10000_YEARS;
            let (ldt, mut s) = ldt_text(lo - SECONDS_0000_TO_1970, self.nanos);
            if hi < 0 {
                if ldt.date.year == -10_000 {
                    s.replace_range(0..2, &(hi - 1).to_string());
                } else if lo == 0 {
                    s.insert_str(0, &hi.to_string());
                } else {
                    s.insert_str(1, &hi.abs().to_string());
                }
            }
            buf.push_str(&s);
        }
        buf.push('Z');
        f.write_str(&buf)
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Period

/// `java.time.Period`: years, months and days.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Period {
    years: i32,
    months: i32,
    days: i32,
}

impl Period {
    /// `Period.ZERO`.
    pub const ZERO: Period = Period { years: 0, months: 0, days: 0 };

    /// `Period.of(years, months, days)`.
    pub fn of(years: i32, months: i32, days: i32) -> Period {
        Period { years, months, days }
    }

    /// `Period.parse(text)` for `[-]P[nY][nM][nW][nD]`.
    pub fn parse(text: &str) -> Result<Period> {
        let bad = || Error::new("DateTimeParseException", "Text cannot be parsed to a Period");
        let (negate, rest) = match text.strip_prefix('-') {
            Some(r) => (-1, r),
            None => (1, text.strip_prefix('+').unwrap_or(text)),
        };
        let mut rest = rest.strip_prefix(['P', 'p']).ok_or_else(bad)?;
        let (mut years, mut months, mut weeks, mut days) = (0i32, 0i32, 0i32, 0i32);
        let mut any = false;
        while !rest.is_empty() {
            let end = rest.find(|c: char| c.is_ascii_alphabetic()).ok_or_else(bad)?;
            let n: i32 = rest[..end].parse().map_err(|_| bad())?;
            match rest.as_bytes()[end].to_ascii_uppercase() {
                b'Y' => years = n,
                b'M' => months = n,
                b'W' => weeks = n,
                b'D' => days = n,
                _ => return Err(bad()),
            }
            any = true;
            rest = &rest[end + 1..];
        }
        if !any {
            return Err(bad());
        }
        let days = weeks.checked_mul(7).and_then(|w| w.checked_add(days)).ok_or_else(bad)?;
        Ok(Period { years: years * negate, months: months * negate, days: days * negate })
    }

    /// `Period.getYears()`.
    pub fn years(&self) -> i32 {
        self.years
    }

    /// `Period.getMonths()`.
    pub fn months(&self) -> i32 {
        self.months
    }

    /// `Period.getDays()`.
    pub fn days(&self) -> i32 {
        self.days
    }

    /// `toTotalMonths()`.
    pub fn to_total_months(&self) -> i64 {
        self.years as i64 * 12 + self.months as i64
    }

    /// `minus(other)`: each unit subtracted exactly.
    pub fn minus(&self, other: &Period) -> Result<Period> {
        let sub = |a: i32, b: i32| a.checked_sub(b).ok_or_else(|| Error::new("ArithmeticException", "integer overflow"));
        Ok(Period { years: sub(self.years, other.years)?, months: sub(self.months, other.months)?, days: sub(self.days, other.days)? })
    }

    /// `isNegative()`: any unit below zero.
    pub fn is_negative(&self) -> bool {
        self.years < 0 || self.months < 0 || self.days < 0
    }
}

impl fmt::Display for Period {
    /// `toString()`: `P1Y2M3D`, `P0D`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Period::ZERO {
            return f.write_str("P0D");
        }
        f.write_str("P")?;
        if self.years != 0 {
            write!(f, "{}Y", self.years)?;
        }
        if self.months != 0 {
            write!(f, "{}M", self.months)?;
        }
        if self.days != 0 {
            write!(f, "{}D", self.days)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------------
// java.util.Date and java.sql.Timestamp

/// A `java.util.Date`, or a `java.sql.Timestamp` (a `Date` subclass with nanoseconds): one type, as Datafaker's
/// `<T extends Date>` methods take and return either.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Date {
    /// `Date.fastTime`: for a `Timestamp`, whole seconds only (its millis are in `nanos`).
    time: i64,
    /// `Some(nanos)` for a `Timestamp`.
    nanos: Option<i32>,
}

impl Date {
    /// `new Date(millis)`.
    pub fn new(millis: i64) -> Date {
        Date { time: millis, nanos: None }
    }

    /// `new Timestamp(millis)`.
    pub fn timestamp(millis: i64) -> Date {
        let mut time = (millis / 1000) * 1000;
        let mut nanos = ((millis % 1000) * 1_000_000) as i32;
        if nanos < 0 {
            nanos += 1_000_000_000;
            time = ((millis / 1000) - 1) * 1000;
        }
        Date { time, nanos: Some(nanos) }
    }

    /// `Timestamp.valueOf(dateTime)`: the date-time in the default zone, on the Julian/Gregorian calendar.
    pub fn timestamp_of(date_time: LocalDateTime) -> Date {
        let d = date_time.date;
        let t = date_time.time;
        let day = HybridDate::epoch_day_of(d.year as i64, d.month, d.day);
        let local = day * SECONDS_PER_DAY + t.nano_of_day / NANOS_PER_SECOND;
        let seconds = local - default_zone().util_offset_by_wall(local) as i64;
        Date { time: seconds * 1000, nanos: Some(t.nano() as i32) }
    }

    /// `new Date()` / `new Timestamp(System.currentTimeMillis())` without the timestamp part.
    pub fn now() -> Date {
        Date::new(current_time_millis())
    }

    /// Whether this is a `java.sql.Timestamp`.
    pub fn is_timestamp(&self) -> bool {
        self.nanos.is_some()
    }

    /// `getTime()`.
    pub fn get_time(&self) -> i64 {
        self.time + self.nanos.map_or(0, |n| n as i64 / 1_000_000)
    }

    /// `Timestamp.getNanos()` (for a `Date`, its millis as nanos).
    pub fn nanos(&self) -> i32 {
        self.nanos.unwrap_or((self.time.rem_euclid(1000) * 1_000_000) as i32)
    }

    /// `before(when)` as `Date` declares it (the overload a `<T extends Date>` method reaches).
    pub fn before(&self, when: &Date) -> bool {
        self.get_time() < when.get_time()
    }

    /// `after(when)`.
    pub fn after(&self, when: &Date) -> bool {
        self.get_time() > when.get_time()
    }

    /// `equals(other)`: a `Timestamp` equals only a `Timestamp` with the same time and nanos; a `Date` equals any
    /// `Date` with the same `getTime()`.
    pub fn java_equals(&self, other: &Date) -> bool {
        match self.nanos {
            Some(n) => other.nanos == Some(n) && self.time == other.time,
            None => self.get_time() == other.get_time(),
        }
    }

    /// `new Date(text)`: [`Date::parse_millis`].
    pub fn parse_text(text: &str) -> Result<Date> {
        Ok(Date::new(Date::parse_millis(text)?))
    }

    /// `Date.parse(text)`, the deprecated lenient parser behind `new Date(String)`: words and numbers in any order
    /// (`Sat, 12 Aug 1995 13:30:00 GMT`, `8/12/95 1:30 PM`, `12 August 1995 13:30 GMT+0430`), the English month and
    /// weekday names and the zones of RFC 822 by prefix, two-digit years in the century from 80 years before the
    /// clock's year, the proleptic Gregorian calendar from 1582 and the Julian one before it, times without a zone in
    /// the default zone. Anything else is an `IllegalArgumentException` without a message.
    pub fn parse_millis(text: &str) -> Result<i64> {
        const WORDS: [&str; 32] = [
            "am", "pm", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday", "january", "february",
            "march", "april", "may", "june", "july", "august", "september", "october", "november", "december", "gmt", "ut",
            "utc", "est", "edt", "cst", "cdt", "mst", "mdt", "pst", "pdt",
        ];
        const ACTIONS: [i32; 32] = [
            14, 1, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 10000, 10000, 10000, 10000 + 5 * 60,
            10000 + 4 * 60, 10000 + 6 * 60, 10000 + 5 * 60, 10000 + 7 * 60, 10000 + 6 * 60, 10000 + 8 * 60, 10000 + 7 * 60,
        ];
        let syntax = || Error::illegal_argument("null");
        let byte = |n: i32| n as i8 as i32;
        let s: Vec<u16> = text.encode_utf16().collect();
        let (mut year, mut mon, mut mday, mut hour, mut min, mut sec) = (i32::MIN, -1, -1, -1, -1, -1);
        let (mut tzoffset, mut prevc) = (-1i32, 0i32);
        let mut i = 0;
        let limit = s.len();
        let is_digit = |c: i32| (b'0' as i32..=b'9' as i32).contains(&c);
        while i < limit {
            let mut c = s[i] as i32;
            i += 1;
            if c <= b' ' as i32 || c == b',' as i32 {
                continue;
            }
            if c == b'(' as i32 {
                let mut depth = 1;
                while i < limit {
                    c = s[i] as i32;
                    i += 1;
                    if c == b'(' as i32 {
                        depth += 1;
                    } else if c == b')' as i32 {
                        depth -= 1;
                        if depth <= 0 {
                            break;
                        }
                    }
                }
                continue;
            }
            if is_digit(c) {
                let mut n = c - b'0' as i32;
                while i < limit && {
                    c = s[i] as i32;
                    is_digit(c)
                } {
                    n = n.wrapping_mul(10).wrapping_add(c - b'0' as i32);
                    i += 1;
                }
                if prevc == b'+' as i32 || prevc == b'-' as i32 && year != i32::MIN {
                    n = if n < 24 { n.wrapping_mul(60) } else { n % 100 + (n / 100).wrapping_mul(60) };
                    if prevc == b'+' as i32 {
                        n = n.wrapping_neg();
                    }
                    if tzoffset != 0 && tzoffset != -1 {
                        return Err(syntax());
                    }
                    tzoffset = n;
                } else if n >= 70 {
                    if year != i32::MIN {
                        return Err(syntax());
                    } else if c <= b' ' as i32 || c == b',' as i32 || c == b'/' as i32 || i >= limit {
                        year = n;
                    } else {
                        return Err(syntax());
                    }
                } else if c == b':' as i32 {
                    if hour < 0 {
                        hour = byte(n);
                    } else if min < 0 {
                        min = byte(n);
                    } else {
                        return Err(syntax());
                    }
                } else if c == b'/' as i32 {
                    if mon < 0 {
                        mon = byte(n - 1);
                    } else if mday < 0 {
                        mday = byte(n);
                    } else {
                        return Err(syntax());
                    }
                } else if i < limit && c != b',' as i32 && c > b' ' as i32 && c != b'-' as i32 {
                    return Err(syntax());
                } else if hour >= 0 && min < 0 {
                    min = byte(n);
                } else if min >= 0 && sec < 0 {
                    sec = byte(n);
                } else if mday < 0 {
                    mday = byte(n);
                } else if year == i32::MIN && mon >= 0 && mday >= 0 {
                    year = n;
                } else {
                    return Err(syntax());
                }
                prevc = 0;
            } else if c == b'/' as i32 || c == b':' as i32 || c == b'+' as i32 || c == b'-' as i32 {
                prevc = c;
            } else {
                let st = i - 1;
                while i < limit && (s[i] as u8 as u16 == s[i]) && (s[i] as u8).is_ascii_alphabetic() {
                    i += 1;
                }
                if i <= st + 1 {
                    return Err(syntax());
                }
                let word = &s[st..i];
                let k = (0..WORDS.len()).rev().find(|&k| {
                    let w = WORDS[k].as_bytes();
                    word.len() <= w.len() && word.iter().zip(w).all(|(a, b)| (*a as u8).eq_ignore_ascii_case(b))
                });
                let Some(k) = k else { return Err(syntax()) };
                match ACTIONS[k] {
                    0 => {}
                    1 => {
                        if !(1..=12).contains(&hour) {
                            return Err(syntax());
                        } else if hour < 12 {
                            hour += 12;
                        }
                    }
                    14 => {
                        if !(1..=12).contains(&hour) {
                            return Err(syntax());
                        } else if hour == 12 {
                            hour = 0;
                        }
                    }
                    action if action <= 13 => {
                        if mon < 0 {
                            mon = action - 2;
                        } else {
                            return Err(syntax());
                        }
                    }
                    action => tzoffset = action - 10000,
                }
                prevc = 0;
            }
        }
        if year == i32::MIN || mon < 0 || mday < 0 {
            return Err(syntax());
        }
        if year < 100 {
            // `defaultCenturyStart`: the clock's year (in the default zone) less 80.
            let start = LocalDateTime::of_instant(Instant::now()).date().year() - 80;
            year += (start / 100) * 100;
            if year < start {
                year += 100;
            }
        }
        let (sec, min, hour) = (sec.max(0), min.max(0), hour.max(0));
        // `BaseCalendar.normalizeMonth`, then the fixed date of the first of the month plus the day, linearly.
        let mut y = year as i64;
        let mut month = mon as i64 + 1;
        if month <= 0 {
            let xm = 1 - month;
            y -= xm / 12 + 1;
            month = 13 - xm % 12;
            if month == 13 {
                y += 1;
                month = 1;
            }
        } else if month > 12 {
            y += (month - 1) / 12;
            month = (month - 1) % 12 + 1;
        }
        let first = if year >= 1582 { days_from_civil(y, month, 1) } else { julian_epoch_day(y, month, 1) };
        let day = first + mday as i64 - 1;
        let time_of_day = ((hour as i64 * 60 + min as i64) * 60 + sec as i64) * 1000;
        let ms = day.wrapping_mul(86_400_000).wrapping_add(time_of_day);
        if tzoffset == -1 {
            let offset = default_zone().util_offset_by_wall(ms.div_euclid(1000));
            return Ok(ms.wrapping_sub(offset as i64 * 1000));
        }
        Ok(ms.wrapping_add(tzoffset.wrapping_mul(60 * 1000) as i64))
    }

    /// `TimeZone.getDefault().getDisplayName(inDaylightTime(this), style, Locale.US)`.
    pub(crate) fn zone_name(&self, long: bool) -> String {
        let zone = default_zone();
        zone.time_zone_name(zone.util_offset_at(self.get_time().div_euclid(1000)).1, long)
    }

    /// The fields of `getTime()` in the default zone on the Julian/Gregorian calendar, and the millis of the day.
    pub(crate) fn fields(&self) -> (i64, HybridDate, i64) {
        let millis = self.get_time();
        let millis = millis + default_zone().util_offset_at(millis.div_euclid(1000)).0 as i64 * 1000;
        let day = millis.div_euclid(86_400_000);
        (day, HybridDate::of_epoch_day(day), millis.rem_euclid(86_400_000))
    }
}

pub(crate) const MONTHS: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
/// Monday first.
pub(crate) const WEEKDAYS: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

impl fmt::Display for Date {
    /// `Date.toString()` (`EEE MMM dd HH:mm:ss zzz yyyy`) or `Timestamp.toString()`
    /// (`yyyy-mm-dd hh:mm:ss.fffffffff`, trailing zeros of the fraction dropped).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (day, date, ms) = self.fields();
        let (hour, minute, second) = (ms / 3_600_000, ms / 60_000 % 60, ms / 1000 % 60);
        match self.nanos {
            None => write!(
                f,
                "{} {} {:02} {hour:02}:{minute:02}:{second:02} {} {}",
                &WEEKDAYS[day_of_week(day) as usize - 1][..3],
                &MONTHS[date.month as usize - 1][..3],
                date.day,
                self.zone_name(false),
                date.year_of_era
            ),
            Some(nanos) => {
                let mut fraction = nanos.to_string();
                if nanos == 0 {
                    fraction = "0".into();
                } else {
                    fraction = format!("{fraction:0>9}").trim_end_matches('0').to_string();
                }
                write!(f, "{:04}-{:02}-{:02} {hour:02}:{minute:02}:{second:02}.{fraction}", date.year_of_era, date.month, date.day)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil() {
        for day in [-800_000i64, -141_428, -141_427, -1, 0, 1, 11_016, 19_000, 2_932_896] {
            let (y, m, d) = civil_from_days(day);
            assert_eq!(days_from_civil(y, m as i64, d as i64), day);
        }
        assert_eq!(LocalDate::of_epoch_day(0).unwrap().to_string(), "1970-01-01");
        assert_eq!(LocalDate::of(2024, 2, 29).unwrap().to_epoch_day(), 19_782);
        assert_eq!(LocalDate::of(2024, 2, 29).unwrap().minus_years(1).unwrap().to_string(), "2023-02-28");
        assert_eq!(LocalDate::of(2023, 1, 31).unwrap().plus_period(Period::of(1, 1, 1)).unwrap().to_string(), "2024-03-01");
        assert_eq!(LocalDate::of(-44, 3, 15).unwrap().to_string(), "-0044-03-15");
        assert_eq!(LocalDate::of(10_000, 1, 1).unwrap().to_string(), "+10000-01-01");
        assert_eq!(LocalDate::of(1970, 1, 1).unwrap().day_of_week(), 4);
    }

    #[test]
    fn fixed_clock() {
        let instant = Instant::parse("2024-06-01T12:00:00Z").unwrap();
        let inner = with_fixed_clock(instant, || {
            assert_eq!(LocalDate::now().to_string(), "2024-06-01");
            with_fixed_clock(Instant::parse("1999-12-31T23:59:59Z").unwrap(), LocalDate::now)
        });
        assert_eq!(inner.to_string(), "1999-12-31");
        assert_ne!(LocalDate::now().to_string(), "2024-06-01");
    }

    #[test]
    fn hybrid() {
        let cutover = HybridDate::of_epoch_day(GREGORIAN_CUTOVER_DAY);
        assert_eq!((cutover.year, cutover.month, cutover.day, cutover.julian), (1582, 10, 15, false));
        let before = HybridDate::of_epoch_day(GREGORIAN_CUTOVER_DAY - 1);
        assert_eq!((before.year, before.month, before.day, before.julian), (1582, 10, 4, true));
        assert_eq!(HybridDate::epoch_day_of(1582, 10, 4), GREGORIAN_CUTOVER_DAY - 1);
        assert_eq!(HybridDate::epoch_day_of(1, 1, 1), -719_164);
    }

    #[test]
    fn local_time() {
        let t = LocalTime::of(23, 59, 30, 0).unwrap();
        assert_eq!(t.to_string(), "23:59:30");
        assert_eq!(t.plus(1, ChronoUnit::Minutes).unwrap().to_string(), "00:00:30");
        assert_eq!(t.minus(25, ChronoUnit::Hours).unwrap().to_string(), "22:59:30");
        assert_eq!(LocalTime::MIDNIGHT.to_string(), "00:00");
        assert_eq!(LocalTime::of(1, 2, 3, 4_000).unwrap().to_string(), "01:02:03.000004");
        assert_eq!(LocalTime::parse("10:15:30.5").unwrap().to_string(), "10:15:30.500");
        assert_eq!(t.plus(1, ChronoUnit::Days).unwrap_err().message(), "Unsupported unit: Days");
    }

    #[test]
    fn instant() {
        assert_eq!(Instant::EPOCH.to_string(), "1970-01-01T00:00:00Z");
        assert_eq!(Instant::of_epoch_milli(1_700_000_000_123).to_string(), "2023-11-14T22:13:20.123Z");
        assert_eq!(Instant::of_epoch_milli(-1).to_string(), "1969-12-31T23:59:59.999Z");
        assert_eq!(Instant::of_epoch_milli(-1).to_epoch_milli().unwrap(), -1);
        assert_eq!(Instant::parse("2023-11-14T22:13:20.123Z").unwrap(), Instant::of_epoch_milli(1_700_000_000_123));
        assert_eq!(Instant::of_epoch_second(253_402_300_800, 0).unwrap().to_string(), "+10000-01-01T00:00:00Z");
        assert_eq!(Instant::of_epoch_second(-62_167_219_201, 0).unwrap().to_string(), "-0001-12-31T23:59:59Z");
    }

    #[test]
    fn instant_parse_offsets() {
        // What `Instant.parse` on MockServer 8.0.0's JDK accepts after the seconds (`PUT /mockserver/clock`).
        let at = |text: &str| Instant::parse(&format!("2024-01-01T00:00:00{text}")).ok().map(|i| i.to_string());
        for (offset, expected) in [("Z", "2024-01-01T00:00:00Z"), ("z", "2024-01-01T00:00:00Z"), ("+01:00", "2023-12-31T23:00:00Z"), ("+0100", "2023-12-31T23:00:00Z"), ("+01", "2023-12-31T23:00:00Z"), ("-05", "2024-01-01T05:00:00Z"), ("-0530", "2024-01-01T05:30:00Z"), ("+01:00:30", "2023-12-31T22:59:30Z"), ("+010030", "2023-12-31T22:59:30Z"), ("+18:00", "2023-12-31T06:00:00Z")] {
            assert_eq!(at(offset).as_deref(), Some(expected), "{offset}");
        }
        for offset in ["", "+1", "+01:0", "+01:00:3", "+0100:00", "+01:", "+01:00Z", "+24", "+18:01", "+010", "+01000", "+01:60", "+0160", "+01:00:60"] {
            assert_eq!(at(offset), None, "{offset}");
        }
    }

    #[test]
    fn period_and_duration() {
        assert_eq!(Period::ZERO.to_string(), "P0D");
        assert_eq!(Period::of(1, -2, 3).to_string(), "P1Y-2M3D");
        assert!(Period::of(0, 1, 1).minus(&Period::of(1, 1, 1)).unwrap().is_negative());
        assert_eq!(Period::parse("P1Y2M3W4D").unwrap(), Period::of(1, 2, 25));
        assert_eq!(Duration::of(1500, ChronoUnit::Millis).unwrap().to_string(), "PT1.5S");
        assert_eq!(Duration::of(-1, ChronoUnit::Micros).unwrap().to_string(), "PT-0.000001S");
        assert_eq!(Duration::of(1, ChronoUnit::Weeks).unwrap_err().message(), "Unit must not have an estimated duration");
        assert_eq!(TimeUnit::Days.to_millis(i64::MAX / 2), i64::MAX);
    }

    #[test]
    fn dates() {
        assert_eq!(Date::new(1_700_000_000_123).to_string(), "Tue Nov 14 22:13:20 GMT 2023");
        assert_eq!(Date::timestamp(1_700_000_000_123).to_string(), "2023-11-14 22:13:20.123");
        assert_eq!(Date::timestamp(1_700_000_000_000).to_string(), "2023-11-14 22:13:20.0");
        assert_eq!(Date::timestamp(-1).get_time(), -1);
        let ldt = LocalDate::of(2000, 1, 2).unwrap().at_time(LocalTime::of_nano_of_day(1_234_567_891).unwrap());
        assert_eq!(Date::timestamp_of(ldt).to_string(), "2000-01-02 00:00:01.234567891");
        assert!(!Date::timestamp(5).java_equals(&Date::new(5)));
        assert!(Date::new(5).java_equals(&Date::timestamp(5)));
    }
}
