//! `net.datafaker.providers.base.Mbti`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Myers-Briggs types, from `mbti.*`.
#[derive(Clone, Copy)]
pub struct Mbti<'a> {
    faker: &'a Faker,
}

impl<'a> Mbti<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Mbti { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Mbti, key)
    }

    /// `mbti.<choice>.<field>`, `choice` resolved on the faker from `mbti.choice` for each call.
    fn field(&self, field: &str) -> Result<String> {
        let choice = self.faker.resolve("mbti.choice")?;
        self.resolve(&format!("mbti.{choice}.{field}"))
    }

    /// `mbti.<choice>.type`.
    pub fn r#type(&self) -> Result<String> {
        self.field("type")
    }

    /// `mbti.<choice>.name`.
    pub fn name(&self) -> Result<String> {
        self.field("name")
    }

    /// `mbti.<choice>.characteristic`.
    pub fn characteristic(&self) -> Result<String> {
        self.field("characteristic")
    }

    /// `mbti.<choice>.personage`.
    pub fn personage(&self) -> Result<String> {
        self.field("personage")
    }

    /// `mbti.<choice>.merit`.
    pub fn merit(&self) -> Result<String> {
        self.field("merit")
    }

    /// `mbti.<choice>.weakness`.
    pub fn weakness(&self) -> Result<String> {
        self.field("weakness")
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("type", &[], |f, _| s(Mbti::new(f).r#type())),
    Method::new("name", &[], |f, _| s(Mbti::new(f).name())),
    Method::new("characteristic", &[], |f, _| s(Mbti::new(f).characteristic())),
    Method::new("personage", &[], |f, _| s(Mbti::new(f).personage())),
    Method::new("merit", &[], |f, _| s(Mbti::new(f).merit())),
    Method::new("weakness", &[], |f, _| s(Mbti::new(f).weakness())),
];
