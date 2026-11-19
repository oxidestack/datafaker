//! `net.datafaker.providers.food.Cheese`.

use std::fmt;

use super::{ProviderKind, resolve, s};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// `Cheese.Wedge`, a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wedge {
    /// The cheese's name.
    pub name: String,
    /// Its type.
    pub r#type: String,
    /// Its producer.
    pub producer: String,
    /// Its texture.
    pub texture: String,
    /// Its color.
    pub color: String,
    /// The milk it is made of.
    pub milk: String,
    /// Its rind.
    pub rind: String,
    /// Whether the rind is edible.
    pub rind_edibility: String,
    /// How it is packaged.
    pub packaging: String,
}

impl fmt::Display for Wedge {
    /// The record's `toString()`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Wedge[name={}, type={}, producer={}, texture={}, color={}, milk={}, rind={}, rindEdibility={}, packaging={}]",
            self.name, self.r#type, self.producer, self.texture, self.color, self.milk, self.rind, self.rind_edibility, self.packaging
        )
    }
}

/// Cheese, from `cheese.*`.
#[derive(Clone, Copy)]
pub struct Cheese<'a> {
    faker: &'a Faker,
}

impl<'a> Cheese<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Cheese { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Cheese, key)
    }

    /// `cheese.type`.
    pub fn r#type(&self) -> Result<String> {
        self.resolve("cheese.type")
    }

    /// `cheese.texture`.
    pub fn texture(&self) -> Result<String> {
        self.resolve("cheese.texture")
    }

    /// `cheese.milk`.
    pub fn milk(&self) -> Result<String> {
        self.resolve("cheese.milk")
    }

    /// `cheese.color`.
    pub fn color(&self) -> Result<String> {
        self.resolve("cheese.color")
    }

    /// `cheese.name`.
    pub fn name(&self) -> Result<String> {
        self.resolve("cheese.name")
    }

    /// `cheese.producer`.
    pub fn producer(&self) -> Result<String> {
        self.resolve("cheese.producer")
    }

    /// `cheese.rind`.
    pub fn rind(&self) -> Result<String> {
        self.resolve("cheese.rind")
    }

    /// `cheese.rind_edibility`.
    pub fn rind_edibility(&self) -> Result<String> {
        self.resolve("cheese.rind_edibility")
    }

    /// `cheese.packaging`.
    pub fn packaging(&self) -> Result<String> {
        self.resolve("cheese.packaging")
    }

    /// A wedge, its fields drawn in the record constructor's argument order.
    pub fn wedge(&self) -> Result<Wedge> {
        Ok(Wedge {
            name: self.name()?,
            r#type: self.r#type()?,
            producer: self.producer()?,
            texture: self.texture()?,
            color: self.color()?,
            milk: self.milk()?,
            rind: self.rind()?,
            rind_edibility: self.rind_edibility()?,
            packaging: self.packaging()?,
        })
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("type", &[], |f, _| s(Cheese::new(f).r#type())),
    Method::new("texture", &[], |f, _| s(Cheese::new(f).texture())),
    Method::new("milk", &[], |f, _| s(Cheese::new(f).milk())),
    Method::new("color", &[], |f, _| s(Cheese::new(f).color())),
    Method::new("name", &[], |f, _| s(Cheese::new(f).name())),
    Method::new("producer", &[], |f, _| s(Cheese::new(f).producer())),
    Method::new("rind", &[], |f, _| s(Cheese::new(f).rind())),
    Method::new("rindEdibility", &[], |f, _| s(Cheese::new(f).rind_edibility())),
    Method::new("packaging", &[], |f, _| s(Cheese::new(f).packaging())),
    // A record; expressions only print it.
    Method::new("wedge", &[], |f, _| Ok(JValue::Str(Cheese::new(f).wedge()?.to_string()))),
];
