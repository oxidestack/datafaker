//! `net.datafaker.providers.base.PhoneNumber` and `PhoneNumberGenerator`, on libphonenumber 9.0.41 (the
//! `libphonenumber` crate, a port of the version MockServer 9.2.0 ships).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use libphonenumber::{PhoneNumberFormat, PhoneNumberType, PhoneNumberUtil};

use super::{ProviderKind, s};
use crate::dispatch::{Method, P};
use crate::{Error, Faker, Result};

/// `PhoneNumberGenerator.MAX_RETRIES`.
const MAX_RETRIES: usize = 100;

/// The provider's state: the country code chosen from the faker's locale when the provider was first created
/// (upstream computes it in the constructor of the cached provider).
#[derive(Default)]
struct CountryCode(Option<String>);

/// Phone numbers that libphonenumber accepts as valid for the faker's country.
#[derive(Clone, Copy)]
pub struct PhoneNumber<'a> {
    faker: &'a Faker,
}

impl<'a> PhoneNumber<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        faker.with_provider_state(ProviderKind::PhoneNumber, |state: &mut CountryCode| {
            if state.0.is_none() {
                let locale = faker.locale();
                state.0 = Some(country_code_iso2(locale.language(), locale.country()));
            }
        });
        PhoneNumber { faker }
    }

    /// `countryCodeIso2()` (package-private upstream): the region the numbers are generated for.
    pub fn country_code_iso2(&self) -> String {
        self.faker
            .with_provider_state(ProviderKind::PhoneNumber, |state: &mut CountryCode| state.0.clone())
            .expect("set by new")
    }

    fn generate(&self, number_type: PhoneNumberType, format: PhoneNumberFormat) -> Result<String> {
        random_phone_number(self.faker, &self.country_code_iso2(), number_type, format)
    }

    /// A mobile number in national format.
    pub fn cell_phone(&self) -> Result<String> {
        self.generate(PhoneNumberType::Mobile, PhoneNumberFormat::National)
    }

    /// A mobile number in international format.
    pub fn cell_phone_international(&self) -> Result<String> {
        self.generate(PhoneNumberType::Mobile, PhoneNumberFormat::International)
    }

    /// A fixed-line number in national format.
    pub fn phone_number(&self) -> Result<String> {
        self.phone_number_national()
    }

    /// A fixed-line number in international format.
    pub fn phone_number_international(&self) -> Result<String> {
        self.generate(PhoneNumberType::FixedLine, PhoneNumberFormat::International)
    }

    /// A fixed-line number in national format.
    pub fn phone_number_national(&self) -> Result<String> {
        self.generate(PhoneNumberType::FixedLine, PhoneNumberFormat::National)
    }

    /// Four digits.
    pub fn extension(&self) -> String {
        self.subscriber_number()
    }

    /// `length` digits (none for a negative length).
    pub fn subscriber_number_with(&self, length: i32) -> String {
        self.faker.numerify(&"#".repeat(length.max(0) as usize))
    }

    /// Four digits.
    pub fn subscriber_number(&self) -> String {
        self.subscriber_number_with(4)
    }
}

/// `PhoneNumber.countryCodeIso2(locale)`.
pub(crate) fn country_code_iso2(language: &str, country: &str) -> String {
    match country {
        "" => detect_country_by_language(language),
        "CAT" => "ES".into(),
        "NEP" => "NP".into(),
        "PAK" => "PK".into(),
        "BORK" => "US".into(),
        _ => country.into(),
    }
}

/// `PhoneNumber.detectCountryByLanguage(language)`.
fn detect_country_by_language(language: &str) -> String {
    match language {
        "af" => "ZA",
        "ar" => "SA",
        "am" => "ET",
        "be" => "BY",
        "bn" => "BD",
        "bs" => "BA",
        "ca" => "ES",
        "cy" => "GB",
        "cs" => "CZ",
        "el" => "GR",
        "et" => "EE",
        "en" => "US",
        "eu" => "ES",
        "fa" => "IR",
        "ga" => "IE",
        "gl" => "ES",
        "he" => "IL",
        "hi" => "IN",
        "hy" => "AM",
        "ja" => "JP",
        "ka" => "GE",
        "km" => "KH",
        "ko" => "KR",
        "mo" => "MD",
        "sq" => "AL",
        "sw" => "TZ",
        "ug" => "CN",
        "ur" => "PK",
        "ta" => "IN",
        "test" => "US",
        "uk" => "UA",
        "zh" => "CN",
        _ => return crate::java::string_to_upper(language, ""),
    }
    .into()
}

/// `PhoneNumberGenerator.CACHE`: the national-number pattern per region and type, shared by every faker.
fn pattern_cache() -> &'static Mutex<HashMap<(String, PhoneNumberType), String>> {
    static CACHE: OnceLock<Mutex<HashMap<(String, PhoneNumberType), String>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// `PhoneNumberGenerator.randomPhoneNumber(countryCodeIso2, type, format)`.
fn random_phone_number(faker: &Faker, country: &str, number_type: PhoneNumberType, format: PhoneNumberFormat) -> Result<String> {
    let util = PhoneNumberUtil::get_instance();
    let key = (country.to_string(), number_type);
    let cached = pattern_cache().lock().expect("pattern cache").get(&key).cloned();
    let pattern = match cached {
        Some(pattern) => pattern,
        None => {
            let pattern = phone_number_pattern(util, country, number_type)?;
            pattern_cache().lock().expect("pattern cache").entry(key).or_insert(pattern).clone()
        }
    };
    let mut candidate = generate_phone_number(faker, util, country, &pattern)?;
    let mut i = 0;
    while i < MAX_RETRIES && !util.is_valid_number(&candidate) {
        candidate = generate_phone_number(faker, util, country, &pattern)?;
        i += 1;
    }
    Ok(util.format(&candidate, format))
}

/// `PhoneNumberGenerator.generatePhoneNumber(countryCodeIso2, phoneNumberPattern)`.
fn generate_phone_number(
    faker: &Faker,
    util: &PhoneNumberUtil,
    country: &str,
    pattern: &str,
) -> Result<libphonenumber::PhoneNumber> {
    let national_number = faker.fake_values_service().regexify(pattern)?;
    util.parse(&national_number, Some(country)).map_err(|e| {
        Error::runtime(format!("Failed to parse generated phone number {national_number}"))
            .with_cause(Error::new("NumberParseException", e.message()))
    })
}

/// `PhoneNumberGenerator.phoneNumberPattern(countryCodeIso2, type)`: the type's national-number pattern of the
/// region's metadata, read as upstream reads it (`getMetadataForRegion` by reflection).
fn phone_number_pattern(util: &PhoneNumberUtil, country: &str, number_type: PhoneNumberType) -> Result<String> {
    let metadata = util
        .get_metadata_for_region(country)
        .ok_or_else(|| Error::new("NullPointerException", format!("Unsupported country code: {country}")))?;
    let desc = match number_type {
        PhoneNumberType::Mobile => metadata.mobile.as_ref(),
        PhoneNumberType::FixedLine => metadata.fixed_line.as_ref(),
        other => return Err(Error::illegal_argument(format!("Unsupported phone number type: {}", other.name()))),
    };
    Ok(desc.map(|d| d.national_number_pattern.clone()).unwrap_or_default())
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("cellPhone", &[], |f, _| s(PhoneNumber::new(f).cell_phone())),
    Method::new("cellPhoneInternational", &[], |f, _| s(PhoneNumber::new(f).cell_phone_international())),
    Method::new("phoneNumber", &[], |f, _| s(PhoneNumber::new(f).phone_number())),
    Method::new("phoneNumberInternational", &[], |f, _| s(PhoneNumber::new(f).phone_number_international())),
    Method::new("phoneNumberNational", &[], |f, _| s(PhoneNumber::new(f).phone_number_national())),
    Method::new("extension", &[], |f, _| s(Ok(PhoneNumber::new(f).extension()))),
    Method::new("subscriberNumber", &[P::Int], |f, a| s(Ok(PhoneNumber::new(f).subscriber_number_with(a[0].as_int())))),
    Method::new("subscriberNumber", &[], |f, _| s(Ok(PhoneNumber::new(f).subscriber_number()))),
];
