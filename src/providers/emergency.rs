//! `net.datafaker.providers.base.Emergency`.

use std::fmt;

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Emergencies, from `emergency.*`.
#[derive(Clone, Copy)]
pub struct Emergency<'a> {
    faker: &'a Faker,
}

/// `Emergency.EmergencyCase`, a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmergencyCase {
    /// What the emergency is.
    pub nature: String,
    /// Where it is.
    pub location: String,
    /// What to do.
    pub instruction: String,
    /// Whether it is a drill.
    pub dry_run: bool,
}

impl fmt::Display for EmergencyCase {
    /// The record's `toString()`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EmergencyCase[nature={}, location={}, instruction={}, dryRun={}]", self.nature, self.location, self.instruction, self.dry_run)
    }
}

impl<'a> Emergency<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Emergency { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Emergency, key)
    }

    /// `emergency.nature`.
    pub fn nature(&self) -> Result<String> {
        self.resolve("emergency.nature")
    }

    /// `emergency.location`.
    pub fn location(&self) -> Result<String> {
        self.resolve("emergency.location")
    }

    /// `emergency.instruction`.
    pub fn instruction(&self) -> Result<String> {
        self.resolve("emergency.instruction")
    }

    /// `random().nextBoolean()`.
    pub fn dry_run(&self) -> bool {
        self.faker.random().next_boolean()
    }

    /// A nature, a location, an instruction and a dry-run flag, drawn in that order.
    pub fn emergency_case(&self) -> Result<EmergencyCase> {
        Ok(EmergencyCase { nature: self.nature()?, location: self.location()?, instruction: self.instruction()?, dry_run: self.dry_run() })
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("nature", &[], |f, _| s(Emergency::new(f).nature())),
    Method::new("location", &[], |f, _| s(Emergency::new(f).location())),
    Method::new("instruction", &[], |f, _| s(Emergency::new(f).instruction())),
    Method::new("dryRun", &[], |f, _| Ok(JValue::Bool(Emergency::new(f).dry_run()))),
    // A record; expressions only print it.
    Method::new("emergencyCase", &[], |f, _| Ok(JValue::Str(Emergency::new(f).emergency_case()?.to_string()))),
];
