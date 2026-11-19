//! `net.datafaker.providers.base.Computer`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Computers, from `computer.*`.
#[derive(Clone, Copy)]
pub struct Computer<'a> {
    faker: &'a Faker,
}

impl<'a> Computer<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Computer { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Computer, key)
    }

    /// `computer.type`.
    pub fn r#type(&self) -> Result<String> {
        self.resolve("computer.type")
    }

    /// `computer.platform`.
    pub fn platform(&self) -> Result<String> {
        self.resolve("computer.platform")
    }

    /// `computer.os.<linux|macos|windows>`, the family drawn with `options().option(…)`.
    pub fn operating_system(&self) -> Result<String> {
        let family = self.faker.options().option(&["linux", "macos", "windows"])?;
        self.resolve(&format!("computer.os.{family}"))
    }

    /// `computer.os.linux`.
    pub fn linux(&self) -> Result<String> {
        self.resolve("computer.os.linux")
    }

    /// `computer.os.macos`.
    pub fn macos(&self) -> Result<String> {
        self.resolve("computer.os.macos")
    }

    /// `computer.os.windows`.
    pub fn windows(&self) -> Result<String> {
        self.resolve("computer.os.windows")
    }

    /// `computer.brand`.
    pub fn brand(&self) -> Result<String> {
        self.resolve("computer.brand")
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("type", &[], |f, _| s(Computer::new(f).r#type())),
    Method::new("platform", &[], |f, _| s(Computer::new(f).platform())),
    Method::new("operatingSystem", &[], |f, _| s(Computer::new(f).operating_system())),
    Method::new("linux", &[], |f, _| s(Computer::new(f).linux())),
    Method::new("macos", &[], |f, _| s(Computer::new(f).macos())),
    Method::new("windows", &[], |f, _| s(Computer::new(f).windows())),
    Method::new("brand", &[], |f, _| s(Computer::new(f).brand())),
];
