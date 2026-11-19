//! `net.datafaker.providers.base.Nation`.

use super::iso_codes::{ISO_COUNTRIES, ISO_LANGUAGES};
use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Nations, from `nation.*`, and the JDK's ISO 639 and ISO 3166 codes.
#[derive(Clone, Copy)]
pub struct Nation<'a> {
    faker: &'a Faker,
}

impl<'a> Nation<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Nation { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Nation, key)
    }

    /// `nation.nationality`.
    pub fn nationality(&self) -> Result<String> {
        self.resolve("nation.nationality")
    }

    /// `nation.language`.
    pub fn language(&self) -> Result<String> {
        self.resolve("nation.language")
    }

    /// `nation.capital_city`.
    pub fn capital_city(&self) -> Result<String> {
        self.resolve("nation.capital_city")
    }

    /// `nation.flag`.
    pub fn flag(&self) -> Result<String> {
        self.resolve("nation.flag")
    }

    /// One of `Locale.getISOLanguages()`.
    pub fn iso_language(&self) -> Result<String> {
        self.faker.options().option(ISO_LANGUAGES).map(str::to_string)
    }

    /// One of `Locale.getISOCountries()`.
    pub fn iso_country(&self) -> Result<String> {
        self.faker.options().option(ISO_COUNTRIES).map(str::to_string)
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("nationality", &[], |f, _| s(Nation::new(f).nationality())),
    Method::new("language", &[], |f, _| s(Nation::new(f).language())),
    Method::new("capitalCity", &[], |f, _| s(Nation::new(f).capital_city())),
    Method::new("flag", &[], |f, _| s(Nation::new(f).flag())),
    Method::new("isoLanguage", &[], |f, _| s(Nation::new(f).iso_language())),
    Method::new("isoCountry", &[], |f, _| s(Nation::new(f).iso_country())),
];
