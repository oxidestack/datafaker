//! `net.datafaker.providers.base.Country`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Countries, from `country.*`; currencies from [`Money`](super::Money).
#[derive(Clone, Copy)]
pub struct Country<'a> {
    faker: &'a Faker,
}

impl<'a> Country<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Country { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Country, key)
    }

    /// The flag image URL of a `country.code2`.
    pub fn flag(&self) -> Result<String> {
        Ok(format!("https://flags.fmcdn.net/data/flags/w580/{}.png", self.resolve("country.code2")?))
    }

    /// `country.code2`.
    pub fn country_code2(&self) -> Result<String> {
        self.resolve("country.code2")
    }

    /// `country.code3`.
    pub fn country_code3(&self) -> Result<String> {
        self.resolve("country.code3")
    }

    /// `country.capital`.
    pub fn capital(&self) -> Result<String> {
        self.resolve("country.capital")
    }

    /// `money().currency()`.
    pub fn currency(&self) -> Result<String> {
        self.faker.money().currency()
    }

    /// `money().currencyCode()`.
    pub fn currency_code(&self) -> Result<String> {
        self.faker.money().currency_code()
    }

    /// `country.name`.
    pub fn name(&self) -> Result<String> {
        self.resolve("country.name")
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("flag", &[], |f, _| s(Country::new(f).flag())),
    Method::new("countryCode2", &[], |f, _| s(Country::new(f).country_code2())),
    Method::new("countryCode3", &[], |f, _| s(Country::new(f).country_code3())),
    Method::new("capital", &[], |f, _| s(Country::new(f).capital())),
    Method::new("currency", &[], |f, _| s(Country::new(f).currency())),
    Method::new("currencyCode", &[], |f, _| s(Country::new(f).currency_code())),
    Method::new("name", &[], |f, _| s(Country::new(f).name())),
];
