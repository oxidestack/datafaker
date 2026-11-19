//! `net.datafaker.providers.base.LanguageCode`.

use super::iso_codes::ISO_LANGUAGES;
use super::s;
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Language codes: the JDK's `Locale.getISOLanguages()`.
#[derive(Clone, Copy)]
pub struct LanguageCode<'a> {
    faker: &'a Faker,
}

impl<'a> LanguageCode<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        LanguageCode { faker }
    }

    /// One of `Locale.getISOLanguages()` (two letters).
    pub fn iso639(&self) -> Result<String> {
        self.faker.options().option(ISO_LANGUAGES).map(str::to_string)
    }
}

pub(crate) static METHODS: &[Method] = &[Method::new("iso639", &[], |f, _| s(LanguageCode::new(f).iso639()))];
