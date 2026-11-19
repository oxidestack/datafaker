//! `net.datafaker.providers.base.Code`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{Method, P};
use crate::{Faker, Result};

const GTIN_8_CHECK_DIGITS: [u32; 7] = [3, 1, 3, 1, 3, 1, 3];
const GTIN_13_CHECK_DIGITS: [u32; 12] = [1, 3, 1, 3, 1, 3, 1, 3, 1, 3, 1, 3];
const REPORTING_BODY_IDENTIFIERS: [&str; 17] = ["01", "10", "30", "33", "35", "44", "45", "49", "50", "51", "52", "53", "54", "86", "91", "98", "99"];

/// Codes with check digits: ISBN-10/13, IMEI, EAN/GTIN; and ASINs.
#[derive(Clone, Copy)]
pub struct Code<'a> {
    faker: &'a Faker,
}

impl<'a> Code<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Code { faker }
    }

    /// A GS1 prefix of an ISBN-13: `978` or `979` (YAML `code.isbn_gs1` overrides it in expressions).
    pub fn isbn_gs1(&self) -> Result<String> {
        self.faker.options().option(&["978", "979"]).map(str::to_string)
    }

    /// An ISBN group: `0` or `1`.
    pub fn isbn_group(&self) -> Result<String> {
        self.faker.options().option(&["0", "1"]).map(str::to_string)
    }

    /// An ISBN registrant and publication element, `-`-separated, 8 digits in all.
    pub fn isbn_registrant(&self) -> Result<String> {
        let n = self.faker.number();
        let ct = self.faker.random().next_int_bounded(6)? + 1;
        Ok(match ct {
            6 => {
                let first = n.number_between(0, 1);
                format!("{first}{}-{}", n.digit(), n.digits(6))
            }
            5 => format!("{}-{}", n.number_between(200, 699), n.digits(5)),
            4 => format!("{}-{}", n.number_between(7000, 8499), n.digits(4)),
            3 => format!("{}-{}", n.number_between(85000, 89999), n.digits(3)),
            2 => format!("{}-{}", n.number_between(900000, 949999), n.digits(2)),
            _ => format!("{}-{}", n.number_between(9500000, 9999999), n.digits(1)),
        })
    }

    /// `isbn10()`: without separators.
    pub fn isbn10(&self) -> Result<String> {
        self.isbn10_with(false)
    }

    /// `isbn10(separator)`: group, registrant and check digit (`X` for 10).
    pub fn isbn10_with(&self, separator: bool) -> Result<String> {
        let mut isbn = format!("{}-{}-", self.faker.expression("#{code.isbn_group}")?, self.faker.expression("#{code.isbn_registrant}")?);
        let check = isbn10_check_digit(&isbn)?;
        if check == 10 { isbn.push('X') } else { isbn.push_str(&check.to_string()) }
        Ok(if separator { isbn } else { isbn.replace('-', "") })
    }

    /// `isbn13()`: without separators.
    pub fn isbn13(&self) -> Result<String> {
        self.isbn13_with(false)
    }

    /// `isbn13(separator)`: GS1 prefix, group, registrant and check digit.
    pub fn isbn13_with(&self, separator: bool) -> Result<String> {
        let gs1 = self.faker.expression("#{code.isbn_gs1}")?;
        let group = self.faker.expression("#{code.isbn_group}")?;
        let registrant = self.faker.expression("#{code.isbn_registrant}")?;
        let mut isbn = format!("{gs1}-{group}-{registrant}-");
        isbn.push_str(&isbn13_check_digit(&isbn)?.to_string());
        Ok(if separator { isbn } else { isbn.replace('-', "") })
    }

    /// `code.asin`.
    pub fn asin(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Code, "code.asin")
    }

    /// A 15-digit IMEI: a reporting body identifier, 12 digits from 0 to 8, and the Luhn check digit.
    pub fn imei(&self) -> Result<String> {
        const LEN: usize = 15;
        let mut digits = [0u32; LEN];
        let prefix = self.faker.options().option(&REPORTING_BODY_IDENTIFIERS)?;
        for (i, c) in prefix.chars().take(2).enumerate() {
            digits[i] = c.to_digit(10).unwrap_or(0);
        }
        for d in digits.iter_mut().take(LEN - 1).skip(2) {
            *d = self.faker.number().number_between(0, 9) as u32;
        }
        let len_offset = (LEN + 1) % 2;
        let mut sum = 0;
        for (i, d) in digits.iter().take(LEN - 1).enumerate() {
            if !(i + len_offset).is_multiple_of(2) {
                let t = d << 1;
                sum += if t > 9 { t - 9 } else { t };
            } else {
                sum += d;
            }
        }
        digits[LEN - 1] = (10 - sum % 10) % 10;
        Ok(digits.iter().map(|d| char::from_digit(*d, 10).unwrap_or('0')).collect())
    }

    /// `ean8()`: [`Code::gtin8`].
    pub fn ean8(&self) -> Result<String> {
        self.gtin8()
    }

    /// Seven random digits and the GTIN check digit.
    pub fn gtin8(&self) -> Result<String> {
        self.gtin(r"\d{7}", &GTIN_8_CHECK_DIGITS)
    }

    /// Twelve random digits and the GTIN check digit.
    pub fn gtin13(&self) -> Result<String> {
        self.gtin(r"\d{12}", &GTIN_13_CHECK_DIGITS)
    }

    /// `ean13()`: [`Code::gtin13`].
    pub fn ean13(&self) -> Result<String> {
        self.gtin13()
    }

    fn gtin(&self, regex: &str, check_digits: &[u32]) -> Result<String> {
        let mut value = self.faker.regexify(regex)?;
        let sum: u32 = value.chars().zip(check_digits).map(|(c, w)| c.to_digit(10).unwrap_or(0) * w).sum();
        let check = 10 - sum % 10;
        value.push(char::from_digit(if check == 10 { 0 } else { check }, 10).unwrap_or('0'));
        Ok(value)
    }
}

/// `Integer.parseInt` of one character of an ISBN being built (a YAML override can put anything there).
fn digit(c: char) -> Result<u32> {
    c.to_digit(10).ok_or_else(|| crate::Error::new("NumberFormatException", format!("For input string: \"{c}\"")))
}

fn isbn10_check_digit(isbn: &str) -> Result<u32> {
    let mut sum = 0u32;
    for (i, c) in isbn.chars().filter(|c| *c != '-').enumerate() {
        sum = sum.wrapping_add((i as u32 + 1) * digit(c)?);
    }
    Ok(sum % 11)
}

fn isbn13_check_digit(isbn: &str) -> Result<u32> {
    let mut sum = 0u32;
    for (i, c) in isbn.chars().filter(|c| *c != '-').enumerate() {
        sum += if i % 2 == 0 { 1 } else { 3 } * digit(c)?;
    }
    Ok((10 - sum % 10) % 10)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("isbnGs1", &[], |f, _| s(Code::new(f).isbn_gs1())),
    Method::new("isbnGroup", &[], |f, _| s(Code::new(f).isbn_group())),
    Method::new("isbnRegistrant", &[], |f, _| s(Code::new(f).isbn_registrant())),
    Method::new("isbn10", &[], |f, _| s(Code::new(f).isbn10())),
    Method::new("isbn10", &[P::Boolean], |f, a| s(Code::new(f).isbn10_with(a[0].as_bool()))),
    Method::new("isbn13", &[], |f, _| s(Code::new(f).isbn13())),
    Method::new("isbn13", &[P::Boolean], |f, a| s(Code::new(f).isbn13_with(a[0].as_bool()))),
    Method::new("asin", &[], |f, _| s(Code::new(f).asin())),
    Method::new("imei", &[], |f, _| s(Code::new(f).imei())),
    Method::new("ean8", &[], |f, _| s(Code::new(f).ean8())),
    Method::new("gtin8", &[], |f, _| s(Code::new(f).gtin8())),
    Method::new("gtin13", &[], |f, _| s(Code::new(f).gtin13())),
    Method::new("ean13", &[], |f, _| s(Code::new(f).ean13())),
];
