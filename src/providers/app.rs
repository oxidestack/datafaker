//! `net.datafaker.providers.base.App`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Apps, from `app.*`.
#[derive(Clone, Copy)]
pub struct App<'a> {
    faker: &'a Faker,
}

impl<'a> App<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        App { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::App, key)
    }

    /// `app.name`.
    pub fn name(&self) -> Result<String> {
        self.resolve("app.name")
    }

    /// `app.version` (resolved on the faker), numerified.
    pub fn version(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.faker.resolve("app.version")?))
    }

    /// `app.author`.
    pub fn author(&self) -> Result<String> {
        self.resolve("app.author")
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("name", &[], |f, _| s(App::new(f).name())),
    Method::new("version", &[], |f, _| s(App::new(f).version())),
    Method::new("author", &[], |f, _| s(App::new(f).author())),
];
