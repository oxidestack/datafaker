//! `net.datafaker.providers.base.Business`.

use super::time_and_date::random_period;
use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::java::{DateTimeFormatter, LocalDate, Period};
use crate::{Faker, Result};

/// Credit card numbers, types, expiry dates and security codes.
#[derive(Clone, Copy)]
pub struct Business<'a> {
    faker: &'a Faker,
}

impl<'a> Business<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Business { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Business, key)
    }

    /// `business.credit_card_numbers_format`, numerified.
    pub fn credit_card_number(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("business.credit_card_numbers_format")?))
    }

    /// `business.credit_card_types`.
    pub fn credit_card_type(&self) -> Result<String> {
        self.resolve("business.credit_card_types")
    }

    /// Today plus a random period of up to 20 years, as `yyyy-MM-dd`.
    pub fn credit_card_expiry(&self) -> Result<String> {
        self.credit_card_expiry_from(LocalDate::now())
    }

    /// `creditCardExpiry()` with `today` for `LocalDate.now()`.
    pub fn credit_card_expiry_from(&self, today: LocalDate) -> Result<String> {
        let period = random_period(self.faker, Period::ZERO, Period::of(20, 0, 0))?;
        today.plus_period(period)?.format(&DateTimeFormatter::of_pattern("yyyy-MM-dd")?)
    }

    /// `business.security_code`, numerified.
    pub fn security_code(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("business.security_code")?))
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("creditCardNumber", &[], |f, _| s(Business::new(f).credit_card_number())),
    Method::new("creditCardType", &[], |f, _| s(Business::new(f).credit_card_type())),
    Method::new("creditCardExpiry", &[], |f, _| s(Business::new(f).credit_card_expiry())),
    Method::new("securityCode", &[], |f, _| s(Business::new(f).security_code())),
];
