//! `net.datafaker.providers.base.Barcode`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Barcode numbers (EAN, GTIN) as `long`s, and barcode types.
#[derive(Clone, Copy)]
pub struct Barcode<'a> {
    faker: &'a Faker,
}

impl<'a> Barcode<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Barcode { faker }
    }

    /// A 13-digit EAN (leading zeros dropped, as a `long`).
    pub fn ean13(&self) -> Result<i64> {
        self.ean(13)
    }

    /// An 8-digit EAN.
    pub fn ean8(&self) -> Result<i64> {
        self.ean(8)
    }

    /// A 14-digit GTIN.
    pub fn gtin14(&self) -> Result<i64> {
        self.ean(14)
    }

    /// A 12-digit GTIN.
    pub fn gtin12(&self) -> Result<i64> {
        self.ean(12)
    }

    /// `gtin13()`: [`Barcode::ean13`].
    pub fn gtin13(&self) -> Result<i64> {
        self.ean13()
    }

    /// `gtin8()`: [`Barcode::ean8`].
    pub fn gtin8(&self) -> Result<i64> {
        self.ean8()
    }

    /// `barcode.types`.
    pub fn r#type(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Barcode, "barcode.types")
    }

    fn ean(&self, length: i32) -> Result<i64> {
        let first_part = self.faker.number().random_number_digits(length - 1)?;
        let (mut odd, mut even, mut i) = (0i32, 0i32, 0);
        let mut number = first_part;
        while number > 0 {
            i += 1;
            if i % 2 == 1 {
                odd += (number % 10) as i32;
            } else {
                even += (number % 10) as i32;
            }
            number /= 10;
        }
        if i % 2 == 0 {
            std::mem::swap(&mut odd, &mut even);
        }
        let var = if length == 13 { odd + even + (even << 1) } else { odd + even + (odd << 1) };
        let rounded = if var % 10 == 0 { var } else { var + 10 - var % 10 };
        let check_digit = rounded - var;
        let mut product = 10i64;
        while product <= check_digit as i64 {
            product *= 10;
        }
        Ok(first_part.wrapping_mul(product).wrapping_add(check_digit as i64))
    }
}

fn l(r: Result<i64>) -> Result<JValue> {
    r.map(JValue::Long)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("ean13", &[], |f, _| l(Barcode::new(f).ean13())),
    Method::new("ean8", &[], |f, _| l(Barcode::new(f).ean8())),
    Method::new("gtin14", &[], |f, _| l(Barcode::new(f).gtin14())),
    Method::new("gtin12", &[], |f, _| l(Barcode::new(f).gtin12())),
    Method::new("gtin13", &[], |f, _| l(Barcode::new(f).gtin13())),
    Method::new("gtin8", &[], |f, _| l(Barcode::new(f).gtin8())),
    Method::new("type", &[], |f, _| s(Barcode::new(f).r#type())),
];
