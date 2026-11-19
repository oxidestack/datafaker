//! A port of Datafaker 2.7.0 (`net.datafaker:datafaker`, Java): fake data generated from Datafaker's own locale files,
//! with the same output as Datafaker for the same `java.util.Random` seed.
//!
//! A [`Faker`] is the entry point, as in Java: `Faker::with_seed(9999)` is `new Faker(new Random(9999))`, and each
//! provider is a method of it (`faker.name()`, `faker.internet()`, `faker.star_wars()`, …). Expressions resolve as
//! Datafaker resolves them ([`Faker::expression`]), and every method that can throw in Java returns [`Result`], whose
//! [`Error`] carries the Java exception's simple class name and message. The locale files are embedded at build time,
//! so nothing is read from disk at run time.
//!
//! # Example
//!
//! ```
//! use datafaker::{Faker, Locale};
//!
//! let faker = Faker::with_seed(9999); // new Faker(new Random(9999))
//! assert_eq!(faker.name().first_name().unwrap(), "Kermit");
//! assert_eq!(faker.internet().email_address().unwrap(), "betty.vandervort@hotmail.com");
//!
//! // `#{…}` expressions, as in `faker.expression(...)` in Java.
//! let greeting = faker.expression("Hello, #{Name.firstName}!").unwrap();
//! assert!(greeting.starts_with("Hello, "));
//!
//! // Another locale, with Datafaker's fallback chain (de_DE, then de, then English).
//! let german = Faker::with_locale_and_seed(Locale::new("de", "DE"), 1);
//! assert!(!german.address().city().unwrap().is_empty());
//! ```
//!
//! # Modules
//!
//! - [`providers`]: every ported provider, each a view over a [`Faker`].
//! - [`transformations`] and [`sequence`]: schemas rendered as CSV, JSON, YAML, XML, TOML or SQL, and fake collections
//!   and streams.
//! - [`java`]: the parts of the JDK whose behaviour the output depends on (`Random`, `java.time`, `BigDecimal`,
//!   `ImageIO`'s writers).
//! - [`random_service`], [`weighted_random_selector`], [`rgxgen`]: the randomness Datafaker draws through.
//! - [`idnumbers`], [`idn`], [`digests`]: national identity numbers, `IDN.toASCII`, message digests.
//! - [`dispatch`], [`object_methods`], [`annotations`]: what Datafaker finds by reflection, as method tables, and custom
//!   providers.
//! - [`yaml`], [`en_files`], [`resources`], [`lazy`]: loading the locale files.

#![warn(missing_docs)]

pub mod annotations;
pub mod dispatch;
pub mod digests;
pub mod en_files;
mod error;
mod expr;
mod faker;
pub mod lazy;
pub mod idn;
pub mod idnumbers;
mod idn_tables;
pub mod java;
mod locale;
mod md5;
pub mod object_methods;
pub mod providers;
mod random_methods;
pub mod random_service;
pub mod rgxgen;
pub mod sequence;
mod service;
pub mod transformations;
mod values;
pub mod weighted_random_selector;
pub mod yaml;

/// Datafaker's locale files, embedded at build time.
pub mod resources {
    include!(concat!(env!("OUT_DIR"), "/resources.rs"));
}

pub use dispatch::JValue;
pub use error::{Error, Result};
pub use faker::Faker;
pub use locale::Locale;
pub use random_service::{Range, RandomService};
pub use service::FakeValuesService;
pub use values::{FakeValues, FakeValuesContext, FakeValuesGrouping, FakeValuesInterface, capitalize, prefix_unqualified_expressions, to_java_names};
pub use weighted_random_selector::WeightedRandomSelector;
