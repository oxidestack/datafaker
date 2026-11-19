//! `net.datafaker.providers.base.Domain`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{java_str, Method, P};
use crate::{Faker, Result};

/// Domain names around a given name, from `domain.*`.
#[derive(Clone, Copy)]
pub struct Domain<'a> {
    faker: &'a Faker,
}

impl<'a> Domain<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Domain { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Domain, key)
    }

    /// `name.top`.
    pub fn first_level_domain(&self, name: &str) -> Result<String> {
        Ok(format!("{name}.{}", self.resolve("domain.top")?))
    }

    /// `name.top.country`.
    pub fn second_level_domain(&self, name: &str) -> Result<String> {
        let top = self.resolve("domain.top")?;
        Ok(format!("{name}.{top}.{}", self.resolve("domain.country")?))
    }

    /// `prefix.name.top.country`.
    pub fn full_domain(&self, name: &str) -> Result<String> {
        let prefix = self.resolve("domain.prefix")?;
        let top = self.resolve("domain.top")?;
        Ok(format!("{prefix}.{name}.{top}.{}", self.resolve("domain.country")?))
    }

    /// `name.top`, with a prefix one time in three and a country suffix one time in two.
    pub fn valid_domain(&self, name: &str) -> Result<String> {
        let random = self.faker.random();
        let has_prefix = random.next_int_bounded(3)? == 1;
        let has_suffix = random.next_int_bounded(2)? == 1;
        let mut res = format!("{name}.{}", self.resolve("domain.top")?);
        if has_prefix {
            res = format!("{}.{res}", self.resolve("domain.prefix")?);
        }
        if has_suffix {
            res = format!("{res}.{}", self.resolve("domain.country")?);
        }
        Ok(res)
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("firstLevelDomain", &[P::String], |f, a| s(Domain::new(f).first_level_domain(java_str(&a[0])))).accepting_null(),
    Method::new("secondLevelDomain", &[P::String], |f, a| s(Domain::new(f).second_level_domain(java_str(&a[0])))).accepting_null(),
    Method::new("fullDomain", &[P::String], |f, a| s(Domain::new(f).full_domain(java_str(&a[0])))).accepting_null(),
    Method::new("validDomain", &[P::String], |f, a| s(Domain::new(f).valid_domain(java_str(&a[0])))).accepting_null(),
];
