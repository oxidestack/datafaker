//! `net.datafaker.providers.base.Money`.

use super::jdk_tables::CURRENCIES;
use super::s;
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Currencies of the JDK (`Currency.getAvailableCurrencies()`), read by `scripts/jdk-tables.sh`.
///
/// Upstream copies a `HashSet` of `Currency` objects, whose order follows identity hash codes and changes between
/// JVM runs; here the currencies are in code order, so a seed draws other currencies than any one JVM run does.
/// Names and symbols are those of the JVM's default locale `en_US`.
#[derive(Clone, Copy)]
pub struct Money<'a> {
    faker: &'a Faker,
}

impl<'a> Money<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Money { faker }
    }

    fn random_currency(&self) -> Result<(&'static str, &'static str, &'static str, &'static str)> {
        Ok(CURRENCIES[self.faker.random().next_int_bounded(CURRENCIES.len() as i32)? as usize])
    }

    /// `getDisplayName()` of a random currency.
    pub fn currency(&self) -> Result<String> {
        Ok(self.random_currency()?.2.to_string())
    }

    /// `getCurrencyCode()` of a random currency.
    pub fn currency_code(&self) -> Result<String> {
        Ok(self.random_currency()?.0.to_string())
    }

    /// `getNumericCodeAsString()` of a random currency.
    pub fn currency_numeric_code(&self) -> Result<String> {
        Ok(self.random_currency()?.1.to_string())
    }

    /// `getSymbol()` of a random currency.
    pub fn currency_symbol(&self) -> Result<String> {
        Ok(self.random_currency()?.3.to_string())
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("currency", &[], |f, _| s(Money::new(f).currency())),
    Method::new("currencyCode", &[], |f, _| s(Money::new(f).currency_code())),
    Method::new("currencyNumericCode", &[], |f, _| s(Money::new(f).currency_numeric_code())),
    Method::new("currencySymbol", &[], |f, _| s(Money::new(f).currency_symbol())),
];
