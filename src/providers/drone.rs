//! `net.datafaker.providers.base.Drone`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Drone specifications, from `drone.*` (numeric ones numerified).
#[derive(Clone, Copy)]
pub struct Drone<'a> {
    faker: &'a Faker,
}

impl<'a> Drone<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Drone { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Drone, key)
    }

    /// `drone.name`.
    pub fn name(&self) -> Result<String> {
        self.resolve("drone.name")
    }

    /// `drone.weight`, numerified.
    pub fn weight(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.weight")?))
    }

    /// `drone.max_ascent_speed`, numerified.
    pub fn max_ascent_speed(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.max_ascent_speed")?))
    }

    /// `drone.max_descent_speed`, numerified.
    pub fn max_descent_speed(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.max_descent_speed")?))
    }

    /// `drone.flight_time`, numerified.
    pub fn flight_time(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.flight_time")?))
    }

    /// `drone.max_altitude`, numerified.
    pub fn max_altitude(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.max_altitude")?))
    }

    /// `drone.max_flight_distance`, numerified.
    pub fn max_flight_distance(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.max_flight_distance")?))
    }

    /// `drone.max_speed`, numerified.
    pub fn max_speed(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.max_speed")?))
    }

    /// `drone.max_wind_resistance`, numerified.
    pub fn max_wind_resistance(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.max_wind_resistance")?))
    }

    /// `drone.max_angular_velocity`, numerified.
    pub fn max_angular_velocity(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.max_angular_velocity")?))
    }

    /// `drone.max_tilt_angle`, numerified.
    pub fn max_tilt_angle(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.max_tilt_angle")?))
    }

    /// `drone.operating_temperature`, numerified.
    pub fn operating_temperature(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.operating_temperature")?))
    }

    /// `drone.battery_capacity`, numerified.
    pub fn battery_capacity(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.battery_capacity")?))
    }

    /// `drone.battery_voltage`, numerified.
    pub fn battery_voltage(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.battery_voltage")?))
    }

    /// `drone.battery_type`.
    pub fn battery_type(&self) -> Result<String> {
        self.resolve("drone.battery_type")
    }

    /// `drone.battery_weight`, numerified.
    pub fn battery_weight(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.battery_weight")?))
    }

    /// `drone.charging_temperature`, numerified.
    pub fn charging_temperature(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.charging_temperature")?))
    }

    /// `drone.max_charging_power`, numerified.
    pub fn max_charging_power(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.max_charging_power")?))
    }

    /// `drone.iso`.
    pub fn iso(&self) -> Result<String> {
        self.resolve("drone.iso")
    }

    /// `drone.max_resolution`, numerified.
    pub fn max_resolution(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("drone.max_resolution")?))
    }

    /// `drone.photo_format`.
    pub fn photo_format(&self) -> Result<String> {
        self.resolve("drone.photo_format")
    }

    /// `drone.video_format`.
    pub fn video_format(&self) -> Result<String> {
        self.resolve("drone.video_format")
    }

    /// `drone.max_shutter_speed`.
    pub fn max_shutter_speed(&self) -> Result<String> {
        self.resolve("drone.max_shutter_speed")
    }

    /// `drone.min_shutter_speed`.
    pub fn min_shutter_speed(&self) -> Result<String> {
        self.resolve("drone.min_shutter_speed")
    }

    /// `drone.shutter_speed_units` (`@Deterministic` upstream).
    pub fn shutter_speed_units(&self) -> Result<String> {
        self.resolve("drone.shutter_speed_units")
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("name", &[], |f, _| s(Drone::new(f).name())),
    Method::new("weight", &[], |f, _| s(Drone::new(f).weight())),
    Method::new("maxAscentSpeed", &[], |f, _| s(Drone::new(f).max_ascent_speed())),
    Method::new("maxDescentSpeed", &[], |f, _| s(Drone::new(f).max_descent_speed())),
    Method::new("flightTime", &[], |f, _| s(Drone::new(f).flight_time())),
    Method::new("maxAltitude", &[], |f, _| s(Drone::new(f).max_altitude())),
    Method::new("maxFlightDistance", &[], |f, _| s(Drone::new(f).max_flight_distance())),
    Method::new("maxSpeed", &[], |f, _| s(Drone::new(f).max_speed())),
    Method::new("maxWindResistance", &[], |f, _| s(Drone::new(f).max_wind_resistance())),
    Method::new("maxAngularVelocity", &[], |f, _| s(Drone::new(f).max_angular_velocity())),
    Method::new("maxTiltAngle", &[], |f, _| s(Drone::new(f).max_tilt_angle())),
    Method::new("operatingTemperature", &[], |f, _| s(Drone::new(f).operating_temperature())),
    Method::new("batteryCapacity", &[], |f, _| s(Drone::new(f).battery_capacity())),
    Method::new("batteryVoltage", &[], |f, _| s(Drone::new(f).battery_voltage())),
    Method::new("batteryType", &[], |f, _| s(Drone::new(f).battery_type())),
    Method::new("batteryWeight", &[], |f, _| s(Drone::new(f).battery_weight())),
    Method::new("chargingTemperature", &[], |f, _| s(Drone::new(f).charging_temperature())),
    Method::new("maxChargingPower", &[], |f, _| s(Drone::new(f).max_charging_power())),
    Method::new("iso", &[], |f, _| s(Drone::new(f).iso())),
    Method::new("maxResolution", &[], |f, _| s(Drone::new(f).max_resolution())),
    Method::new("photoFormat", &[], |f, _| s(Drone::new(f).photo_format())),
    Method::new("videoFormat", &[], |f, _| s(Drone::new(f).video_format())),
    Method::new("maxShutterSpeed", &[], |f, _| s(Drone::new(f).max_shutter_speed())),
    Method::new("minShutterSpeed", &[], |f, _| s(Drone::new(f).min_shutter_speed())),
    Method::new("shutterSpeedUnits", &[], |f, _| s(Drone::new(f).shutter_speed_units())),
];
