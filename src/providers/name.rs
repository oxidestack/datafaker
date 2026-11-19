//! `net.datafaker.providers.base.Name`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Names of people, from `name.*` in the locale files.
#[derive(Clone, Copy)]
pub struct Name<'a> {
    faker: &'a Faker,
}

impl<'a> Name<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Name { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Name, key)
    }

    /// A full name, with a prefix or a suffix now and then.
    pub fn name(&self) -> Result<String> {
        self.resolve("name.name")
    }

    /// `name.name_with_middle`.
    pub fn name_with_middle(&self) -> Result<String> {
        self.resolve("name.name_with_middle")
    }

    /// `fullName()`: the same as [`Self::name`].
    pub fn full_name(&self) -> Result<String> {
        self.name()
    }

    /// `name.first_name`.
    pub fn first_name(&self) -> Result<String> {
        self.resolve("name.first_name")
    }

    /// `name.female_first_name`.
    pub fn female_first_name(&self) -> Result<String> {
        self.resolve("name.female_first_name")
    }

    /// `name.male_first_name`.
    pub fn male_first_name(&self) -> Result<String> {
        self.resolve("name.male_first_name")
    }

    /// `name.last_name`.
    pub fn last_name(&self) -> Result<String> {
        self.resolve("name.last_name")
    }

    /// `name.prefix`.
    pub fn prefix(&self) -> Result<String> {
        self.resolve("name.prefix")
    }

    /// `name.suffix`.
    pub fn suffix(&self) -> Result<String> {
        self.resolve("name.suffix")
    }

    /// A job title: descriptor, level and job.
    pub fn title(&self) -> Result<String> {
        let descriptor = self.resolve("name.title.descriptor")?;
        let level = self.resolve("name.title.level")?;
        let job = self.resolve("name.title.job")?;
        Ok([descriptor, level, job].join(" "))
    }

    /// Deprecated upstream: `credentials().username()`.
    pub fn username(&self) -> Result<String> {
        self.faker.credentials().username()
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("name", &[], |f, _| s(f.name().name())),
    Method::new("nameWithMiddle", &[], |f, _| s(f.name().name_with_middle())),
    Method::new("fullName", &[], |f, _| s(f.name().full_name())),
    Method::new("firstName", &[], |f, _| s(f.name().first_name())),
    Method::new("femaleFirstName", &[], |f, _| s(f.name().female_first_name())),
    Method::new("malefirstName", &[], |f, _| s(f.name().male_first_name())),
    Method::new("maleFirstName", &[], |f, _| s(f.name().male_first_name())),
    Method::new("lastName", &[], |f, _| s(f.name().last_name())),
    Method::new("prefix", &[], |f, _| s(f.name().prefix())),
    Method::new("suffix", &[], |f, _| s(f.name().suffix())),
    Method::new("title", &[], |f, _| s(f.name().title())),
    Method::new("username", &[], |f, _| s(f.name().username())),
];
