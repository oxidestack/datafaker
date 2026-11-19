//! `net.datafaker.providers.base.Space`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Space, from `space.*`.
#[derive(Clone, Copy)]
pub struct Space<'a> {
    faker: &'a Faker,
}

impl<'a> Space<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Space { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Space, key)
    }

    /// `space.planet`.
    pub fn planet(&self) -> Result<String> {
        self.resolve("space.planet")
    }

    /// `space.moon`.
    pub fn moon(&self) -> Result<String> {
        self.resolve("space.moon")
    }

    /// `space.galaxy`.
    pub fn galaxy(&self) -> Result<String> {
        self.resolve("space.galaxy")
    }

    /// `space.nebula`.
    pub fn nebula(&self) -> Result<String> {
        self.resolve("space.nebula")
    }

    /// `space.star_cluster`.
    pub fn star_cluster(&self) -> Result<String> {
        self.resolve("space.star_cluster")
    }

    /// `space.constellation`.
    pub fn constellation(&self) -> Result<String> {
        self.resolve("space.constellation")
    }

    /// `space.star`.
    pub fn star(&self) -> Result<String> {
        self.resolve("space.star")
    }

    /// `space.agency`.
    pub fn agency(&self) -> Result<String> {
        self.resolve("space.agency")
    }

    /// `space.agency_abv`.
    pub fn agency_abbreviation(&self) -> Result<String> {
        self.resolve("space.agency_abv")
    }

    /// `space.nasa_space_craft`.
    pub fn nasa_space_craft(&self) -> Result<String> {
        self.resolve("space.nasa_space_craft")
    }

    /// `space.company`.
    pub fn company(&self) -> Result<String> {
        self.resolve("space.company")
    }

    /// `numberBetween(10, 100) + ' ' + resolve("space.distance_measurement")`: in Java an `int` plus a `char` is an
    /// `int`, so the number comes out 32 higher and without the space (`"119light years"`).
    pub fn distance_measurement(&self) -> Result<String> {
        let n = self.faker.number().number_between(10, 100) + ' ' as i32;
        Ok(format!("{n}{}", self.resolve("space.distance_measurement")?))
    }

    /// `space.meteorite`.
    pub fn meteorite(&self) -> Result<String> {
        self.resolve("space.meteorite")
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("planet", &[], |f, _| s(Space::new(f).planet())),
    Method::new("moon", &[], |f, _| s(Space::new(f).moon())),
    Method::new("galaxy", &[], |f, _| s(Space::new(f).galaxy())),
    Method::new("nebula", &[], |f, _| s(Space::new(f).nebula())),
    Method::new("starCluster", &[], |f, _| s(Space::new(f).star_cluster())),
    Method::new("constellation", &[], |f, _| s(Space::new(f).constellation())),
    Method::new("star", &[], |f, _| s(Space::new(f).star())),
    Method::new("agency", &[], |f, _| s(Space::new(f).agency())),
    Method::new("agencyAbbreviation", &[], |f, _| s(Space::new(f).agency_abbreviation())),
    Method::new("nasaSpaceCraft", &[], |f, _| s(Space::new(f).nasa_space_craft())),
    Method::new("company", &[], |f, _| s(Space::new(f).company())),
    Method::new("distanceMeasurement", &[], |f, _| s(Space::new(f).distance_measurement())),
    Method::new("meteorite", &[], |f, _| s(Space::new(f).meteorite())),
];
