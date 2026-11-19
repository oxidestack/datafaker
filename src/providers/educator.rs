//! `net.datafaker.providers.base.Educator`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Schools and courses, from `educator.*`.
#[derive(Clone, Copy)]
pub struct Educator<'a> {
    faker: &'a Faker,
}

impl<'a> Educator<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Educator { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Educator, key)
    }

    /// `educator.name` and `educator.tertiary.type`.
    pub fn university(&self) -> Result<String> {
        Ok(format!("{} {}", self.resolve("educator.name")?, self.resolve("educator.tertiary.type")?))
    }

    /// `educator.tertiary.degree.type` and `educator.tertiary.degree.subject`.
    pub fn course(&self) -> Result<String> {
        Ok(format!("{} {}", self.resolve("educator.tertiary.degree.type")?, self.resolve("educator.tertiary.degree.subject")?))
    }

    /// `educator.tertiary.degree.subject` and a numerified `educator.tertiary.degree.course_number`.
    pub fn subject_with_number(&self) -> Result<String> {
        let subject = self.resolve("educator.tertiary.degree.subject")?;
        Ok(format!("{subject} {}", self.faker.numerify(&self.resolve("educator.tertiary.degree.course_number")?)))
    }

    /// `educator.name` and `educator.secondary`.
    pub fn secondary_school(&self) -> Result<String> {
        Ok(format!("{} {}", self.resolve("educator.name")?, self.resolve("educator.secondary")?))
    }

    /// `educator.name` and ` Campus`.
    pub fn campus(&self) -> Result<String> {
        Ok(format!("{} Campus", self.resolve("educator.name")?))
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("university", &[], |f, _| s(Educator::new(f).university())),
    Method::new("course", &[], |f, _| s(Educator::new(f).course())),
    Method::new("subjectWithNumber", &[], |f, _| s(Educator::new(f).subject_with_number())),
    Method::new("secondarySchool", &[], |f, _| s(Educator::new(f).secondary_school())),
    Method::new("campus", &[], |f, _| s(Educator::new(f).campus())),
];
