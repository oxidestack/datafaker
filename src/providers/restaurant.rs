//! `net.datafaker.providers.base.Restaurant`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Restaurants, from `restaurant.*`.
#[derive(Clone, Copy)]
pub struct Restaurant<'a> {
    faker: &'a Faker,
}

impl<'a> Restaurant<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Restaurant { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Restaurant, key)
    }

    /// `restaurant.name_prefix`, bothified with upper-case letters.
    pub fn name_prefix(&self) -> Result<String> {
        Ok(self.faker.bothify_with(&self.resolve("restaurant.name_prefix")?, true))
    }

    /// `restaurant.name_suffix`.
    pub fn name_suffix(&self) -> Result<String> {
        self.resolve("restaurant.name_suffix")
    }

    /// `restaurant.name`.
    pub fn name(&self) -> Result<String> {
        self.resolve("restaurant.name")
    }

    /// `restaurant.type`.
    pub fn r#type(&self) -> Result<String> {
        self.resolve("restaurant.type")
    }

    /// `restaurant.description`.
    pub fn description(&self) -> Result<String> {
        self.resolve("restaurant.description")
    }

    /// `restaurant.review`.
    pub fn review(&self) -> Result<String> {
        self.resolve("restaurant.review")
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("namePrefix", &[], |f, _| s(Restaurant::new(f).name_prefix())),
    Method::new("nameSuffix", &[], |f, _| s(Restaurant::new(f).name_suffix())),
    Method::new("name", &[], |f, _| s(Restaurant::new(f).name())),
    Method::new("type", &[], |f, _| s(Restaurant::new(f).r#type())),
    Method::new("description", &[], |f, _| s(Restaurant::new(f).description())),
    Method::new("review", &[], |f, _| s(Restaurant::new(f).review())),
];
