//! `net.datafaker.providers.base.Credentials`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method, P};
use crate::{Faker, Result};

pub const MIN_PASSWORD_LENGTH: i32 = 8;
pub const MAX_PASSWORD_LENGTH: i32 = 16;

/// User names, passwords and user ids.
#[derive(Clone, Copy)]
pub struct Credentials<'a> {
    faker: &'a Faker,
}

impl<'a> Credentials<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Credentials { faker }
    }

    /// `first.last`, lower-cased in the faker's locale, without apostrophes or whitespace.
    pub fn username(&self) -> Result<String> {
        let name = self.faker.name();
        let first = name.first_name()?;
        let last = name.last_name()?;
        Ok(Self::username_from(&first, &last, &self.faker.locale()))
    }

    /// The joining `username()` applies to a first and a last name.
    pub fn username_from(first_name: &str, last_name: &str, locale: &crate::Locale) -> String {
        let first = crate::java::string_to_lower(first_name, locale.language());
        let last = crate::java::string_to_lower(last_name, locale.language());
        let joined: Vec<u16> = format!("{first}.{last}")
            .encode_utf16()
            .filter(|c| *c != b'\'' as u16 && !crate::java::is_whitespace(*c))
            .collect();
        String::from_utf16_lossy(&joined)
    }

    /// `password()`: 8 to 16 lower-case letters and digits, at least one digit.
    pub fn password(&self) -> Result<String> {
        self.password_with(MIN_PASSWORD_LENGTH, MAX_PASSWORD_LENGTH, false, false, true)
    }

    /// `password(minimumLength, maximumLength, includeUppercase, includeSpecial, includeDigit)`.
    pub fn password_with(
        &self,
        minimum_length: i32,
        maximum_length: i32,
        include_uppercase: bool,
        include_special: bool,
        include_digit: bool,
    ) -> Result<String> {
        self.faker.text().text_with(minimum_length, maximum_length, include_uppercase, include_special, include_digit)
    }

    /// `credentials.weak_password`.
    pub fn weak_password(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Credentials, "credentials.weak_password")
    }

    /// `userId()`: a string matching the locale's `credentials.uid_pattern`.
    pub fn user_id(&self) -> Result<Option<String>> {
        let pattern = resolve(self.faker, ProviderKind::Credentials, "credentials.uid_pattern")?;
        self.user_id_matching(&pattern)
    }

    /// `userId(regex)`: `None` where `regex` does not compile. Upstream checks with `java.util.regex.Pattern`;
    /// here the check is RgxGen's parser, which accepts a superset.
    pub fn user_id_matching(&self, regex: &str) -> Result<Option<String>> {
        match crate::rgxgen::RgxGen::parse(regex) {
            Ok(_) => self.faker.regexify(regex).map(Some),
            Err(_) => Ok(None),
        }
    }
}

fn c(f: &Faker) -> Credentials<'_> {
    f.credentials()
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("username", &[], |f, _| s(c(f).username())),
    Method::new("password", &[], |f, _| s(c(f).password())),
    Method::new("password", &[P::Boolean], |f, a| s(c(f).password_with(8, 16, false, false, a[0].as_bool()))),
    Method::new("password", &[P::Int, P::Int], |f, a| s(c(f).password_with(a[0].as_int(), a[1].as_int(), false, false, true))),
    Method::new("password", &[P::Int, P::Int, P::Boolean], |f, a| {
        s(c(f).password_with(a[0].as_int(), a[1].as_int(), a[2].as_bool(), false, true))
    }),
    Method::new("password", &[P::Int, P::Int, P::Boolean, P::Boolean], |f, a| {
        s(c(f).password_with(a[0].as_int(), a[1].as_int(), a[2].as_bool(), a[3].as_bool(), true))
    }),
    Method::new("password", &[P::Int, P::Int, P::Boolean, P::Boolean, P::Boolean], |f, a| {
        s(c(f).password_with(a[0].as_int(), a[1].as_int(), a[2].as_bool(), a[3].as_bool(), a[4].as_bool()))
    }),
    Method::new("weakPassword", &[], |f, _| s(c(f).weak_password())),
    Method::new("userId", &[], |f, _| Ok(c(f).user_id()?.map_or(JValue::Null, JValue::Str))),
    // `userId(null)` returns null.
    Method::new("userId", &[P::String], |f, a| {
        if a[0] == JValue::Null {
            return Ok(JValue::Null);
        }
        Ok(c(f).user_id_matching(a[0].as_str())?.map_or(JValue::Null, JValue::Str))
    })
    .accepting_null(),
];
