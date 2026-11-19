//! `net.datafaker.providers.food.Tea`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Tea, from `tea.*`.
#[derive(Clone, Copy)]
pub struct Tea<'a> {
    faker: &'a Faker,
}

impl<'a> Tea<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Tea { faker }
    }

    /// `tea.variety.<type in lower case>`, the type drawn first.
    pub fn variety(&self) -> Result<String> {
        let kind = self.r#type()?.to_lowercase();
        resolve(self.faker, ProviderKind::Tea, &format!("tea.variety.{kind}"))
    }

    /// `tea.type`.
    pub fn r#type(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Tea, "tea.type")
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("variety", &[], |f, _| s(Tea::new(f).variety())),
    Method::new("type", &[], |f, _| s(Tea::new(f).r#type())),
];
