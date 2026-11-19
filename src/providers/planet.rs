//! `net.datafaker.providers.base.Planet`, with `Planet.PlanetName`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{JValue, Method, P};
use crate::{Faker, Result};

/// `Planet.PlanetName`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetName {
    /// `JUPITER`.
    Jupiter,
    /// `MARS`.
    Mars,
    /// `VENUS`.
    Venus,
    /// `URANUS`.
    Uranus,
    /// `MERCURY`.
    Mercury,
    /// `NEPTUNE`.
    Neptune,
    /// `EARTH`.
    Earth,
    /// `SATURN`.
    Saturn,
    /// `PLUTO`.
    Pluto,
}

impl PlanetName {
    /// The constants in declaration order.
    pub const ALL: [PlanetName; 9] = [
        PlanetName::Jupiter,
        PlanetName::Mars,
        PlanetName::Venus,
        PlanetName::Uranus,
        PlanetName::Mercury,
        PlanetName::Neptune,
        PlanetName::Earth,
        PlanetName::Saturn,
        PlanetName::Pluto,
    ];

    /// `getName()`: the YAML key.
    pub fn get_name(self) -> &'static str {
        ["jupiter", "mars", "venus", "uranus", "mercury", "neptune", "earth", "saturn", "pluto"][self as usize]
    }
}

const CONSTANTS: &[&str] = &["JUPITER", "MARS", "VENUS", "URANUS", "MERCURY", "NEPTUNE", "EARTH", "SATURN", "PLUTO"];
const PLANET_NAME: P = P::Enum("PlanetName", CONSTANTS);

/// Planets of the solar system and their physical data, from `planet.*`. Each no-argument method draws a planet
/// (`options().option(PlanetName.class)`); as upstream, `lengthOfDay()` returns a mean distance from the sun.
#[derive(Clone, Copy)]
pub struct Planet<'a> {
    faker: &'a Faker,
}

impl<'a> Planet<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Planet { faker }
    }

    fn random_planet(&self) -> Result<PlanetName> {
        Ok(PlanetName::ALL[self.faker.random().next_enum(PlanetName::ALL.len())?])
    }

    fn of(&self, field: &str, planet: PlanetName) -> Result<String> {
        resolve(self.faker, ProviderKind::Planet, &format!("planet.{field}.{}", planet.get_name()))
    }

    /// `planet.name`.
    pub fn name(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Planet, "planet.name")
    }

    /// `planet.symbol` of a random planet.
    pub fn symbol(&self) -> Result<String> {
        self.symbol_of(self.random_planet()?)
    }

    /// `symbol(planetName)`: `planet.symbol.<planet>`.
    pub fn symbol_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("symbol", planet_name)
    }

    /// `planet.classification` of a random planet.
    pub fn classification(&self) -> Result<String> {
        self.classification_of(self.random_planet()?)
    }

    /// `classification(planetName)`: `planet.classification.<planet>`.
    pub fn classification_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("classification", planet_name)
    }

    /// `planet.discovery_year` of a random planet.
    pub fn discovery_year(&self) -> Result<String> {
        self.discovery_year_of(self.random_planet()?)
    }

    /// `discoveryYear(planetName)`: `planet.discovery_year.<planet>`.
    pub fn discovery_year_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("discovery_year", planet_name)
    }

    /// `planet.mean_distance_from_the_sun` of a random planet.
    pub fn mean_distance_from_the_sun(&self) -> Result<String> {
        self.mean_distance_from_the_sun_of(self.random_planet()?)
    }

    /// `meanDistanceFromTheSun(planetName)`: `planet.mean_distance_from_the_sun.<planet>`.
    pub fn mean_distance_from_the_sun_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("mean_distance_from_the_sun", planet_name)
    }

    /// `planet.mean_distance_from_the_sun` of a random planet (upstream calls `meanDistanceFromTheSun` here).
    pub fn length_of_day(&self) -> Result<String> {
        self.mean_distance_from_the_sun_of(self.random_planet()?)
    }

    /// `lengthOfDay(planetName)`: `planet.length_of_day.<planet>`.
    pub fn length_of_day_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("length_of_day", planet_name)
    }

    /// `planet.equatorial_radius` of a random planet.
    pub fn equatorial_radius(&self) -> Result<String> {
        self.equatorial_radius_of(self.random_planet()?)
    }

    /// `equatorialRadius(planetName)`: `planet.equatorial_radius.<planet>`.
    pub fn equatorial_radius_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("equatorial_radius", planet_name)
    }

    /// `planet.surface_area` of a random planet.
    pub fn surface_area(&self) -> Result<String> {
        self.surface_area_of(self.random_planet()?)
    }

    /// `surfaceArea(planetName)`: `planet.surface_area.<planet>`.
    pub fn surface_area_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("surface_area", planet_name)
    }

    /// `planet.volume` of a random planet.
    pub fn volume(&self) -> Result<String> {
        self.volume_of(self.random_planet()?)
    }

    /// `volume(planetName)`: `planet.volume.<planet>`.
    pub fn volume_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("volume", planet_name)
    }

    /// `planet.mass` of a random planet.
    pub fn mass(&self) -> Result<String> {
        self.mass_of(self.random_planet()?)
    }

    /// `mass(planetName)`: `planet.mass.<planet>`.
    pub fn mass_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("mass", planet_name)
    }

    /// `planet.gravitational_parameters` of a random planet.
    pub fn gravitation_parameters(&self) -> Result<String> {
        self.gravitation_parameters_of(self.random_planet()?)
    }

    /// `gravitationParameters(planetName)`: `planet.gravitational_parameters.<planet>`.
    pub fn gravitation_parameters_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("gravitational_parameters", planet_name)
    }

    /// `planet.density` of a random planet.
    pub fn density(&self) -> Result<String> {
        self.density_of(self.random_planet()?)
    }

    /// `density(planetName)`: `planet.density.<planet>`.
    pub fn density_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("density", planet_name)
    }

    /// `planet.equatorial_gravity` of a random planet.
    pub fn equatorial_gravity(&self) -> Result<String> {
        self.equatorial_gravity_of(self.random_planet()?)
    }

    /// `equatorialGravity(planetName)`: `planet.equatorial_gravity.<planet>`.
    pub fn equatorial_gravity_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("equatorial_gravity", planet_name)
    }

    /// `planet.escape_velocity` of a random planet.
    pub fn escape_velocity(&self) -> Result<String> {
        self.escape_velocity_of(self.random_planet()?)
    }

    /// `escapeVelocity(planetName)`: `planet.escape_velocity.<planet>`.
    pub fn escape_velocity_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("escape_velocity", planet_name)
    }

    /// `planet.rotation_period` of a random planet.
    pub fn rotation_period(&self) -> Result<String> {
        self.rotation_period_of(self.random_planet()?)
    }

    /// `rotationPeriod(planetName)`: `planet.rotation_period.<planet>`.
    pub fn rotation_period_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("rotation_period", planet_name)
    }

    /// `planet.orbital_period` of a random planet.
    pub fn orbital_period(&self) -> Result<String> {
        self.orbital_period_of(self.random_planet()?)
    }

    /// `orbitalPeriod(planetName)`: `planet.orbital_period.<planet>`.
    pub fn orbital_period_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("orbital_period", planet_name)
    }

    /// `planet.mean_orbital_speed` of a random planet.
    pub fn mean_orbital_speed(&self) -> Result<String> {
        self.mean_orbital_speed_of(self.random_planet()?)
    }

    /// `meanOrbitalSpeed(planetName)`: `planet.mean_orbital_speed.<planet>`.
    pub fn mean_orbital_speed_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("mean_orbital_speed", planet_name)
    }

    /// `planet.eccentricity` of a random planet.
    pub fn eccentricity(&self) -> Result<String> {
        self.eccentricity_of(self.random_planet()?)
    }

    /// `eccentricity(planetName)`: `planet.eccentricity.<planet>`.
    pub fn eccentricity_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("eccentricity", planet_name)
    }

    /// `planet.inclination` of a random planet.
    pub fn inclination(&self) -> Result<String> {
        self.inclination_of(self.random_planet()?)
    }

    /// `inclination(planetName)`: `planet.inclination.<planet>`.
    pub fn inclination_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("inclination", planet_name)
    }

    /// `planet.axial_tilt` of a random planet.
    pub fn axial_tilt(&self) -> Result<String> {
        self.axial_tilt_of(self.random_planet()?)
    }

    /// `axialTilt(planetName)`: `planet.axial_tilt.<planet>`.
    pub fn axial_tilt_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("axial_tilt", planet_name)
    }

    /// `planet.mean_surface_temperature` of a random planet.
    pub fn mean_surface_temperature(&self) -> Result<String> {
        self.mean_surface_temperature_of(self.random_planet()?)
    }

    /// `meanSurfaceTemperature(planetName)`: `planet.mean_surface_temperature.<planet>`.
    pub fn mean_surface_temperature_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("mean_surface_temperature", planet_name)
    }

    /// `planet.mean_air_temperature` of a random planet.
    pub fn mean_air_temperature(&self) -> Result<String> {
        self.mean_air_temperature_of(self.random_planet()?)
    }

    /// `meanAirTemperature(planetName)`: `planet.mean_air_temperature.<planet>`.
    pub fn mean_air_temperature_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("mean_air_temperature", planet_name)
    }

    /// `planet.atmospheric_composition` of a random planet.
    pub fn atmospheric_composition(&self) -> Result<String> {
        self.atmospheric_composition_of(self.random_planet()?)
    }

    /// `atmosphericComposition(planetName)`: `planet.atmospheric_composition.<planet>`.
    pub fn atmospheric_composition_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("atmospheric_composition", planet_name)
    }

    /// `planet.number_of_known_moons` of a random planet.
    pub fn number_of_known_moons(&self) -> Result<String> {
        self.number_of_known_moons_of(self.random_planet()?)
    }

    /// `numberOfKnownMoons(planetName)`: `planet.number_of_known_moons.<planet>`.
    pub fn number_of_known_moons_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("number_of_known_moons", planet_name)
    }

    /// `planet.rings` of a random planet.
    pub fn rings(&self) -> Result<String> {
        self.rings_of(self.random_planet()?)
    }

    /// `rings(planetName)`: `planet.rings.<planet>`.
    pub fn rings_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("rings", planet_name)
    }

    /// `planet.planetary_discriminant` of a random planet.
    pub fn planetary_discriminant(&self) -> Result<String> {
        self.planetary_discriminant_of(self.random_planet()?)
    }

    /// `planetaryDiscriminant(planetName)`: `planet.planetary_discriminant.<planet>`.
    pub fn planetary_discriminant_of(&self, planet_name: PlanetName) -> Result<String> {
        self.of("planetary_discriminant", planet_name)
    }
}

fn planet(v: &JValue) -> Result<PlanetName> {
    match v {
        JValue::Enum(name) => Ok(CONSTANTS.iter().position(|c| c == name).map_or(PlanetName::Jupiter, |i| PlanetName::ALL[i])),
        JValue::Null => Err(crate::dispatch::npe(
            "Cannot invoke \"net.datafaker.providers.base.Planet$PlanetName.getName()\" because \"planetName\" is null",
        )),
        _ => Ok(PlanetName::Jupiter),
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("name", &[], |f, _| s(Planet::new(f).name())),
    Method::new("symbol", &[], |f, _| s(Planet::new(f).symbol())),
    Method::new("symbol", &[PLANET_NAME], |f, a| s(Planet::new(f).symbol_of(planet(&a[0])?))).accepting_null(),
    Method::new("classification", &[], |f, _| s(Planet::new(f).classification())),
    Method::new("classification", &[PLANET_NAME], |f, a| s(Planet::new(f).classification_of(planet(&a[0])?))).accepting_null(),
    Method::new("discoveryYear", &[], |f, _| s(Planet::new(f).discovery_year())),
    Method::new("discoveryYear", &[PLANET_NAME], |f, a| s(Planet::new(f).discovery_year_of(planet(&a[0])?))).accepting_null(),
    Method::new("meanDistanceFromTheSun", &[], |f, _| s(Planet::new(f).mean_distance_from_the_sun())),
    Method::new("meanDistanceFromTheSun", &[PLANET_NAME], |f, a| s(Planet::new(f).mean_distance_from_the_sun_of(planet(&a[0])?))).accepting_null(),
    Method::new("lengthOfDay", &[], |f, _| s(Planet::new(f).length_of_day())),
    Method::new("lengthOfDay", &[PLANET_NAME], |f, a| s(Planet::new(f).length_of_day_of(planet(&a[0])?))).accepting_null(),
    Method::new("equatorialRadius", &[], |f, _| s(Planet::new(f).equatorial_radius())),
    Method::new("equatorialRadius", &[PLANET_NAME], |f, a| s(Planet::new(f).equatorial_radius_of(planet(&a[0])?))).accepting_null(),
    Method::new("surfaceArea", &[], |f, _| s(Planet::new(f).surface_area())),
    Method::new("surfaceArea", &[PLANET_NAME], |f, a| s(Planet::new(f).surface_area_of(planet(&a[0])?))).accepting_null(),
    Method::new("volume", &[], |f, _| s(Planet::new(f).volume())),
    Method::new("volume", &[PLANET_NAME], |f, a| s(Planet::new(f).volume_of(planet(&a[0])?))).accepting_null(),
    Method::new("mass", &[], |f, _| s(Planet::new(f).mass())),
    Method::new("mass", &[PLANET_NAME], |f, a| s(Planet::new(f).mass_of(planet(&a[0])?))).accepting_null(),
    Method::new("gravitationParameters", &[], |f, _| s(Planet::new(f).gravitation_parameters())),
    Method::new("gravitationParameters", &[PLANET_NAME], |f, a| s(Planet::new(f).gravitation_parameters_of(planet(&a[0])?))).accepting_null(),
    Method::new("density", &[], |f, _| s(Planet::new(f).density())),
    Method::new("density", &[PLANET_NAME], |f, a| s(Planet::new(f).density_of(planet(&a[0])?))).accepting_null(),
    Method::new("equatorialGravity", &[], |f, _| s(Planet::new(f).equatorial_gravity())),
    Method::new("equatorialGravity", &[PLANET_NAME], |f, a| s(Planet::new(f).equatorial_gravity_of(planet(&a[0])?))).accepting_null(),
    Method::new("escapeVelocity", &[], |f, _| s(Planet::new(f).escape_velocity())),
    Method::new("escapeVelocity", &[PLANET_NAME], |f, a| s(Planet::new(f).escape_velocity_of(planet(&a[0])?))).accepting_null(),
    Method::new("rotationPeriod", &[], |f, _| s(Planet::new(f).rotation_period())),
    Method::new("rotationPeriod", &[PLANET_NAME], |f, a| s(Planet::new(f).rotation_period_of(planet(&a[0])?))).accepting_null(),
    Method::new("orbitalPeriod", &[], |f, _| s(Planet::new(f).orbital_period())),
    Method::new("orbitalPeriod", &[PLANET_NAME], |f, a| s(Planet::new(f).orbital_period_of(planet(&a[0])?))).accepting_null(),
    Method::new("meanOrbitalSpeed", &[], |f, _| s(Planet::new(f).mean_orbital_speed())),
    Method::new("meanOrbitalSpeed", &[PLANET_NAME], |f, a| s(Planet::new(f).mean_orbital_speed_of(planet(&a[0])?))).accepting_null(),
    Method::new("eccentricity", &[], |f, _| s(Planet::new(f).eccentricity())),
    Method::new("eccentricity", &[PLANET_NAME], |f, a| s(Planet::new(f).eccentricity_of(planet(&a[0])?))).accepting_null(),
    Method::new("inclination", &[], |f, _| s(Planet::new(f).inclination())),
    Method::new("inclination", &[PLANET_NAME], |f, a| s(Planet::new(f).inclination_of(planet(&a[0])?))).accepting_null(),
    Method::new("axialTilt", &[], |f, _| s(Planet::new(f).axial_tilt())),
    Method::new("axialTilt", &[PLANET_NAME], |f, a| s(Planet::new(f).axial_tilt_of(planet(&a[0])?))).accepting_null(),
    Method::new("meanSurfaceTemperature", &[], |f, _| s(Planet::new(f).mean_surface_temperature())),
    Method::new("meanSurfaceTemperature", &[PLANET_NAME], |f, a| s(Planet::new(f).mean_surface_temperature_of(planet(&a[0])?))).accepting_null(),
    Method::new("meanAirTemperature", &[], |f, _| s(Planet::new(f).mean_air_temperature())),
    Method::new("meanAirTemperature", &[PLANET_NAME], |f, a| s(Planet::new(f).mean_air_temperature_of(planet(&a[0])?))).accepting_null(),
    Method::new("atmosphericComposition", &[], |f, _| s(Planet::new(f).atmospheric_composition())),
    Method::new("atmosphericComposition", &[PLANET_NAME], |f, a| s(Planet::new(f).atmospheric_composition_of(planet(&a[0])?))).accepting_null(),
    Method::new("numberOfKnownMoons", &[], |f, _| s(Planet::new(f).number_of_known_moons())),
    Method::new("numberOfKnownMoons", &[PLANET_NAME], |f, a| s(Planet::new(f).number_of_known_moons_of(planet(&a[0])?))).accepting_null(),
    Method::new("rings", &[], |f, _| s(Planet::new(f).rings())),
    Method::new("rings", &[PLANET_NAME], |f, a| s(Planet::new(f).rings_of(planet(&a[0])?))).accepting_null(),
    Method::new("planetaryDiscriminant", &[], |f, _| s(Planet::new(f).planetary_discriminant())),
    Method::new("planetaryDiscriminant", &[PLANET_NAME], |f, a| s(Planet::new(f).planetary_discriminant_of(planet(&a[0])?))).accepting_null(),
];
