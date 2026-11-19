//! `net.datafaker.providers.base.Compass`, with `CompassPoint`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{JValue, Method, P};
use crate::{Faker, Result};

/// `Compass.CompassPoint`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompassPoint {
    /// `CARDINAL`.
    Cardinal,
    /// `ORDINAL`.
    Ordinal,
    /// `HALF_WIND`.
    HalfWind,
    /// `QUARTER_WIND`.
    QuarterWind,
}

impl CompassPoint {
    /// The constants in declaration order.
    pub const ALL: [CompassPoint; 4] = [CompassPoint::Cardinal, CompassPoint::Ordinal, CompassPoint::HalfWind, CompassPoint::QuarterWind];

    fn yaml_key(self) -> &'static str {
        ["cardinal", "ordinal", "half-wind", "quarter-wind"][self as usize]
    }
}

const CONSTANTS: &[&str] = &["CARDINAL", "ORDINAL", "HALF_WIND", "QUARTER_WIND"];

/// The state of the faker's cached `Compass`: the point `compassPoint(…)` chose last.
#[derive(Default)]
struct Chosen(Option<CompassPoint>);

/// Compass directions, abbreviations and azimuths, from `compass.*`.
///
/// Upstream caches one `Compass` per faker, so a point chosen with [`Compass::compass_point`] stays for every later
/// call on that faker (expressions included); here it is kept on the faker the same way.
#[derive(Clone, Copy)]
pub struct Compass<'a> {
    faker: &'a Faker,
}

impl<'a> Compass<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Compass { faker }
    }

    /// `compassPoint(compassPoint)`: chooses the point later calls use, and returns the provider.
    pub fn compass_point(&self, compass_point: CompassPoint) -> Compass<'a> {
        self.choose(Some(compass_point));
        *self
    }

    fn choose(&self, point: Option<CompassPoint>) {
        self.faker.with_provider_state(ProviderKind::Compass, |state: &mut Chosen| state.0 = point);
    }

    fn chosen(&self) -> Option<CompassPoint> {
        self.faker.with_provider_state(ProviderKind::Compass, |state: &mut Chosen| state.0)
    }

    fn get(&self, any: &str, field: &str) -> Result<String> {
        match self.chosen() {
            None => resolve(self.faker, ProviderKind::Compass, any),
            Some(point) => resolve(self.faker, ProviderKind::Compass, &format!("compass.{}.{field}", point.yaml_key())),
        }
    }

    /// A direction word (of the chosen point, or `compass.direction`).
    pub fn word(&self) -> Result<String> {
        self.get("compass.direction", "word")
    }

    /// A direction abbreviation.
    pub fn abbreviation(&self) -> Result<String> {
        self.get("compass.abbreviation", "abbreviation")
    }

    /// An azimuth.
    pub fn azimuth(&self) -> Result<String> {
        self.get("compass.azimuth", "azimuth")
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("compassPoint", &[P::Enum("CompassPoint", CONSTANTS)], |f, a| {
        // `compassPoint(null)` clears the choice: later calls pick from every point again.
        let point = match &a[0] {
            JValue::Enum(name) => CONSTANTS.iter().position(|c| c == name).map(|i| CompassPoint::ALL[i]),
            _ => None,
        };
        Compass::new(f).choose(point);
        Ok(JValue::Provider(ProviderKind::Compass))
    })
    .accepting_null(),
    Method::new("word", &[], |f, _| s(Compass::new(f).word())),
    Method::new("abbreviation", &[], |f, _| s(Compass::new(f).abbreviation())),
    Method::new("azimuth", &[], |f, _| s(Compass::new(f).azimuth())),
];
