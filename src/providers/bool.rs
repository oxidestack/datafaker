//! `net.datafaker.providers.base.Bool`.

use crate::dispatch::{JValue, Method};
use crate::Faker;

/// Booleans.
#[derive(Clone, Copy)]
pub struct Bool<'a> {
    faker: &'a Faker,
}

impl<'a> Bool<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Bool { faker }
    }

    /// `random().nextBoolean()`.
    pub fn bool(&self) -> bool {
        self.faker.random().next_boolean()
    }
}

pub(crate) static METHODS: &[Method] = &[Method::new("bool", &[], |f, _| Ok(JValue::Bool(Bool::new(f).bool())))];
