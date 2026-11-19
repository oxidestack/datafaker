//! `net.datafaker.providers.base.FakeDuration` (`faker.duration()`).

use crate::dispatch::{JValue, Method, P};
use crate::java::Duration;
use crate::{Faker, Result};

/// Random `java.time.Duration`s up to a maximum (inclusive).
#[derive(Clone, Copy)]
pub struct FakeDuration<'a> {
    faker: &'a Faker,
}

impl<'a> FakeDuration<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        FakeDuration { faker }
    }

    /// `nextLong(max + 1)`.
    fn number_between(&self, max: i64) -> Result<i64> {
        self.faker.random().next_long_bounded(max.wrapping_add(1))
    }

    /// 0 to `minutes_max` minutes.
    pub fn at_most_minutes(&self, minutes_max: i64) -> Result<Duration> {
        Duration::of_minutes(self.number_between(minutes_max)?)
    }

    /// 0 to `hours_max` hours.
    pub fn at_most_hours(&self, hours_max: i64) -> Result<Duration> {
        Duration::of_hours(self.number_between(hours_max)?)
    }

    /// 0 to `days_max` days.
    pub fn at_most_days(&self, days_max: i64) -> Result<Duration> {
        Duration::of_days(self.number_between(days_max)?)
    }

    /// 0 to `seconds_max` seconds.
    pub fn at_most_seconds(&self, seconds_max: i64) -> Result<Duration> {
        Ok(Duration::of_seconds(self.number_between(seconds_max)?))
    }
}

fn d(r: Result<Duration>) -> Result<JValue> {
    r.map(|d| JValue::Str(d.to_string()))
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("atMostMinutes", &[P::Long], |f, a| d(FakeDuration::new(f).at_most_minutes(a[0].as_long()))),
    Method::new("atMostHours", &[P::Long], |f, a| d(FakeDuration::new(f).at_most_hours(a[0].as_long()))),
    Method::new("atMostDays", &[P::Long], |f, a| d(FakeDuration::new(f).at_most_days(a[0].as_long()))),
    Method::new("atMostSeconds", &[P::Long], |f, a| d(FakeDuration::new(f).at_most_seconds(a[0].as_long()))),
];
