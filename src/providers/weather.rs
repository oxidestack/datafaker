//! `net.datafaker.providers.base.Weather`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method, P};
use crate::{Faker, Result};

const DEFAULT_MIN_TEMP_C: i32 = -30;
const DEFAULT_MAX_TEMP_C: i32 = 38;
const DEFAULT_MIN_TEMP_F: i32 = -22;
const DEFAULT_MAX_TEMP_F: i32 = 100;

/// Weather, from `weather.*`.
#[derive(Clone, Copy)]
pub struct Weather<'a> {
    faker: &'a Faker,
}

impl<'a> Weather<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Weather { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Weather, key)
    }

    /// `weather.description`.
    pub fn description(&self) -> Result<String> {
        self.resolve("weather.description")
    }

    /// `temperatureCelsius()`: -30 to 38 °C.
    pub fn temperature_celsius(&self) -> Result<String> {
        self.temperature(DEFAULT_MIN_TEMP_C, DEFAULT_MAX_TEMP_C, "weather.temperature.celsius")
    }

    /// `temperatureFahrenheit()`: -22 to 100 °F.
    pub fn temperature_fahrenheit(&self) -> Result<String> {
        self.temperature(DEFAULT_MIN_TEMP_F, DEFAULT_MAX_TEMP_F, "weather.temperature.fahrenheit")
    }

    /// `temperatureCelsius(minTemperature, maxTemperature)`.
    pub fn temperature_celsius_between(&self, min_temperature: i32, max_temperature: i32) -> Result<String> {
        self.temperature(min_temperature, max_temperature, "weather.temperature.celsius")
    }

    /// `temperatureFahrenheit(minTemperature, maxTemperature)`.
    pub fn temperature_fahrenheit_between(&self, min_temperature: i32, max_temperature: i32) -> Result<String> {
        self.temperature(min_temperature, max_temperature, "weather.temperature.fahrenheit")
    }

    /// `random().nextInt(min, max)` (both inclusive) followed by the unit at `degree_key`.
    fn temperature(&self, min_temperature: i32, max_temperature: i32, degree_key: &str) -> Result<String> {
        let degrees = self.faker.random().next_int_between(min_temperature, max_temperature)?;
        Ok(format!("{degrees}{}", self.resolve(degree_key)?))
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("description", &[], |f, _| s(Weather::new(f).description())),
    Method::new("temperatureCelsius", &[], |f, _| s(Weather::new(f).temperature_celsius())),
    Method::new("temperatureFahrenheit", &[], |f, _| s(Weather::new(f).temperature_fahrenheit())),
    Method::new("temperatureCelsius", &[P::Int, P::Int], |f, a| s(Weather::new(f).temperature_celsius_between(a[0].as_int(), a[1].as_int()))),
    Method::new("temperatureFahrenheit", &[P::Int, P::Int], |f, a| s(Weather::new(f).temperature_fahrenheit_between(a[0].as_int(), a[1].as_int()))),
];
