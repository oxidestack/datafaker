//! `net.datafaker.providers.base.Relationship`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Family relationships, from `relationship.*`.
#[derive(Clone, Copy)]
pub struct Relationship<'a> {
    faker: &'a Faker,
}

impl<'a> Relationship<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Relationship { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Relationship, key)
    }

    /// `Kind`'s expressions, in declaration order.
    const KINDS: [&'static str; 6] = [
        "relationship.familial.direct",
        "relationship.familial.extended",
        "relationship.in_law",
        "relationship.spouse",
        "relationship.parent",
        "relationship.sibling",
    ];

    /// `relationship.familial.direct`.
    pub fn direct(&self) -> Result<String> {
        self.resolve(Self::KINDS[0])
    }

    /// `relationship.familial.extended`.
    pub fn extended(&self) -> Result<String> {
        self.resolve(Self::KINDS[1])
    }

    /// `relationship.in_law`.
    pub fn in_law(&self) -> Result<String> {
        self.resolve(Self::KINDS[2])
    }

    /// `relationship.spouse`.
    pub fn spouse(&self) -> Result<String> {
        self.resolve(Self::KINDS[3])
    }

    /// `relationship.parent`.
    pub fn parent(&self) -> Result<String> {
        self.resolve(Self::KINDS[4])
    }

    /// `relationship.sibling`.
    pub fn sibling(&self) -> Result<String> {
        self.resolve(Self::KINDS[5])
    }

    /// One of the six, the kind drawn with `options().option(Kind.class)` (`random().nextEnum`).
    pub fn any(&self) -> Result<String> {
        let kind = self.faker.random().next_enum(Self::KINDS.len())?;
        self.resolve(Self::KINDS[kind])
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("direct", &[], |f, _| s(Relationship::new(f).direct())),
    Method::new("extended", &[], |f, _| s(Relationship::new(f).extended())),
    Method::new("inLaw", &[], |f, _| s(Relationship::new(f).in_law())),
    Method::new("spouse", &[], |f, _| s(Relationship::new(f).spouse())),
    Method::new("parent", &[], |f, _| s(Relationship::new(f).parent())),
    Method::new("sibling", &[], |f, _| s(Relationship::new(f).sibling())),
    Method::new("any", &[], |f, _| s(Relationship::new(f).any())),
];
