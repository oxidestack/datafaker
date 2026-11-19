//! `net.datafaker.providers.entertainment.Avatar`.

use super::s;
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Robohash avatar URLs.
#[derive(Clone, Copy)]
pub struct Avatar<'a> {
    faker: &'a Faker,
}

impl<'a> Avatar<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Avatar { faker }
    }

    /// `https://robohash.org/` followed by `regexify("[a-z]{8}")` and `.png`.
    pub fn image(&self) -> Result<String> {
        Ok(format!("https://robohash.org/{}.png", self.faker.regexify("[a-z]{8}")?))
    }
}

pub(crate) static METHODS: &[Method] = &[Method::new("image", &[], |f, _| s(Avatar::new(f).image()))];
