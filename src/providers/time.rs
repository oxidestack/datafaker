//! `net.datafaker.providers.base.Time` (`faker.time()`): times of day as nano of day.

use super::time_and_date::{CHRONO_UNIT, chrono_unit};
use crate::dispatch::{JValue, Method, P, npe};
use crate::java::{ChronoUnit, DateTimeFormatter, LocalTime};
use crate::{Error, Faker, Result};

/// Random times of day (`LocalTime.toNanoOfDay()` values).
#[derive(Clone, Copy)]
pub struct Time<'a> {
    faker: &'a Faker,
}

/// `LocalTime.ofNanoOfDay(nanos).format(DateTimeFormatter.ofPattern(pattern))`.
fn format(nanos: i64, pattern: &str) -> Result<String> {
    let time = LocalTime::of_nano_of_day(nanos)?;
    time.format(&DateTimeFormatter::of_pattern(pattern)?)
}

impl<'a> Time<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Time { faker }
    }

    fn number_between(&self, min: i64, max: i64) -> Result<i64> {
        self.faker.number().number_between_long(min, max)
    }

    /// `future(atMost, unit)`: from now to `atMost` units ahead (wrapping past midnight, as `LocalTime` does).
    pub fn future(&self, at_most: i32, unit: ChronoUnit) -> Result<i64> {
        self.future_at(LocalTime::now(), at_most, unit)
    }

    /// `future(atMost, unit)` with `now` for `LocalTime.now()`.
    pub fn future_at(&self, now: LocalTime, at_most: i32, unit: ChronoUnit) -> Result<i64> {
        let at_most_time = now.plus(at_most as i64, unit)?.to_nano_of_day();
        self.number_between(now.to_nano_of_day(), at_most_time)
    }

    /// `future(atMost, unit, pattern)`.
    pub fn future_formatted(&self, at_most: i32, unit: ChronoUnit, pattern: &str) -> Result<String> {
        format(self.future(at_most, unit)?, pattern)
    }

    /// `future(atMost, minimum, unit)`.
    pub fn future_between(&self, at_most: i32, minimum: i32, unit: ChronoUnit) -> Result<i64> {
        self.future_between_at(LocalTime::now(), at_most, minimum, unit)
    }

    /// `future(atMost, minimum, unit)` with `now` for `LocalTime.now()`.
    pub fn future_between_at(&self, now: LocalTime, at_most: i32, minimum: i32, unit: ChronoUnit) -> Result<i64> {
        let at_most_time = now.plus(at_most as i64, unit)?.to_nano_of_day();
        let time = now.plus(minimum as i64, unit)?.to_nano_of_day();
        self.number_between(time, at_most_time)
    }

    /// `future(atMost, minimum, unit, pattern)`.
    pub fn future_between_formatted(&self, at_most: i32, minimum: i32, unit: ChronoUnit, pattern: &str) -> Result<String> {
        format(self.future_between(at_most, minimum, unit)?, pattern)
    }

    /// `past(atLeast, unit)`.
    pub fn past(&self, at_least: i32, unit: ChronoUnit) -> Result<i64> {
        self.past_at(LocalTime::now(), at_least, unit)
    }

    /// `past(atLeast, unit)` with `now` for `LocalTime.now()`.
    pub fn past_at(&self, now: LocalTime, at_least: i32, unit: ChronoUnit) -> Result<i64> {
        let at_least_time = now.minus(at_least as i64, unit)?.to_nano_of_day();
        self.number_between(at_least_time, now.to_nano_of_day())
    }

    /// `past(atMost, unit, pattern)`.
    pub fn past_formatted(&self, at_most: i32, unit: ChronoUnit, pattern: &str) -> Result<String> {
        format(self.past(at_most, unit)?, pattern)
    }

    /// `past(atMost, minimum, unit)`.
    pub fn past_between(&self, at_most: i32, minimum: i32, unit: ChronoUnit) -> Result<i64> {
        self.past_between_at(LocalTime::now(), at_most, minimum, unit)
    }

    /// `past(atMost, minimum, unit)` with `now` for `LocalTime.now()`.
    pub fn past_between_at(&self, now: LocalTime, at_most: i32, minimum: i32, unit: ChronoUnit) -> Result<i64> {
        let at_most_time = now.minus(at_most as i64, unit)?.to_nano_of_day();
        let time = now.minus(minimum as i64, unit)?.to_nano_of_day();
        self.number_between(at_most_time, time)
    }

    /// `past(atMost, minimum, unit, pattern)`.
    pub fn past_between_formatted(&self, at_most: i32, minimum: i32, unit: ChronoUnit, pattern: &str) -> Result<String> {
        format(self.past_between(at_most, minimum, unit)?, pattern)
    }

    /// `between(from, to)`: `from` inclusive, `to` exclusive.
    pub fn between(&self, from: LocalTime, to: LocalTime) -> Result<i64> {
        if to.is_before(&from) {
            return Err(Error::illegal_argument(format!(
                "Invalid time range: the upper bound time ({to}) is before the lower bound ({from})"
            )));
        }
        if from == to {
            return Ok(from.to_nano_of_day());
        }
        self.number_between(from.to_nano_of_day(), to.to_nano_of_day())
    }

    /// `between(from, to, pattern)`.
    pub fn between_formatted(&self, from: LocalTime, to: LocalTime, pattern: &str) -> Result<String> {
        format(self.between(from, to)?, pattern)
    }
}

fn long(r: Result<i64>) -> Result<JValue> {
    r.map(JValue::Long)
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

// The `between` overloads take `LocalTime`s, which have no `(String)` constructor: expressions cannot reach them, and
// a template only with nulls. A template cannot pass a `ChronoUnit` either (Velocity converts no string to one), so a
// unit that gets here is null: `LocalTime.plus(amount, null)` throws.
pub(crate) static METHODS: &[Method] = &[
    Method::new("future", &[P::Int, CHRONO_UNIT], |f, a| {
        crate::dispatch::non_null(a, 1, "Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")?;
        long(Time::new(f).future(a[0].as_int(), chrono_unit(&a[1])))
    })
    .accepting_null(),
    Method::new("future", &[P::Int, CHRONO_UNIT, P::String], |f, a| {
        crate::dispatch::non_null(a, 1, "Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")?;
        s(Time::new(f).future_formatted(a[0].as_int(), chrono_unit(&a[1]), a[2].as_str()))
    })
    .accepting_null(),
    Method::new("future", &[P::Int, P::Int, CHRONO_UNIT], |f, a| {
        crate::dispatch::non_null(a, 2, "Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")?;
        long(Time::new(f).future_between(a[0].as_int(), a[1].as_int(), chrono_unit(&a[2])))
    })
    .accepting_null(),
    Method::new("future", &[P::Int, P::Int, CHRONO_UNIT, P::String], |f, a| {
        crate::dispatch::non_null(a, 2, "Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")?;
        s(Time::new(f).future_between_formatted(a[0].as_int(), a[1].as_int(), chrono_unit(&a[2]), a[3].as_str()))
    })
    .accepting_null(),
    Method::new("past", &[P::Int, CHRONO_UNIT], |f, a| {
        crate::dispatch::non_null(a, 1, "Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")?;
        long(Time::new(f).past(a[0].as_int(), chrono_unit(&a[1])))
    })
    .accepting_null(),
    Method::new("past", &[P::Int, CHRONO_UNIT, P::String], |f, a| {
        crate::dispatch::non_null(a, 1, "Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")?;
        s(Time::new(f).past_formatted(a[0].as_int(), chrono_unit(&a[1]), a[2].as_str()))
    })
    .accepting_null(),
    Method::new("past", &[P::Int, P::Int, CHRONO_UNIT], |f, a| {
        crate::dispatch::non_null(a, 2, "Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")?;
        long(Time::new(f).past_between(a[0].as_int(), a[1].as_int(), chrono_unit(&a[2])))
    })
    .accepting_null(),
    Method::new("past", &[P::Int, P::Int, CHRONO_UNIT, P::String], |f, a| {
        crate::dispatch::non_null(a, 2, "Cannot invoke \"java.time.temporal.TemporalUnit.addTo(java.time.temporal.Temporal, long)\" because \"<parameter2>\" is null")?;
        s(Time::new(f).past_between_formatted(a[0].as_int(), a[1].as_int(), chrono_unit(&a[2]), a[3].as_str()))
    })
    .accepting_null(),
    Method::new("between", &[P::Opaque("LocalTime"), P::Opaque("LocalTime")], |_, _| Err(npe(IS_BEFORE))).accepting_null(),
    Method::new("between", &[P::Opaque("LocalTime"), P::Opaque("LocalTime"), P::String], |_, _| Err(npe(IS_BEFORE))).accepting_null(),
];

const IS_BEFORE: &str = "Cannot invoke \"java.time.LocalTime.isBefore(java.time.LocalTime)\" because \"to\" is null";
