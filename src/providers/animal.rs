//! `net.datafaker.providers.base.Animal`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result, capitalize, java};

/// Animals, from `creature.animal.*`.
#[derive(Clone, Copy)]
pub struct Animal<'a> {
    faker: &'a Faker,
}

impl<'a> Animal<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Animal { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Animal, key)
    }

    /// `creature.animal.name`.
    pub fn name(&self) -> Result<String> {
        self.resolve("creature.animal.name")
    }

    /// `genus() + " " + species()`.
    pub fn scientific_name(&self) -> Result<String> {
        Ok(format!("{} {}", self.genus()?, self.species()?))
    }

    /// `creature.animal.genus` (resolved on the faker), capitalized.
    pub fn genus(&self) -> Result<String> {
        Ok(capitalize(&self.faker.resolve("creature.animal.genus")?))
    }

    /// `creature.animal.species`, lower-cased (`toLowerCase()` in an English JVM).
    pub fn species(&self) -> Result<String> {
        Ok(java::string_to_lower(&self.resolve("creature.animal.species")?, "en"))
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("name", &[], |f, _| s(Animal::new(f).name())),
    Method::new("scientificName", &[], |f, _| s(Animal::new(f).scientific_name())),
    Method::new("genus", &[], |f, _| s(Animal::new(f).genus())),
    Method::new("species", &[], |f, _| s(Animal::new(f).species())),
];
