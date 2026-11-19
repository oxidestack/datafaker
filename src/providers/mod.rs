//! Datafaker's providers (`net.datafaker.providers.*`). Each is a view over a [`Faker`], as
//! `faker.name()` returns a cached provider bound to its faker upstream, plus the table of its public methods that
//! expressions reach.
//!
//! Providers whose public methods only resolve YAML keys are generated from the Java sources into `generated.rs`
//! by `scripts/gen-providers.py`, which also writes the registry of every ported provider (`registry!` below);
//! the others are written by hand, one module each.

/// A provider's Java enum whose constants each name a YAML key: `Name, "DispatchName" { Variant = "CONSTANT" =>
/// "yaml_key", … }` defines the Rust enum in declaration order with `VALUES`, `constant()`, `yaml_key()`, the
/// method-table parameter `PARAM` (`P::Enum`, named `DispatchName` in the vector call syntax) and `of(&JValue)`.
macro_rules! provider_enum {
    ($(#[$doc:meta])* $name:ident, $dispatch:literal { $($variant:ident = $constant:literal => $key:literal,)* }) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $(
                #[doc = concat!("`", $constant, "`: the `", $key, "` key.")]
                $variant,
            )*
        }

        impl $name {
            /// `values()`.
            pub const VALUES: &'static [$name] = &[$($name::$variant,)*];

            const CONSTANTS: &'static [&'static str] = &[$($constant,)*];

            /// The parameter type of the method tables (unused for an enum no public method takes).
            #[allow(dead_code)]
            pub(crate) const PARAM: $crate::dispatch::P = $crate::dispatch::P::Enum($dispatch, $name::CONSTANTS);

            /// The Java constant's name (`name()`).
            pub fn constant(self) -> &'static str {
                match self {
                    $($name::$variant => $constant,)*
                }
            }

            /// The YAML key segment the constant stands for.
            pub fn yaml_key(self) -> &'static str {
                match self {
                    $($name::$variant => $key,)*
                }
            }

            /// The constant a method table's argument names, for a Java body that reads `param.yamlKey` (a `null`
            /// one throws that dereference's `NullPointerException`).
            #[allow(dead_code)]
            pub(crate) fn of_yaml_key_param(value: &$crate::dispatch::JValue, param: &str) -> $crate::Result<$name> {
                match value {
                    $crate::dispatch::JValue::Null => Err($crate::Error::new(
                        "NullPointerException",
                        format!("Cannot read field \"yamlKey\" because \"{param}\" is null"),
                    )),
                    value => Ok($name::of(value)),
                }
            }

            /// [`of`](Self::of), `None` for a `null` argument.
            #[allow(dead_code)]
            pub(crate) fn of_nullable(value: &$crate::dispatch::JValue) -> Option<$name> {
                (*value != $crate::dispatch::JValue::Null).then(|| $name::of(value))
            }

            /// The constant a method table's coerced argument names.
            #[allow(dead_code)]
            pub(crate) fn of(value: &$crate::dispatch::JValue) -> $name {
                let $crate::dispatch::JValue::Enum(name) = value else { unreachable!("coerced to an enum constant") };
                $name::VALUES.iter().copied().find(|c| c.constant() == *name).expect("a constant of the enum")
            }
        }
    };
}

mod address;
mod animal;
mod app;
mod avatar;
mod aviation;
mod aws;
mod azure;
mod barcode;
mod bool;
mod brand;
mod business;
mod cheese;
mod cnpj;
mod code;
mod coffee;
mod color;
mod commerce;
mod company;
mod compass;
mod computer;
mod country;
mod cpf;
mod credentials;
mod date_and_time;
mod disease;
mod domain;
mod dota2;
mod driving_license;
mod drone;
mod dune;
mod educator;
mod emergency;
mod fake_duration;
mod file;
mod finance;
mod financial_terms;
mod fingerprint;
mod food;
mod hashing;
mod hearthstone;
mod http;
mod id_number;
mod image;
pub(crate) mod internet;
mod iso_codes;
mod jdk_tables;
mod language_code;
mod locality;
mod lorem;
mod mbti;
mod medical;
mod medical_procedure;
mod minecraft;
mod money;
mod music;
mod name;
mod nation;
mod number;
mod options;
mod oscar_movie;
mod passport;
mod phone_number;
mod planet;
mod relationship;
mod restaurant;
mod shakespeare;
mod sip;
mod space;
mod star_wars;
mod tea;
pub mod text;
mod time;
mod time_and_date;
mod tire;
mod tron;
mod twitter;
mod unique;
mod vehicle;
mod weather;

pub use address::Address;
pub use animal::Animal;
pub use app::App;
pub use avatar::Avatar;
pub use aviation::{Aviation, AviationCodeType};
pub use aws::Aws;
pub use azure::Azure;
pub use barcode::Barcode;
pub use bool::Bool;
pub use brand::Brand;
pub use business::Business;
pub use cheese::{Cheese, Wedge};
pub use cnpj::CNPJ;
pub use code::Code;
pub use coffee::{Coffee, Country as CoffeeCountry};
pub use color::Color;
pub use commerce::Commerce;
pub use company::Company;
pub use compass::{Compass, CompassPoint};
pub use computer::Computer;
pub use country::Country;
pub use cpf::CPF;
pub use credentials::Credentials;
pub use date_and_time::{DateAndTime, str2duration_unit};
pub use disease::{Disease, DiseaseType};
pub use domain::Domain;
pub use dota2::Dota2;
pub use driving_license::DrivingLicense;
pub use drone::Drone;
pub use dune::{Dune, Quote as DuneQuote, Saying as DuneSaying};
pub use educator::Educator;
pub use emergency::{Emergency, EmergencyCase};
pub use fake_duration::FakeDuration;
pub use file::File;
pub use finance::{CreditCardType, Finance};
pub use financial_terms::{Category as FinancialTermsCategory, FinancialTerms};
pub use fingerprint::{Fingerprint, PatternType as FingerprintPatternType};
pub use food::Food;
pub use hashing::Hashing;
pub use hearthstone::Hearthstone;
pub use http::{Browser, Http};
pub use id_number::IdNumber;
pub use image::{Base64ImageRuleConfig, Image, ImageBuilder, ImageType};
pub use internet::{BotUserAgent, Internet, PortRange, UserAgent};
pub use language_code::LanguageCode;
pub use locality::Locality;
pub use lorem::Lorem;
pub use mbti::Mbti;
pub use medical::Medical;
pub use medical_procedure::MedicalProcedure;
pub use minecraft::Minecraft;
pub use money::Money;
pub use music::Music;
pub use name::Name;
pub use nation::Nation;
pub use number::Number;
pub use options::Options;
pub use oscar_movie::OscarMovie;
pub use passport::Passport;
pub use phone_number::PhoneNumber;
pub use planet::{Planet, PlanetName};
pub use relationship::Relationship;
pub use restaurant::Restaurant;
pub use shakespeare::Shakespeare;
pub use sip::Sip;
pub use space::Space;
pub use star_wars::StarWars;
pub use tea::Tea;
pub use text::Text;
pub use time::Time;
pub use time_and_date::TimeAndDate;
pub use tire::Tire;
pub use tron::{AlternateCharacterSpelling as TronAlternateCharacterSpelling, Character as TronCharacter, Quote as TronQuote, Tron};
pub use twitter::Twitter;
pub use unique::Unique;
pub use vehicle::Vehicle;
pub use weather::Weather;

use crate::Faker;

/// The registry of ported providers: `Kind => module, "javaAccessor", rust_accessor, "java.package";` per provider
/// class, in the order of `BaseProviders`. It defines [`ProviderKind`], the `Faker` accessors and the faker's
/// provider methods that expressions reach (`#{Ancient.god}`), which consult the faker's white list.
macro_rules! registry {
    ($($kind:ident => $($module:ident)::+, $java:literal, $rust:ident, $package:literal;)*) => {
        /// A provider class.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum ProviderKind {
            $(
                #[doc = concat!("`", $package, ".", stringify!($kind), "`.")]
                $kind,
            )*
        }

        impl ProviderKind {
            /// Every provider class, in registry order.
            pub const ALL: &'static [ProviderKind] = &[$(ProviderKind::$kind,)*];

            /// The class's simple name.
            pub fn simple_name(self) -> &'static str {
                match self {
                    $(ProviderKind::$kind => stringify!($kind),)*
                }
            }

            /// The class's binary name (`getName()`), as the white list sees it.
            pub fn class_name(self) -> &'static str {
                match self {
                    $(ProviderKind::$kind => concat!($package, ".", stringify!($kind)),)*
                }
            }

            pub(crate) fn methods(self) -> &'static [$crate::dispatch::Method] {
                match self {
                    $(ProviderKind::$kind => $($module)::+::METHODS,)*
                }
            }
        }

        impl $crate::Faker {
            $(
                #[doc = concat!("`", $java, "()`.")]
                pub fn $rust(&self) -> $($module)::+::$kind<'_> {
                    $($module)::+::$kind::new(self)
                }
            )*
        }

        /// The faker's provider methods (`faker.ancient()`), first in its method table.
        pub(crate) static ACCESSORS: &[$crate::dispatch::Method] = &[
            $($crate::dispatch::Method::new($java, &[], |f, _| {
                f.check_white_list(ProviderKind::$kind.class_name())?;
                Ok($crate::dispatch::JValue::Provider(ProviderKind::$kind))
            }),)*
        ];
    };
}

/// A provider whose public methods only resolve YAML keys: `Kind { rust_name, "javaName", provider|faker "key"; … }`
/// defines the struct, its methods (`provider`: `AbstractProvider.resolve(key)`, `faker`: `faker.resolve(key)`)
/// and its `METHODS` table.
macro_rules! yaml_provider {
    ($(#[$doc:meta])* $kind:ident { $($(#[$mdoc:meta])* $rust:ident, $java:literal, $via:ident $key:literal;)* }) => {
        $(#[$doc])*
        #[derive(Clone, Copy)]
        pub struct $kind<'a> {
            faker: &'a $crate::Faker,
        }

        impl<'a> $kind<'a> {
            pub(crate) fn new(faker: &'a $crate::Faker) -> Self {
                $kind { faker }
            }

            $(
                $(#[$mdoc])*
                pub fn $rust(&self) -> $crate::Result<String> {
                    yaml_provider!(@resolve $via, self.faker, $kind, $key)
                }
            )*
        }

        pub(crate) static METHODS: &[$crate::dispatch::Method] = &[
            $($crate::dispatch::Method::new($java, &[], |f, _| $kind::new(f).$rust().map($crate::dispatch::JValue::Str)),)*
        ];
    };
    (@resolve provider, $faker:expr, $kind:ident, $key:literal) => {
        $crate::providers::resolve($faker, $crate::providers::ProviderKind::$kind, $key)
    };
    (@resolve faker, $faker:expr, $kind:ident, $key:literal) => {
        $faker.resolve($key)
    };
}

pub(crate) mod generated;
pub use generated::*;

/// `AbstractProvider.resolve(key)`.
pub(crate) fn resolve(faker: &Faker, kind: ProviderKind, key: &str) -> crate::Result<String> {
    crate::expr::resolve_key(faker, key, Some(crate::dispatch::Target::Provider(kind).into()))
}

/// A method-table result from a `Result<String>`.
pub(crate) fn s(r: crate::Result<String>) -> crate::Result<crate::dispatch::JValue> {
    r.map(crate::dispatch::JValue::Str)
}

/// `new DecimalFormatSymbols(locale)`'s zero digit, decimal separator and negative prefix (the JDK's, from
/// `jdk_tables::NUMBER_SYMBOLS`): the locale's entry, else its language's, else `0`, `.`, `-`.
pub(crate) fn number_symbols(locale: &crate::Locale) -> (char, char, &'static str) {
    let find = |country: &str| {
        jdk_tables::NUMBER_SYMBOLS.iter().find(|(l, c, ..)| *l == locale.language() && *c == country).map(|&(_, _, z, d, m)| (z, d, m))
    };
    find(locale.country()).or_else(|| find("")).unwrap_or(('0', '.', "-"))
}

/// `new DecimalFormat("#0.<decimals zeros>", new DecimalFormatSymbols(locale)).format(value)`: `HALF_EVEN` on the
/// double's exact value, with the locale's digits, decimal separator and minus sign.
pub(crate) fn format_fixed(value: f64, decimals: usize, locale: &crate::Locale) -> String {
    let (zero, separator, minus) = number_symbols(locale);
    let text = format!("{:.*}", decimals, value.abs());
    let negative = value.is_sign_negative() && text.chars().any(|c| c.is_ascii_digit() && c != '0');
    let mut out = String::new();
    if negative {
        out.push_str(minus);
    }
    for c in text.chars() {
        out.push(match c {
            '.' => separator,
            d => char::from_u32(zero as u32 + (d as u32 - '0' as u32)).unwrap_or(d),
        });
    }
    out
}

/// `String.format(locale, "%.<precision>f", value)`: `Formatter` rounds the double's shortest decimal digits
/// `HALF_UP` (not its exact value), then prints them with the locale's zero digit and decimal separator.
pub(crate) fn format_f(value: f64, precision: usize, locale: &crate::Locale) -> String {
    let (zero, separator, _) = number_symbols(locale);
    let negative = value < 0.0 || value == 0.0 && value.is_sign_negative();
    let sci = format!("{:e}", value.abs());
    let (mantissa, exponent) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exponent: i64 = exponent.parse().unwrap_or(0);
    let mut digits: Vec<u8> = mantissa.bytes().filter(u8::is_ascii_digit).map(|b| b - b'0').collect();
    // `digits` are d.ddd × 10^exponent: `point` digits before the decimal point (prepend zeros when it is < 1).
    let mut point = exponent + 1;
    if point < 1 {
        let pad = (1 - point) as usize;
        digits.splice(0..0, std::iter::repeat_n(0, pad));
        point = 1;
    }
    let keep = point as usize + precision;
    if digits.len() > keep {
        let round_up = digits[keep] >= 5;
        digits.truncate(keep);
        if round_up {
            let mut i = keep;
            loop {
                if i == 0 {
                    digits.insert(0, 1);
                    point += 1;
                    break;
                }
                i -= 1;
                if digits[i] == 9 {
                    digits[i] = 0;
                } else {
                    digits[i] += 1;
                    break;
                }
            }
        }
    }
    digits.resize(point as usize + precision, 0);
    let mut out = String::new();
    if negative {
        out.push('-');
    }
    for (i, d) in digits.iter().enumerate() {
        if i == point as usize {
            out.push(separator);
        }
        out.push(char::from_u32(zero as u32 + *d as u32).unwrap_or('0'));
    }
    out
}

/// `AbstractProvider.resolve(key, message)`: as [`resolve`], but a missing key fails with `message`.
pub(crate) fn resolve_or(faker: &Faker, kind: ProviderKind, key: &str, message: impl FnOnce() -> String) -> crate::Result<String> {
    match faker.fake_values_service().safe_fetch(key, None)? {
        Some(expression) => crate::expr::resolve_expression(faker, &expression, Some(crate::dispatch::Target::Provider(kind).into())),
        None => Err(crate::Error::runtime(message())),
    }
}
