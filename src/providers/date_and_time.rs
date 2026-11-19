//! `net.datafaker.providers.base.DateAndTime` (`faker.date()`, deprecated upstream for `TimeAndDate`): the same on
//! `java.util.Date`/`java.sql.Timestamp`, formatted with `SimpleDateFormat`.

use super::time_and_date::{CHRONO_UNIT, TIME_UNIT, chrono_unit, random_period, text, time_unit};
use crate::dispatch::{JValue, Method, P, non_null, npe};
use crate::java::time::current_time_millis;
use crate::java::{ChronoUnit, Date, Duration, LocalDate, LocalDateTime, LocalTime, Period, SimpleDateFormat, TimeUnit};
use crate::{Error, Faker, Result};

const DEFAULT_MIN_AGE: i32 = 18;
const DEFAULT_MAX_AGE: i32 = 65;
/// `DateAndTime.DAYS_NANOS`.
pub const DAYS_NANOS: i64 = 86_400_000_000_000;

/// Random dates and timestamps, birthdays, durations and periods.
#[derive(Clone, Copy)]
pub struct DateAndTime<'a> {
    faker: &'a Faker,
}

/// `new SimpleDateFormat(pattern).format(date)`.
fn format(date: Date, pattern: &str) -> Result<String> {
    Ok(SimpleDateFormat::new(pattern)?.format(&date))
}

/// `DateAndTime.str2durationUnit(unit)`.
pub fn str2duration_unit(unit: Option<&str>) -> Result<ChronoUnit> {
    let illegal = || Error::illegal_argument(format!("Illegal duration unit '{}'", unit.unwrap_or("null")));
    let Some(u) = unit.filter(|u| !u.trim_matches(|c: char| c <= ' ').is_empty()) else { return Err(illegal()) };
    Ok(match u.to_uppercase().as_str() {
        "NANO" | "NANOS" => ChronoUnit::Nanos,
        "MICRO" | "MICROS" => ChronoUnit::Micros,
        "MILLI" | "MILLIS" => ChronoUnit::Millis,
        "SECOND" | "SECONDS" => ChronoUnit::Seconds,
        "MINUTE" | "MINUTES" => ChronoUnit::Minutes,
        "HOUR" | "HOURS" => ChronoUnit::Hours,
        "DAY" | "DAYS" => ChronoUnit::Days,
        _ => return Err(illegal()),
    })
}

impl<'a> DateAndTime<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        DateAndTime { faker }
    }

    /// `future(atMost, unit)`: a timestamp after now, less than `atMost` units ahead.
    pub fn future(&self, at_most: i32, unit: TimeUnit) -> Result<Date> {
        let a_bit_later = Date::timestamp(current_time_millis() + 1);
        self.future_from(at_most, unit, a_bit_later)
    }

    /// `future(atMost, unit, pattern)`.
    pub fn future_formatted(&self, at_most: i32, unit: TimeUnit, pattern: &str) -> Result<String> {
        format(self.future(at_most, unit)?, pattern)
    }

    /// `future(atMost, minimum, unit)`.
    pub fn future_between(&self, at_most: i32, minimum: i32, unit: TimeUnit) -> Result<Date> {
        let minimum_date = Date::timestamp(current_time_millis().wrapping_add(unit.to_millis(minimum as i64)));
        self.future_from(at_most.wrapping_sub(minimum), unit, minimum_date)
    }

    /// `future(atMost, minimum, unit, pattern)`.
    pub fn future_between_formatted(&self, at_most: i32, minimum: i32, unit: TimeUnit, pattern: &str) -> Result<String> {
        format(self.future_between(at_most, minimum, unit)?, pattern)
    }

    /// `future(atMost, unit, referenceDate)`: always a `Timestamp`.
    pub fn future_from(&self, at_most: i32, unit: TimeUnit, reference: Date) -> Result<Date> {
        let upper_bound = unit.to_millis(at_most as i64);
        let millis = reference.get_time();
        let offset = self.faker.random().next_long_bounded(upper_bound.wrapping_sub(1))?;
        Ok(Date::timestamp(millis.wrapping_add(1).wrapping_add(offset)))
    }

    /// `future(atMost, unit, referenceDate, pattern)`.
    pub fn future_from_formatted(&self, at_most: i32, unit: TimeUnit, reference: Date, pattern: &str) -> Result<String> {
        format(self.future_from(at_most, unit, reference)?, pattern)
    }

    /// `past(atMost, unit)`.
    pub fn past(&self, at_most: i32, unit: TimeUnit) -> Result<Date> {
        let a_bit_earlier = Date::timestamp(current_time_millis() - 1);
        self.past_from(at_most, unit, a_bit_earlier)
    }

    /// `past(atMost, unit, pattern)`.
    pub fn past_formatted(&self, at_most: i32, unit: TimeUnit, pattern: &str) -> Result<String> {
        format(self.past(at_most, unit)?, pattern)
    }

    /// `past(atMost, minimum, unit)`.
    pub fn past_between(&self, at_most: i32, minimum: i32, unit: TimeUnit) -> Result<Date> {
        let minimum_date = Date::timestamp(current_time_millis().wrapping_sub(unit.to_millis(minimum as i64)));
        self.past_from(at_most.wrapping_sub(minimum), unit, minimum_date)
    }

    /// `past(atMost, minimum, unit, pattern)`.
    pub fn past_between_formatted(&self, at_most: i32, minimum: i32, unit: TimeUnit, pattern: &str) -> Result<String> {
        format(self.past_between(at_most, minimum, unit)?, pattern)
    }

    /// `past(atMost, unit, referenceDate)`: always a `Timestamp`.
    pub fn past_from(&self, at_most: i32, unit: TimeUnit, reference: Date) -> Result<Date> {
        let upper_bound = unit.to_millis(at_most as i64);
        let millis = reference.get_time();
        let offset = self.faker.random().next_long_bounded(upper_bound.wrapping_sub(1))?;
        Ok(Date::timestamp(millis.wrapping_sub(1).wrapping_sub(offset)))
    }

    /// `past(atMost, unit, referenceDate, pattern)`.
    pub fn past_from_formatted(&self, at_most: i32, unit: TimeUnit, reference: Date, pattern: &str) -> Result<String> {
        format(self.past_from(at_most, unit, reference)?, pattern)
    }

    /// `between(from, to)`: `from` itself when they are equal, otherwise a `Timestamp` from `from` inclusive to `to`
    /// exclusive.
    pub fn between(&self, from: Date, to: Date) -> Result<Date> {
        if to.before(&from) {
            return Err(Error::illegal_argument(format!(
                "Invalid date range: the upper bound date ({to}) is before the lower bound ({from})"
            )));
        }
        if from.java_equals(&to) {
            return Ok(from);
        }
        let offset = self.faker.random().next_long_bounded(to.get_time().wrapping_sub(from.get_time()))?;
        Ok(Date::timestamp(from.get_time().wrapping_add(offset)))
    }

    /// `between(from, to, pattern)`.
    pub fn between_formatted(&self, from: Date, to: Date, pattern: &str) -> Result<String> {
        format(self.between(from, to)?, pattern)
    }

    /// A birthday timestamp between 65 and 18 years ago.
    pub fn birthday(&self) -> Result<Date> {
        self.birthday_between(DEFAULT_MIN_AGE, DEFAULT_MAX_AGE)
    }

    /// `birthdayLocalDate()`.
    pub fn birthday_local_date(&self) -> Result<LocalDate> {
        self.birthday_local_date_between(DEFAULT_MIN_AGE, DEFAULT_MAX_AGE)
    }

    /// `birthday(pattern)`.
    pub fn birthday_formatted(&self, pattern: &str) -> Result<String> {
        format(self.birthday_between(DEFAULT_MIN_AGE, DEFAULT_MAX_AGE)?, pattern)
    }

    /// `birthday(minAge, maxAge)`: a random day and a random time of day.
    pub fn birthday_between(&self, min_age: i32, max_age: i32) -> Result<Date> {
        self.birthday_on(LocalDate::now(), min_age, max_age)
    }

    /// `birthday(minAge, maxAge)` with `today` for `LocalDate.now()`.
    pub fn birthday_on(&self, today: LocalDate, min_age: i32, max_age: i32) -> Result<Date> {
        let from = today.minus_years(max_age as i64)?;
        if min_age == max_age {
            return Ok(Date::timestamp_of(LocalDateTime::of(from, LocalTime::MIDNIGHT)));
        }
        let start = from.to_epoch_day();
        let stop = today.minus_years(min_age as i64)?.to_epoch_day();
        let date = LocalDate::of_epoch_day(self.faker.random().next_long_between(start, stop)?)?;
        let time = LocalTime::of_nano_of_day(self.faker.number().number_between_long(0, DAYS_NANOS)?)?;
        Ok(Date::timestamp_of(LocalDateTime::of(date, time)))
    }

    /// `birthdayLocalDate(minAge, maxAge)`.
    pub fn birthday_local_date_between(&self, min_age: i32, max_age: i32) -> Result<LocalDate> {
        super::TimeAndDate::new(self.faker).birthday_between(min_age, max_age)
    }

    /// `birthday(minAge, maxAge, pattern)`.
    pub fn birthday_between_formatted(&self, min_age: i32, max_age: i32, pattern: &str) -> Result<String> {
        format(self.birthday_between(min_age, max_age)?, pattern)
    }

    /// `duration(max, unit)`.
    pub fn duration(&self, max: i64, unit: ChronoUnit) -> Result<Duration> {
        self.duration_between(0, max, unit)
    }

    /// `duration(min, max, unit)`.
    pub fn duration_between(&self, min: i64, max: i64, unit: ChronoUnit) -> Result<Duration> {
        Duration::of(self.faker.random().next_long_between(min, max)?, unit)
    }

    /// `duration(max, unit)` with the unit by name (`"days"`, `"HOUR"`, …).
    pub fn duration_named(&self, max: i64, unit: Option<&str>) -> Result<Duration> {
        self.duration_between(0, max, str2duration_unit(unit)?)
    }

    /// `duration(min, max, unit)` with the unit by name.
    pub fn duration_between_named(&self, min: i64, max: i64, unit: Option<&str>) -> Result<Duration> {
        let value = self.faker.random().next_long_between(min, max)?;
        Duration::of(value, str2duration_unit(unit)?)
    }

    /// `period(min, max)`.
    pub fn period(&self, min: Period, max: Period) -> Result<Period> {
        random_period(self.faker, min, max)
    }
}

/// A `Date` coerced from an expression argument (`new Date(String)`): its millis.
fn date(v: &crate::dispatch::JValue) -> Date {
    Date::new(v.as_long())
}

// In the order of `DateAndTime.class.getMethods()` in MockServer 8.0.0's image (Temurin 26.0.2, the shaded class),
// which decides between overloads that both coerce: `future(int, TimeUnit, String)` before `future(int, TimeUnit,
// Date)` (JDK 25 has them the other way round), `past(int, TimeUnit, Date)` before `past(int, TimeUnit, String)`;
// the image gives these pairs the same order on every run. `period(Period, Period)` is last: `Period` has no
// `(String)` constructor, so expressions never reach it, and a template only with nulls. A template cannot pass a
// `TimeUnit` either (Velocity converts no string to one), so the unit of a call that gets here is null.
pub(crate) static METHODS: &[Method] = &[
    Method::new("future", &[P::Int, TIME_UNIT, P::String], |f, a| {
        non_null(a, 1, "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")?;
        text(DateAndTime::new(f).future_formatted(a[0].as_int(), time_unit(&a[1]), a[2].as_str()))
    })
    .accepting_null(),
    Method::new("future", &[P::Int, TIME_UNIT], |f, a| {
        non_null(a, 1, "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")?;
        text(DateAndTime::new(f).future(a[0].as_int(), time_unit(&a[1])))
    })
    .accepting_null(),
    Method::new("future", &[P::Int, TIME_UNIT, P::Date], |f, a| {
        non_null(a, 1, "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")?;
        text(DateAndTime::new(f).future_from(a[0].as_int(), time_unit(&a[1]), date(&a[2])))
    })
    .accepting_null(),
    Method::new("future", &[P::Int, P::Int, TIME_UNIT, P::String], |f, a| {
        non_null(a, 2, "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")?;
        text(DateAndTime::new(f).future_between_formatted(a[0].as_int(), a[1].as_int(), time_unit(&a[2]), a[3].as_str()))
    })
    .accepting_null(),
    Method::new("future", &[P::Int, P::Int, TIME_UNIT], |f, a| {
        non_null(a, 2, "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")?;
        text(DateAndTime::new(f).future_between(a[0].as_int(), a[1].as_int(), time_unit(&a[2])))
    })
    .accepting_null(),
    Method::new("future", &[P::Int, TIME_UNIT, P::Date, P::String], |f, a| {
        non_null(a, 1, "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")?;
        text(DateAndTime::new(f).future_from_formatted(a[0].as_int(), time_unit(&a[1]), date(&a[2]), a[3].as_str()))
    })
    .accepting_null(),
    Method::new("past", &[P::Int, TIME_UNIT, P::Date], |f, a| {
        non_null(a, 1, "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")?;
        text(DateAndTime::new(f).past_from(a[0].as_int(), time_unit(&a[1]), date(&a[2])))
    })
    .accepting_null(),
    Method::new("past", &[P::Int, P::Int, TIME_UNIT], |f, a| {
        non_null(a, 2, "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")?;
        text(DateAndTime::new(f).past_between(a[0].as_int(), a[1].as_int(), time_unit(&a[2])))
    })
    .accepting_null(),
    Method::new("past", &[P::Int, TIME_UNIT, P::Date, P::String], |f, a| {
        non_null(a, 1, "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")?;
        text(DateAndTime::new(f).past_from_formatted(a[0].as_int(), time_unit(&a[1]), date(&a[2]), a[3].as_str()))
    })
    .accepting_null(),
    Method::new("past", &[P::Int, P::Int, TIME_UNIT, P::String], |f, a| {
        non_null(a, 2, "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")?;
        text(DateAndTime::new(f).past_between_formatted(a[0].as_int(), a[1].as_int(), time_unit(&a[2]), a[3].as_str()))
    })
    .accepting_null(),
    Method::new("past", &[P::Int, TIME_UNIT], |f, a| {
        non_null(a, 1, "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")?;
        text(DateAndTime::new(f).past(a[0].as_int(), time_unit(&a[1])))
    })
    .accepting_null(),
    Method::new("past", &[P::Int, TIME_UNIT, P::String], |f, a| {
        non_null(a, 1, "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null")?;
        text(DateAndTime::new(f).past_formatted(a[0].as_int(), time_unit(&a[1]), a[2].as_str()))
    })
    .accepting_null(),
    Method::new("birthday", &[], |f, _| text(DateAndTime::new(f).birthday())),
    Method::new("birthdayLocalDate", &[], |f, _| text(DateAndTime::new(f).birthday_local_date())),
    // `new SimpleDateFormat(null)` throws a bare `NullPointerException` once the birthday is drawn.
    Method::new("birthday", &[P::String], |f, a| {
        if a[0] == JValue::Null {
            DateAndTime::new(f).birthday()?;
            return Err(Error::without_message("NullPointerException"));
        }
        text(DateAndTime::new(f).birthday_formatted(a[0].as_str()))
    })
    .accepting_null(),
    Method::new("birthday", &[P::Int, P::Int], |f, a| text(DateAndTime::new(f).birthday_between(a[0].as_int(), a[1].as_int()))),
    Method::new("birthdayLocalDate", &[P::Int, P::Int], |f, a| {
        text(DateAndTime::new(f).birthday_local_date_between(a[0].as_int(), a[1].as_int()))
    }),
    Method::new("birthday", &[P::Int, P::Int, P::String], |f, a| {
        if a[2] == JValue::Null {
            DateAndTime::new(f).birthday_between(a[0].as_int(), a[1].as_int())?;
            return Err(Error::without_message("NullPointerException"));
        }
        text(DateAndTime::new(f).birthday_between_formatted(a[0].as_int(), a[1].as_int(), a[2].as_str()))
    })
    .accepting_null(),
    Method::new("duration", &[P::Long, CHRONO_UNIT], |f, a| text(DateAndTime::new(f).duration(a[0].as_long(), chrono_unit(&a[1])))),
    Method::new("duration", &[P::Long, P::Long, P::String], |f, a| {
        text(DateAndTime::new(f).duration_between_named(a[0].as_long(), a[1].as_long(), Some(a[2].as_str())))
    }),
    Method::new("duration", &[P::Long, P::String], |f, a| text(DateAndTime::new(f).duration_named(a[0].as_long(), Some(a[1].as_str())))),
    Method::new("duration", &[P::Long, P::Long, CHRONO_UNIT], |f, a| {
        text(DateAndTime::new(f).duration_between(a[0].as_long(), a[1].as_long(), chrono_unit(&a[2])))
    }),
    // A template reaches `between` with a null `to` only (it has no `Date` to pass).
    Method::new("between", &[P::Date, P::Date], |f, a| {
        non_null(a, 1, BEFORE)?;
        text(DateAndTime::new(f).between(date(&a[0]), date(&a[1])))
    })
    .accepting_null(),
    Method::new("between", &[P::Date, P::Date, P::String], |f, a| {
        non_null(a, 1, BEFORE)?;
        text(DateAndTime::new(f).between_formatted(date(&a[0]), date(&a[1]), a[2].as_str()))
    })
    .accepting_null(),
    // A template reaches `period(Period, Period)` with null periods only.
    Method::new("period", &[P::Opaque("Period"), P::Opaque("Period")], |_, _| Err(npe(PERIOD_MINUS))).accepting_null(),
];

const BEFORE: &str = "Cannot invoke \"java.util.Date.before(java.util.Date)\" because \"to\" is null";

/// `max.minus(min)`, the first thing `period(min, max)` does (`TimeAndDate.period` too).
pub(crate) const PERIOD_MINUS: &str = "Cannot invoke \"java.time.Period.minus(java.time.temporal.TemporalAmount)\" because \"max\" is null";
