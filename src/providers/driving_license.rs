//! `net.datafaker.providers.base.DrivingLicense`.

use super::s;
use crate::dispatch::{java_str, Method, P};
use crate::java::string_to_upper;
use crate::{Faker, Result};

/// US driving licence numbers.
#[derive(Clone, Copy)]
pub struct DrivingLicense<'a> {
    faker: &'a Faker,
}

impl<'a> DrivingLicense<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        DrivingLicense { faker }
    }

    /// `drivingLicense(stateAbbreviation)`: `driving_license.usa.<state>` bothified, upper-cased.
    pub fn driving_license(&self, state_abbreviation: &str) -> Result<String> {
        let pattern = self.faker.resolve(&format!("driving_license.usa.{state_abbreviation}"))?;
        Ok(string_to_upper(&self.faker.bothify(&pattern), ""))
    }
}

pub(crate) static METHODS: &[Method] = &[Method::new("drivingLicense", &[P::String], |f, a| s(DrivingLicense::new(f).driving_license(java_str(&a[0])))).accepting_null()];
