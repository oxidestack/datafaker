//! `net.datafaker.providers.base.TimeAndDate` (`faker.timeAndDate()`).

use crate::dispatch::{JValue, Method, P, npe};
use crate::java::{ChronoUnit, DateTimeFormatter, Duration, Instant, LocalDate, Period, TimeUnit};
use crate::{Error, Faker, Result};

pub(crate) const DEFAULT_MIN_AGE: i32 = 18;
pub(crate) const DEFAULT_MAX_AGE: i32 = 65;

/// `TimeUnit` as a method-table parameter.
pub(crate) const TIME_UNIT: P = P::Enum("TimeUnit", TimeUnit::NAMES);
/// `ChronoUnit` as a method-table parameter.
pub(crate) const CHRONO_UNIT: P = P::Enum("ChronoUnit", ChronoUnit::NAMES);

/// Random instants, birthdays, durations and periods on the `java.time` API.
#[derive(Clone, Copy)]
pub struct TimeAndDate<'a> {
    faker: &'a Faker,
}

/// `DateTimeFormatter.ofPattern(pattern).withZone(ZoneId.systemDefault()).format(instant)`.
fn format_instant(instant: Instant, pattern: &str) -> Result<String> {
    DateTimeFormatter::of_pattern(pattern)?.with_default_zone().format_instant(instant)
}

/// The same for a `LocalDate` (the zone only answers zone queries).
fn format_date(date: LocalDate, pattern: &str) -> Result<String> {
    date.format(&DateTimeFormatter::of_pattern(pattern)?.with_default_zone())
}

impl<'a> TimeAndDate<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        TimeAndDate { faker }
    }

    /// A future instant, up to fifty years from now.
    pub fn future(&self) -> Result<Instant> {
        let fifty_years = TimeUnit::Days.to_millis(18_262);
        let at_most = self.faker.number().number_between_long(1, fifty_years)?;
        self.future_within(at_most, TimeUnit::Milliseconds)
    }

    /// `future(atMost, unit)`: after now, less than `atMost` units ahead.
    pub fn future_within(&self, at_most: i64, unit: TimeUnit) -> Result<Instant> {
        let a_bit_later = Instant::now().plus_millis(1)?;
        self.future_from(at_most, unit, a_bit_later)
    }

    /// `future(atMost, unit, pattern)`.
    pub fn future_within_formatted(&self, at_most: i64, unit: TimeUnit, pattern: &str) -> Result<String> {
        format_instant(self.future_within(at_most, unit)?, pattern)
    }

    /// `future(atMost, minimum, unit)`.
    pub fn future_between(&self, at_most: i64, minimum: i64, unit: TimeUnit) -> Result<Instant> {
        let minimum_date = Instant::now().plus(minimum, unit.to_chrono_unit())?;
        self.future_from(at_most.wrapping_sub(minimum), unit, minimum_date)
    }

    /// `future(atMost, minimum, unit, pattern)`.
    pub fn future_between_formatted(&self, at_most: i64, minimum: i64, unit: TimeUnit, pattern: &str) -> Result<String> {
        format_instant(self.future_between(at_most, minimum, unit)?, pattern)
    }

    /// `future(atMost, unit, referenceDate)`: after `reference`, less than `atMost` units ahead.
    pub fn future_from(&self, at_most: i64, unit: TimeUnit, reference: Instant) -> Result<Instant> {
        let upper_bound = unit.to_millis(at_most);
        let reference = reference.to_epoch_milli()?;
        let offset = self.faker.random().next_long_bounded(upper_bound.wrapping_sub(1))?;
        Ok(Instant::of_epoch_milli(reference.wrapping_add(1).wrapping_add(offset)))
    }

    /// `future(atMost, unit, referenceDate, pattern)`.
    pub fn future_from_formatted(&self, at_most: i64, unit: TimeUnit, reference: Instant, pattern: &str) -> Result<String> {
        format_instant(self.future_from(at_most, unit, reference)?, pattern)
    }

    /// A past instant, up to fifty years ago.
    pub fn past(&self) -> Result<Instant> {
        let fifty_years = TimeUnit::Days.to_millis(18_262);
        let at_most = self.faker.number().number_between_long(1, fifty_years)?;
        self.past_within(at_most, TimeUnit::Milliseconds)
    }

    /// `past(atMost, unit)`.
    pub fn past_within(&self, at_most: i64, unit: TimeUnit) -> Result<Instant> {
        let a_bit_earlier = Instant::now().minus_millis(1)?;
        self.past_from(at_most, unit, a_bit_earlier)
    }

    /// `past(atMost, unit, pattern)`.
    pub fn past_within_formatted(&self, at_most: i64, unit: TimeUnit, pattern: &str) -> Result<String> {
        format_instant(self.past_within(at_most, unit)?, pattern)
    }

    /// `past(atMost, minimum, unit)`.
    pub fn past_between(&self, at_most: i64, minimum: i64, unit: TimeUnit) -> Result<Instant> {
        let minimum_date = Instant::now().minus_millis(unit.to_millis(minimum))?;
        self.past_from(at_most.wrapping_sub(minimum), unit, minimum_date)
    }

    /// `past(atMost, minimum, unit, pattern)`.
    pub fn past_between_formatted(&self, at_most: i64, minimum: i64, unit: TimeUnit, pattern: &str) -> Result<String> {
        format_instant(self.past_between(at_most, minimum, unit)?, pattern)
    }

    /// `past(atMost, unit, referenceDate)`.
    pub fn past_from(&self, at_most: i64, unit: TimeUnit, reference: Instant) -> Result<Instant> {
        let upper_bound = unit.to_millis(at_most);
        let reference = reference.to_epoch_milli()?;
        let offset = self.faker.random().next_long_bounded(upper_bound.wrapping_sub(1))?;
        Ok(Instant::of_epoch_milli(reference.wrapping_sub(1).wrapping_sub(offset)))
    }

    /// `past(atMost, unit, referenceDate, pattern)`.
    pub fn past_from_formatted(&self, at_most: i64, unit: TimeUnit, reference: Instant, pattern: &str) -> Result<String> {
        format_instant(self.past_from(at_most, unit, reference)?, pattern)
    }

    /// `between(from, to)`: `from` inclusive, `to` exclusive.
    pub fn between(&self, from: Instant, to: Instant) -> Result<Instant> {
        if to.is_before(&from) {
            return Err(Error::illegal_argument(format!(
                "Invalid date range: the upper bound date ({to}) is before the lower bound ({from})"
            )));
        }
        if from == to {
            return Ok(from);
        }
        let span = to.to_epoch_milli()?.wrapping_sub(from.to_epoch_milli()?);
        let offset = self.faker.random().next_long_bounded(span)?;
        from.plus_millis(offset)
    }

    /// `between(from, to, pattern)`.
    pub fn between_formatted(&self, from: Instant, to: Instant, pattern: &str) -> Result<String> {
        format_instant(self.between(from, to)?, pattern)
    }

    /// A birthday between 65 and 18 years ago.
    pub fn birthday(&self) -> Result<LocalDate> {
        self.birthday_between(DEFAULT_MIN_AGE, DEFAULT_MAX_AGE)
    }

    /// `birthday(pattern)`.
    pub fn birthday_formatted(&self, pattern: &str) -> Result<String> {
        format_date(self.birthday_between(DEFAULT_MIN_AGE, DEFAULT_MAX_AGE)?, pattern)
    }

    /// `birthday(minAge, maxAge)`.
    pub fn birthday_between(&self, min_age: i32, max_age: i32) -> Result<LocalDate> {
        self.birthday_on(LocalDate::now(), min_age, max_age)
    }

    /// `birthday(minAge, maxAge)` with `today` for `LocalDate.now()`.
    pub fn birthday_on(&self, today: LocalDate, min_age: i32, max_age: i32) -> Result<LocalDate> {
        let from = today.minus_years(max_age as i64)?;
        if min_age == max_age {
            return Ok(from);
        }
        let start = from.to_epoch_day();
        let stop = today.minus_years(min_age as i64)?.to_epoch_day();
        LocalDate::of_epoch_day(self.faker.random().next_long_between(start, stop)?)
    }

    /// `birthday(minAge, maxAge, pattern)`.
    pub fn birthday_between_formatted(&self, min_age: i32, max_age: i32, pattern: &str) -> Result<String> {
        format_date(self.birthday_between(min_age, max_age)?, pattern)
    }

    /// `duration(max, unit)`.
    pub fn duration(&self, max: i64, unit: ChronoUnit) -> Result<Duration> {
        self.duration_between(0, max, unit)
    }

    /// `duration(min, max, unit)`.
    pub fn duration_between(&self, min: i64, max: i64, unit: ChronoUnit) -> Result<Duration> {
        Duration::of(self.faker.random().next_long_between(min, max)?, unit)
    }

    /// `period(min, max)`: each unit from its minimum to its maximum, both inclusive.
    pub fn period(&self, min: Period, max: Period) -> Result<Period> {
        random_period(self.faker, min, max)
    }
}

/// `TimeAndDate.period` and `DateAndTime.period`.
pub(crate) fn random_period(faker: &Faker, min: Period, max: Period) -> Result<Period> {
    if max.minus(&min)?.is_negative() {
        return Err(Error::illegal_argument(format!("Max period({max}) should be not less than min ({min})")));
    }
    let random = faker.random();
    Ok(Period::of(
        random.next_int_between(min.years(), max.years())?,
        random.next_int_between(min.months(), max.months())?,
        random.next_int_between(min.days(), max.days())?,
    ))
}

pub(crate) fn time_unit(v: &JValue) -> TimeUnit {
    match v {
        JValue::Enum(name) => TimeUnit::value_of(name),
        _ => None,
    }
    .unwrap_or(TimeUnit::Milliseconds)
}

pub(crate) fn chrono_unit(v: &JValue) -> ChronoUnit {
    match v {
        JValue::Enum(name) => ChronoUnit::value_of(name),
        _ => None,
    }
    .unwrap_or(ChronoUnit::Millis)
}

/// A method-table result: the value's `toString()`.
pub(crate) fn text<T: ToString>(r: Result<T>) -> Result<JValue> {
    r.map(|v| JValue::Str(v.to_string()))
}

// Overloads taking an `Instant` or a `Period` are last: expressions cannot reach them, as neither class has a
// `(String)` constructor for `FakeValuesService.coerceArguments`, and a template only with nulls. A template cannot pass
// a `TimeUnit` or a `ChronoUnit` either (Velocity converts no string to one), so a unit that gets here is null.
pub(crate) static METHODS: &[Method] = &[
    Method::new("future", &[], |f, _| text(TimeAndDate::new(f).future())),
    Method::new("future", &[P::Long, TIME_UNIT], |f, a| {
        crate::dispatch::non_null(a, 1, TO_MILLIS)?;
        text(TimeAndDate::new(f).future_within(a[0].as_long(), time_unit(&a[1])))
    })
    .accepting_null(),
    Method::new("future", &[P::Long, TIME_UNIT, P::String], |f, a| {
        crate::dispatch::non_null(a, 1, TO_MILLIS)?;
        text(TimeAndDate::new(f).future_within_formatted(a[0].as_long(), time_unit(&a[1]), a[2].as_str()))
    })
    .accepting_null(),
    Method::new("future", &[P::Long, P::Long, TIME_UNIT], |f, a| {
        crate::dispatch::non_null(a, 2, "Cannot invoke \"java.util.concurrent.TimeUnit.toChronoUnit()\" because \"unit\" is null")?;
        text(TimeAndDate::new(f).future_between(a[0].as_long(), a[1].as_long(), time_unit(&a[2])))
    })
    .accepting_null(),
    Method::new("future", &[P::Long, P::Long, TIME_UNIT, P::String], |f, a| {
        crate::dispatch::non_null(a, 2, "Cannot invoke \"java.util.concurrent.TimeUnit.toChronoUnit()\" because \"unit\" is null")?;
        text(TimeAndDate::new(f).future_between_formatted(a[0].as_long(), a[1].as_long(), time_unit(&a[2]), a[3].as_str()))
    })
    .accepting_null(),
    Method::new("past", &[], |f, _| text(TimeAndDate::new(f).past())),
    Method::new("past", &[P::Long, TIME_UNIT], |f, a| {
        crate::dispatch::non_null(a, 1, TO_MILLIS)?;
        text(TimeAndDate::new(f).past_within(a[0].as_long(), time_unit(&a[1])))
    })
    .accepting_null(),
    Method::new("past", &[P::Long, TIME_UNIT, P::String], |f, a| {
        crate::dispatch::non_null(a, 1, TO_MILLIS)?;
        text(TimeAndDate::new(f).past_within_formatted(a[0].as_long(), time_unit(&a[1]), a[2].as_str()))
    })
    .accepting_null(),
    Method::new("past", &[P::Long, P::Long, TIME_UNIT], |f, a| {
        crate::dispatch::non_null(a, 2, TO_MILLIS)?;
        text(TimeAndDate::new(f).past_between(a[0].as_long(), a[1].as_long(), time_unit(&a[2])))
    })
    .accepting_null(),
    Method::new("past", &[P::Long, P::Long, TIME_UNIT, P::String], |f, a| {
        crate::dispatch::non_null(a, 2, TO_MILLIS)?;
        text(TimeAndDate::new(f).past_between_formatted(a[0].as_long(), a[1].as_long(), time_unit(&a[2]), a[3].as_str()))
    })
    .accepting_null(),
    Method::new("birthday", &[], |f, _| text(TimeAndDate::new(f).birthday())),
    // `DateTimeFormatter.ofPattern(null)` (`Objects.requireNonNull(pattern, "pattern")`) once the birthday is drawn.
    Method::new("birthday", &[P::String], |f, a| {
        if a[0] == JValue::Null {
            TimeAndDate::new(f).birthday()?;
            return Err(npe("pattern"));
        }
        text(TimeAndDate::new(f).birthday_formatted(a[0].as_str()))
    })
    .accepting_null(),
    Method::new("birthday", &[P::Int, P::Int], |f, a| text(TimeAndDate::new(f).birthday_between(a[0].as_int(), a[1].as_int()))),
    Method::new("birthday", &[P::Int, P::Int, P::String], |f, a| {
        if a[2] == JValue::Null {
            TimeAndDate::new(f).birthday_between(a[0].as_int(), a[1].as_int())?;
            return Err(npe("pattern"));
        }
        text(TimeAndDate::new(f).birthday_between_formatted(a[0].as_int(), a[1].as_int(), a[2].as_str()))
    })
    .accepting_null(),
    // `Duration.of(random.nextLong(min, max), null)` (`Objects.requireNonNull(unit, "unit")`).
    Method::new("duration", &[P::Long, CHRONO_UNIT], |f, a| {
        if a[1] == JValue::Null {
            f.random().next_long_between(0, a[0].as_long())?;
            return Err(npe("unit"));
        }
        text(TimeAndDate::new(f).duration(a[0].as_long(), chrono_unit(&a[1])))
    })
    .accepting_null(),
    Method::new("duration", &[P::Long, P::Long, CHRONO_UNIT], |f, a| {
        if a[2] == JValue::Null {
            f.random().next_long_between(a[0].as_long(), a[1].as_long())?;
            return Err(npe("unit"));
        }
        text(TimeAndDate::new(f).duration_between(a[0].as_long(), a[1].as_long(), chrono_unit(&a[2])))
    })
    .accepting_null(),
    Method::new("future", &[P::Long, TIME_UNIT, P::Opaque("Instant")], |_, _| Err(npe(TO_MILLIS))).accepting_null(),
    Method::new("future", &[P::Long, TIME_UNIT, P::Opaque("Instant"), P::String], |_, _| Err(npe(TO_MILLIS))).accepting_null(),
    Method::new("past", &[P::Long, TIME_UNIT, P::Opaque("Instant")], |_, _| Err(npe(TO_MILLIS))).accepting_null(),
    Method::new("past", &[P::Long, TIME_UNIT, P::Opaque("Instant"), P::String], |_, _| Err(npe(TO_MILLIS))).accepting_null(),
    Method::new("between", &[P::Opaque("Instant"), P::Opaque("Instant")], |_, _| Err(npe(IS_BEFORE))).accepting_null(),
    Method::new("between", &[P::Opaque("Instant"), P::Opaque("Instant"), P::String], |_, _| Err(npe(IS_BEFORE))).accepting_null(),
    Method::new("period", &[P::Opaque("Period"), P::Opaque("Period")], |_, _| Err(npe(super::date_and_time::PERIOD_MINUS))).accepting_null(),
];

const TO_MILLIS: &str = "Cannot invoke \"java.util.concurrent.TimeUnit.toMillis(long)\" because \"unit\" is null";
const IS_BEFORE: &str = "Cannot invoke \"java.time.Instant.isBefore(java.time.Instant)\" because \"to\" is null";
