//! `net.datafaker.providers.base.Address`.

use super::{ProviderKind, format_f, resolve, resolve_or, s};
use crate::dispatch::{java_str, Method, P};
use crate::{Faker, Result};

/// Addresses: streets, cities, postcodes, coordinates, from `address.*`.
#[derive(Clone, Copy)]
pub struct Address<'a> {
    faker: &'a Faker,
}

impl<'a> Address<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Address { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Address, key)
    }

    /// `address.street_name`.
    pub fn street_name(&self) -> Result<String> {
        self.resolve("address.street_name")
    }

    /// `address.street_address`.
    pub fn street_address(&self) -> Result<String> {
        self.resolve("address.street_address")
    }

    /// `address.street_suffix`.
    pub fn street_suffix(&self) -> Result<String> {
        self.resolve("address.street_suffix")
    }

    /// `address.street_prefix` (`@Deterministic` upstream).
    pub fn street_prefix(&self) -> Result<String> {
        self.resolve("address.street_prefix")
    }

    /// `address.city_suffix`.
    pub fn city_suffix(&self) -> Result<String> {
        self.resolve("address.city_suffix")
    }

    /// `address.city_prefix`.
    pub fn city_prefix(&self) -> Result<String> {
        self.resolve("address.city_prefix")
    }

    /// `address.city`.
    pub fn city(&self) -> Result<String> {
        self.resolve("address.city")
    }

    /// `address.city_name`.
    pub fn city_name(&self) -> Result<String> {
        self.resolve("address.city_name")
    }

    /// `address.state`.
    pub fn state(&self) -> Result<String> {
        self.resolve("address.state")
    }

    /// `address.state_abbr`.
    pub fn state_abbr(&self) -> Result<String> {
        self.resolve("address.state_abbr")
    }

    /// `address.time_zone`.
    pub fn time_zone(&self) -> Result<String> {
        self.resolve("address.time_zone")
    }

    /// `address.country`.
    pub fn country(&self) -> Result<String> {
        self.resolve("address.country")
    }

    /// `address.country_code`.
    pub fn country_code(&self) -> Result<String> {
        self.resolve("address.country_code")
    }

    /// `address.full_address`.
    pub fn full_address(&self) -> Result<String> {
        self.resolve("address.full_address")
    }

    /// A house number from 1 to 999.
    pub fn street_address_number(&self) -> Result<String> {
        Ok(self.faker.random().next_int_between(1, 1000)?.to_string())
    }

    /// `streetAddress(includeSecondary)`.
    pub fn street_address_with(&self, include_secondary: bool) -> Result<String> {
        let street_address = self.street_address()?;
        Ok(if include_secondary { format!("{street_address} {}", self.secondary_address()?) } else { street_address })
    }

    /// `address.secondary_address`, numerified.
    pub fn secondary_address(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("address.secondary_address")?))
    }

    /// `address.postcode`, bothified.
    pub fn zip_code(&self) -> Result<String> {
        Ok(self.faker.bothify(&self.resolve("address.postcode")?))
    }

    /// `address.postcode`, bothified.
    pub fn postcode(&self) -> Result<String> {
        self.zip_code()
    }

    /// `address.postcode`, bothified (an Eircode in `en_IE`).
    pub fn eircode(&self) -> Result<String> {
        self.zip_code()
    }

    /// `address.postcode_plus_four`, bothified.
    pub fn zip_code_plus4(&self) -> Result<String> {
        Ok(self.faker.bothify(&self.resolve("address.postcode_plus_four")?))
    }

    /// `address.postcode_by_state.<stateAbbr>`.
    pub fn zip_code_by_state(&self, state_abbr: &str) -> Result<String> {
        self.resolve(&format!("address.postcode_by_state.{state_abbr}"))
    }

    /// `address.county_by_postcode.<postCode>`; `County is not configured for postcode …` when missing.
    pub fn county_by_zip_code(&self, post_code: &str) -> Result<String> {
        resolve_or(self.faker, ProviderKind::Address, &format!("address.county_by_postcode.{post_code}"), || {
            format!("County is not configured for postcode {post_code}")
        })
    }

    /// A latitude, `%.8f` in the faker's locale.
    pub fn latitude(&self) -> String {
        format_f(self.faker.random().next_double() * 180.0 - 90.0, 8, &self.faker.locale())
    }

    /// A longitude, `%.8f` in the faker's locale.
    pub fn longitude(&self) -> String {
        format_f(self.faker.random().next_double() * 360.0 - 180.0, 8, &self.faker.locale())
    }

    /// `latLon()`: comma-separated.
    pub fn lat_lon(&self) -> String {
        self.lat_lon_with(",")
    }

    /// `latLon(delimiter)`.
    pub fn lat_lon_with(&self, delimiter: &str) -> String {
        let latitude = self.latitude();
        format!("{latitude}{delimiter}{}", self.longitude())
    }

    /// `lonLat()`: comma-separated.
    pub fn lon_lat(&self) -> String {
        self.lon_lat_with(",")
    }

    /// `lonLat(delimiter)`.
    pub fn lon_lat_with(&self, delimiter: &str) -> String {
        let longitude = self.longitude();
        format!("{longitude}{delimiter}{}", self.latitude())
    }

    /// `address.building_number`, numerified.
    pub fn building_number(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("address.building_number")?))
    }

    /// `address.mail_box`, numerified.
    pub fn mail_box(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("address.mail_box")?))
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("streetName", &[], |f, _| s(Address::new(f).street_name())),
    Method::new("streetAddressNumber", &[], |f, _| s(Address::new(f).street_address_number())),
    Method::new("streetAddress", &[], |f, _| s(Address::new(f).street_address())),
    Method::new("streetAddress", &[P::Boolean], |f, a| s(Address::new(f).street_address_with(a[0].as_bool()))),
    Method::new("secondaryAddress", &[], |f, _| s(Address::new(f).secondary_address())),
    Method::new("zipCode", &[], |f, _| s(Address::new(f).zip_code())),
    Method::new("postcode", &[], |f, _| s(Address::new(f).postcode())),
    Method::new("eircode", &[], |f, _| s(Address::new(f).eircode())),
    Method::new("zipCodePlus4", &[], |f, _| s(Address::new(f).zip_code_plus4())),
    Method::new("zipCodeByState", &[P::String], |f, a| s(Address::new(f).zip_code_by_state(java_str(&a[0])))).accepting_null(),
    Method::new("countyByZipCode", &[P::String], |f, a| s(Address::new(f).county_by_zip_code(java_str(&a[0])))).accepting_null(),
    Method::new("streetSuffix", &[], |f, _| s(Address::new(f).street_suffix())),
    Method::new("streetPrefix", &[], |f, _| s(Address::new(f).street_prefix())),
    Method::new("citySuffix", &[], |f, _| s(Address::new(f).city_suffix())),
    Method::new("cityPrefix", &[], |f, _| s(Address::new(f).city_prefix())),
    Method::new("city", &[], |f, _| s(Address::new(f).city())),
    Method::new("cityName", &[], |f, _| s(Address::new(f).city_name())),
    Method::new("state", &[], |f, _| s(Address::new(f).state())),
    Method::new("stateAbbr", &[], |f, _| s(Address::new(f).state_abbr())),
    Method::new("latitude", &[], |f, _| s(Ok(Address::new(f).latitude()))),
    Method::new("longitude", &[], |f, _| s(Ok(Address::new(f).longitude()))),
    Method::new("latLon", &[], |f, _| s(Ok(Address::new(f).lat_lon()))),
    Method::new("latLon", &[P::String], |f, a| s(Ok(Address::new(f).lat_lon_with(java_str(&a[0]))))).accepting_null(),
    Method::new("lonLat", &[], |f, _| s(Ok(Address::new(f).lon_lat()))),
    Method::new("lonLat", &[P::String], |f, a| s(Ok(Address::new(f).lon_lat_with(java_str(&a[0]))))).accepting_null(),
    Method::new("timeZone", &[], |f, _| s(Address::new(f).time_zone())),
    Method::new("country", &[], |f, _| s(Address::new(f).country())),
    Method::new("countryCode", &[], |f, _| s(Address::new(f).country_code())),
    Method::new("buildingNumber", &[], |f, _| s(Address::new(f).building_number())),
    Method::new("fullAddress", &[], |f, _| s(Address::new(f).full_address())),
    Method::new("mailBox", &[], |f, _| s(Address::new(f).mail_box())),
];
