//! `net.datafaker.providers.base.Tire`.

use super::{ProviderKind, resolve};
use crate::dispatch::{java_str, JValue, Method, P};
use crate::{Faker, Result};

/// Tires, from `tire.*`.
#[derive(Clone, Copy)]
pub struct Tire<'a> {
    faker: &'a Faker,
}

impl<'a> Tire<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Tire { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Tire, key)
    }

    /// `tire.code`.
    pub fn code(&self) -> Result<String> {
        self.resolve("tire.code")
    }

    /// `code(prefix)`: `prefix` and a code.
    pub fn code_with_prefix(&self, prefix: &str) -> Result<String> {
        Ok(format!("{prefix}{}", self.code()?))
    }

    /// `code(defaultPrefix)`: a code, prefixed with `P` when `default_prefix`.
    pub fn code_with_default_prefix(&self, default_prefix: bool) -> Result<String> {
        if default_prefix { self.code_with_prefix("P") } else { self.code() }
    }

    /// `tire.vehicle_type`.
    pub fn vehicle_type(&self) -> Result<String> {
        self.resolve("tire.vehicle_type")
    }

    /// `tire.width`.
    pub fn width(&self) -> Result<String> {
        self.resolve("tire.width")
    }

    /// `tire.aspect_ratio`.
    pub fn aspect_ratio(&self) -> Result<String> {
        self.resolve("tire.aspect_ratio")
    }

    /// `tire.construction`.
    pub fn construction(&self) -> Result<String> {
        self.resolve("tire.construction")
    }

    /// `tire.rim_size`.
    pub fn rim_size(&self) -> Result<String> {
        self.resolve("tire.rim_size")
    }

    /// `tire.load_index`.
    pub fn load_index(&self) -> Result<String> {
        self.resolve("tire.load_index")
    }

    /// `tire.speed_rating` (`speedrating()` upstream).
    pub fn speedrating(&self) -> Result<String> {
        self.resolve("tire.speed_rating")
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("code", &[], |f, _| s(Tire::new(f).code())),
    Method::new("code", &[P::String], |f, a| s(Tire::new(f).code_with_prefix(java_str(&a[0])))).accepting_null(),
    Method::new("code", &[P::Boolean], |f, a| s(Tire::new(f).code_with_default_prefix(a[0].as_bool()))),
    Method::new("vehicleType", &[], |f, _| s(Tire::new(f).vehicle_type())),
    Method::new("width", &[], |f, _| s(Tire::new(f).width())),
    Method::new("aspectRatio", &[], |f, _| s(Tire::new(f).aspect_ratio())),
    Method::new("construction", &[], |f, _| s(Tire::new(f).construction())),
    Method::new("rimSize", &[], |f, _| s(Tire::new(f).rim_size())),
    Method::new("loadIndex", &[], |f, _| s(Tire::new(f).load_index())),
    Method::new("speedrating", &[], |f, _| s(Tire::new(f).speedrating())),
];
