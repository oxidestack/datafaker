//! `net.datafaker.providers.base.Color`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{Method, P};
use crate::{Faker, Result};

/// Colors: names from `color.name`, random hex codes.
#[derive(Clone, Copy)]
pub struct Color<'a> {
    faker: &'a Faker,
}

impl<'a> Color<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Color { faker }
    }

    /// `color.name`.
    pub fn name(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Color, "color.name")
    }

    /// `hex()`: `#` and six upper-case hex digits.
    pub fn hex(&self) -> String {
        self.hex_with(true)
    }

    /// `hex(includeHashSign)`.
    pub fn hex_with(&self, include_hash_sign: bool) -> String {
        let hex = self.faker.random().hex_with(6, true);
        if include_hash_sign { format!("#{hex}") } else { hex }
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("name", &[], |f, _| s(Color::new(f).name())),
    Method::new("hex", &[], |f, _| s(Ok(Color::new(f).hex()))),
    Method::new("hex", &[P::Boolean], |f, a| s(Ok(Color::new(f).hex_with(a[0].as_bool())))),
];
