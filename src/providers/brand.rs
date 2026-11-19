//! `net.datafaker.providers.base.Brand`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Brands, from `brand.*` and `vehicle.makes`.
#[derive(Clone, Copy)]
pub struct Brand<'a> {
    faker: &'a Faker,
}

impl<'a> Brand<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Brand { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Brand, key)
    }

    /// `brand.sport`.
    pub fn sport(&self) -> Result<String> {
        self.resolve("brand.sport")
    }

    /// `vehicle.makes`, resolved on the `Vehicle` provider (`faker.vehicle().resolve(…)`).
    pub fn car(&self) -> Result<String> {
        super::resolve(self.faker, super::ProviderKind::Vehicle, "vehicle.makes")
    }

    /// `brand.watch`.
    pub fn watch(&self) -> Result<String> {
        self.resolve("brand.watch")
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("sport", &[], |f, _| s(Brand::new(f).sport())),
    Method::new("car", &[], |f, _| s(Brand::new(f).car())),
    Method::new("watch", &[], |f, _| s(Brand::new(f).watch())),
];
