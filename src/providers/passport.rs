//! `net.datafaker.providers.base.Passport`.

use super::s;
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Passport numbers.
#[derive(Clone, Copy)]
pub struct Passport<'a> {
    faker: &'a Faker,
}

impl<'a> Passport<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Passport { faker }
    }

    /// A string matching the locale's regex at `passport.valid`.
    pub fn valid(&self) -> Result<String> {
        self.faker.regexify(&self.faker.resolve("passport.valid")?)
    }
}

pub(crate) static METHODS: &[Method] = &[Method::new("valid", &[], |f, _| s(Passport::new(f).valid()))];
