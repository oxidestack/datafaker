//! `net.datafaker.providers.base.Aviation`, with `AviationCodeType`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{JValue, Method, P};
use crate::{Faker, Result};

/// `AviationCodeType`: IATA (3-letter airport, 2-character airline) or ICAO (4-letter airport, 3-letter airline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AviationCodeType {
    /// `IATA`: three-letter airport and two-character airline codes.
    Iata,
    /// `ICAO`: four-letter airport and three-letter airline codes.
    Icao,
}

const CODE_TYPE: P = P::Enum("AviationCodeType", &["IATA", "ICAO"]);

const AIRCRAFT_KEYS: [&str; 6] = [
    "aviation.aircraft.airplane",
    "aviation.aircraft.warplane",
    "aviation.aircraft.army_helicopter",
    "aviation.aircraft.civil_helicopter",
    "aviation.aircraft.general",
    "aviation.aircraft.cargo",
];

/// Aviation: aircraft, airports, airlines, flights and gates, from `aviation.*`.
#[derive(Clone, Copy)]
pub struct Aviation<'a> {
    faker: &'a Faker,
}

impl<'a> Aviation<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Aviation { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Aviation, key)
    }

    /// An aircraft of a random kind (airplane, warplane, helicopter, general, cargo).
    pub fn aircraft(&self) -> Result<String> {
        self.resolve(AIRCRAFT_KEYS[self.faker.number().number_between(0, 6) as usize])
    }

    /// `aviation.aircraft.airplane`.
    pub fn airplane(&self) -> Result<String> {
        self.resolve("aviation.aircraft.airplane")
    }

    /// `aviation.aircraft.warplane`.
    pub fn warplane(&self) -> Result<String> {
        self.resolve("aviation.aircraft.warplane")
    }

    /// `aviation.aircraft.general`.
    pub fn general(&self) -> Result<String> {
        self.resolve("aviation.aircraft.general")
    }

    /// `aviation.aircraft.cargo`.
    pub fn cargo(&self) -> Result<String> {
        self.resolve("aviation.aircraft.cargo")
    }

    /// `aviation.aircraft.army_helicopter`.
    pub fn army_helicopter(&self) -> Result<String> {
        self.resolve("aviation.aircraft.army_helicopter")
    }

    /// `aviation.aircraft.civil_helicopter`.
    pub fn civil_helicopter(&self) -> Result<String> {
        self.resolve("aviation.aircraft.civil_helicopter")
    }

    /// `aviation.airport`.
    pub fn airport(&self) -> Result<String> {
        self.resolve("aviation.airport")
    }

    /// `airport(codeType)`: `aviation.airport_iata` or `aviation.airport_icao`.
    pub fn airport_of(&self, code_type: AviationCodeType) -> Result<String> {
        match code_type {
            AviationCodeType::Iata => self.resolve("aviation.airport_iata"),
            AviationCodeType::Icao => self.resolve("aviation.airport_icao"),
        }
    }

    /// `aviation.airport_name`.
    pub fn airport_name(&self) -> Result<String> {
        self.resolve("aviation.airport_name")
    }

    /// `METAR()`: `aviation.metar`.
    pub fn metar(&self) -> Result<String> {
        self.resolve("aviation.metar")
    }

    /// `aviation.manufacturer`.
    pub fn manufacturer(&self) -> Result<String> {
        self.resolve("aviation.manufacturer")
    }

    /// `aviation.aircraft_type_special_designator`.
    pub fn special_type_designator(&self) -> Result<String> {
        self.resolve("aviation.aircraft_type_special_designator")
    }

    /// `aviation.engine_type`.
    pub fn engine_type(&self) -> Result<String> {
        self.resolve("aviation.engine_type")
    }

    /// The deprecated `flight(type)`: ICAO when `type` is `ICAO` in any case, else IATA (Java's `null` too).
    pub fn flight_by_name(&self, code_type: &str) -> Result<String> {
        self.flight_of(if code_type.eq_ignore_ascii_case("ICAO") { AviationCodeType::Icao } else { AviationCodeType::Iata })
    }

    /// `flight(type)`: an airline code and a number from 0 to 9998.
    pub fn flight_of(&self, code_type: AviationCodeType) -> Result<String> {
        let airline = match code_type {
            AviationCodeType::Icao => self.resolve("aviation.ICAO_airline")?,
            AviationCodeType::Iata => self.resolve("aviation.IATA_airline")?,
        };
        Ok(format!("{airline}{}", self.faker.number().number_between(0, 9999)))
    }

    /// `flight()`: an IATA flight.
    pub fn flight(&self) -> Result<String> {
        self.flight_of(AviationCodeType::Iata)
    }

    /// `aviation.flight_status`.
    pub fn flight_status(&self) -> Result<String> {
        self.resolve("aviation.flight_status")
    }

    /// A gate: a number from 1 to 255, or a letter and such a number.
    pub fn gate(&self) -> Result<String> {
        if self.faker.random().next_boolean() {
            Ok(self.faker.number().number_between(1, 256).to_string())
        } else {
            let letter = self.faker.regexify("[A-Z]")?;
            Ok(format!("{letter}{}", self.faker.number().number_between(1, 256)))
        }
    }

    /// `aviation.airline`.
    pub fn airline(&self) -> Result<String> {
        self.resolve("aviation.airline")
    }

    /// `aviation.uld_type`.
    pub fn uld_type(&self) -> Result<String> {
        self.resolve("aviation.uld_type")
    }

    /// `aviation.uld_code`.
    pub fn uld_code(&self) -> Result<String> {
        self.resolve("aviation.uld_code")
    }
}

fn code_type(v: &JValue) -> AviationCodeType {
    if matches!(v, JValue::Enum("ICAO")) { AviationCodeType::Icao } else { AviationCodeType::Iata }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("aircraft", &[], |f, _| s(Aviation::new(f).aircraft())),
    Method::new("airplane", &[], |f, _| s(Aviation::new(f).airplane())),
    Method::new("warplane", &[], |f, _| s(Aviation::new(f).warplane())),
    Method::new("general", &[], |f, _| s(Aviation::new(f).general())),
    Method::new("cargo", &[], |f, _| s(Aviation::new(f).cargo())),
    Method::new("armyHelicopter", &[], |f, _| s(Aviation::new(f).army_helicopter())),
    Method::new("civilHelicopter", &[], |f, _| s(Aviation::new(f).civil_helicopter())),
    Method::new("airport", &[], |f, _| s(Aviation::new(f).airport())),
    Method::new("airport", &[CODE_TYPE], |f, a| {
        crate::dispatch::non_null(
            a,
            0,
            "Cannot invoke \"net.datafaker.providers.base.AviationCodeType.ordinal()\" because \"codeType\" is null",
        )?;
        s(Aviation::new(f).airport_of(code_type(&a[0])))
    })
    .accepting_null(),
    Method::new("airportName", &[], |f, _| s(Aviation::new(f).airport_name())),
    Method::new("METAR", &[], |f, _| s(Aviation::new(f).metar())),
    Method::new("manufacturer", &[], |f, _| s(Aviation::new(f).manufacturer())),
    Method::new("specialTypeDesignator", &[], |f, _| s(Aviation::new(f).special_type_designator())),
    Method::new("engineType", &[], |f, _| s(Aviation::new(f).engine_type())),
    // `flight(AviationCodeType)` before `flight(String)`, as `getMethods()` orders them in MockServer 8.0.0's image
    // (Temurin 26.0.2): an expression argument both accept (`'ICAO'`) reaches the enum overload.
    Method::new("flight", &[CODE_TYPE], |f, a| s(Aviation::new(f).flight_of(code_type(&a[0])))),
    Method::new("flight", &[], |f, _| s(Aviation::new(f).flight())),
    Method::new("flight", &[P::String], |f, a| s(Aviation::new(f).flight_by_name(a[0].as_str()))),
    Method::new("flightStatus", &[], |f, _| s(Aviation::new(f).flight_status())),
    Method::new("gate", &[], |f, _| s(Aviation::new(f).gate())),
    Method::new("airline", &[], |f, _| s(Aviation::new(f).airline())),
    Method::new("uldType", &[], |f, _| s(Aviation::new(f).uld_type())),
    Method::new("uldCode", &[], |f, _| s(Aviation::new(f).uld_code())),
];
