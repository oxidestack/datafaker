//! `net.datafaker.providers.base.Vehicle`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{java_str, JValue, Method, P};
use crate::java::string_to_upper;
use crate::{Faker, Result};

const VIN_REGEX: &str = r"([A-HJ-NPR-Z0-9]){3}[A-HJ-NPR-Z0-9]{5}[A-HJ-NPR-Z0-9]{1}[A-HJ-NPR-Z0-9]{1}[A-HJ-NPR-Z0-0]{1}[A-HJ-NPR-Z0-9]{1}\d{5}";

/// Vehicles: VINs, makes and models, options, licence plates, from `vehicle.*`.
#[derive(Clone, Copy)]
pub struct Vehicle<'a> {
    faker: &'a Faker,
}

impl<'a> Vehicle<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Vehicle { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Vehicle, key)
    }

    /// A VIN-shaped string (upstream's regex, its `[A-HJ-NPR-Z0-0]` included).
    pub fn vin(&self) -> Result<String> {
        self.faker.regexify(VIN_REGEX)
    }

    /// `vehicle.manufacture`.
    pub fn manufacturer(&self) -> Result<String> {
        self.resolve("vehicle.manufacture")
    }

    /// `vehicle.makes`.
    pub fn make(&self) -> Result<String> {
        self.resolve("vehicle.makes")
    }

    /// A model of a random make.
    pub fn model(&self) -> Result<String> {
        self.model_of(&self.make()?)
    }

    /// `model(make)`: `vehicle.models_by_make.<make>`.
    pub fn model_of(&self, make: &str) -> Result<String> {
        self.resolve(&format!("vehicle.models_by_make.{make}"))
    }

    /// A make and one of its models.
    pub fn make_and_model(&self) -> Result<String> {
        let make = self.make()?;
        let model = self.model_of(&make)?;
        Ok(format!("{make} {model}"))
    }

    /// `vehicle.styles`.
    pub fn style(&self) -> Result<String> {
        self.resolve("vehicle.styles")
    }

    /// `vehicle.colors`.
    pub fn color(&self) -> Result<String> {
        self.resolve("vehicle.colors")
    }

    /// `vehicle.upholstery_colors`.
    pub fn upholstery_color(&self) -> Result<String> {
        self.resolve("vehicle.upholstery_colors")
    }

    /// `vehicle.upholstery_fabrics`.
    pub fn upholstery_fabric(&self) -> Result<String> {
        self.resolve("vehicle.upholstery_fabrics")
    }

    /// `vehicle.upholsteries`.
    pub fn upholstery(&self) -> Result<String> {
        self.resolve("vehicle.upholsteries")
    }

    /// `vehicle.transmissions`.
    pub fn transmission(&self) -> Result<String> {
        self.resolve("vehicle.transmissions")
    }

    /// `vehicle.drive_types`.
    pub fn drive_type(&self) -> Result<String> {
        self.resolve("vehicle.drive_types")
    }

    /// `vehicle.fuel_types`.
    pub fn fuel_type(&self) -> Result<String> {
        self.resolve("vehicle.fuel_types")
    }

    /// `vehicle.car_types`.
    pub fn car_type(&self) -> Result<String> {
        self.resolve("vehicle.car_types")
    }

    /// An engine size and a cylinder layout.
    pub fn engine(&self) -> Result<String> {
        let size = self.resolve("vehicle.engine_sizes")?;
        Ok(format!("{size} {}", self.resolve("vehicle.cylinder_engine")?))
    }

    /// `carOptions()`: five to nine options.
    pub fn car_options(&self) -> Result<Vec<String>> {
        self.car_options_between(5, 10)
    }

    /// `carOptions(min, max)`: `numberBetween(min, max)` options of `vehicle.car_options`, repeats possible.
    pub fn car_options_between(&self, min: i32, max: i32) -> Result<Vec<String>> {
        self.list(min, max, "vehicle.car_options")
    }

    /// `standardSpecs()`: five to nine specs.
    pub fn standard_specs(&self) -> Result<Vec<String>> {
        self.standard_specs_between(5, 10)
    }

    /// `standardSpecs(min, max)`.
    pub fn standard_specs_between(&self, min: i32, max: i32) -> Result<Vec<String>> {
        self.list(min, max, "vehicle.standard_specs")
    }

    fn list(&self, min: i32, max: i32, key: &str) -> Result<Vec<String>> {
        let size = self.faker.number().number_between(min, max);
        (0..size.max(0)).map(|_| self.faker.resolve(key)).collect()
    }

    /// `vehicle.doors`.
    pub fn doors(&self) -> Result<String> {
        self.resolve("vehicle.doors")
    }

    /// `vehicle.license_plate`, bothified and upper-cased.
    pub fn license_plate(&self) -> Result<String> {
        Ok(string_to_upper(&self.faker.bothify(&self.faker.resolve("vehicle.license_plate")?), ""))
    }

    /// `licensePlate(stateAbbreviation)`: `vehicle.license_plate_by_state.<state>`, bothified and upper-cased;
    /// `None` (Java's `null`) for an empty state.
    pub fn license_plate_of(&self, state_abbreviation: &str) -> Result<Option<String>> {
        if state_abbreviation.is_empty() {
            return Ok(None);
        }
        let plates = self.resolve(&format!("vehicle.license_plate_by_state.{state_abbreviation}"))?;
        Ok(Some(string_to_upper(&self.faker.bothify(&plates), "")))
    }
}

fn list(r: Result<Vec<String>>) -> Result<JValue> {
    r.map(JValue::from)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("vin", &[], |f, _| s(Vehicle::new(f).vin())),
    Method::new("manufacturer", &[], |f, _| s(Vehicle::new(f).manufacturer())),
    Method::new("make", &[], |f, _| s(Vehicle::new(f).make())),
    Method::new("model", &[], |f, _| s(Vehicle::new(f).model())),
    Method::new("model", &[P::String], |f, a| s(Vehicle::new(f).model_of(java_str(&a[0])))).accepting_null(),
    Method::new("makeAndModel", &[], |f, _| s(Vehicle::new(f).make_and_model())),
    Method::new("style", &[], |f, _| s(Vehicle::new(f).style())),
    Method::new("color", &[], |f, _| s(Vehicle::new(f).color())),
    Method::new("upholsteryColor", &[], |f, _| s(Vehicle::new(f).upholstery_color())),
    Method::new("upholsteryFabric", &[], |f, _| s(Vehicle::new(f).upholstery_fabric())),
    Method::new("upholstery", &[], |f, _| s(Vehicle::new(f).upholstery())),
    Method::new("transmission", &[], |f, _| s(Vehicle::new(f).transmission())),
    Method::new("driveType", &[], |f, _| s(Vehicle::new(f).drive_type())),
    Method::new("fuelType", &[], |f, _| s(Vehicle::new(f).fuel_type())),
    Method::new("carType", &[], |f, _| s(Vehicle::new(f).car_type())),
    Method::new("engine", &[], |f, _| s(Vehicle::new(f).engine())),
    Method::new("carOptions", &[], |f, _| list(Vehicle::new(f).car_options())),
    Method::new("carOptions", &[P::Int, P::Int], |f, a| list(Vehicle::new(f).car_options_between(a[0].as_int(), a[1].as_int()))),
    Method::new("standardSpecs", &[], |f, _| list(Vehicle::new(f).standard_specs())),
    Method::new("standardSpecs", &[P::Int, P::Int], |f, a| list(Vehicle::new(f).standard_specs_between(a[0].as_int(), a[1].as_int()))),
    Method::new("doors", &[], |f, _| s(Vehicle::new(f).doors())),
    Method::new("licensePlate", &[], |f, _| s(Vehicle::new(f).license_plate())),
    Method::new("licensePlate", &[P::String], |f, a| {
        Vehicle::new(f).license_plate_of(java_str(&a[0])).map(|o| o.map_or(JValue::Null, JValue::Str))
    })
    .accepting_null(),
];
