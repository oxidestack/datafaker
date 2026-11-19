//! `net.datafaker.providers.base.Commerce`.

use super::{ProviderKind, format_fixed, resolve, s};
use crate::dispatch::{Method, P};
use crate::{Faker, Result};

/// Commerce: departments, products, prices and promotion codes.
#[derive(Clone, Copy)]
pub struct Commerce<'a> {
    faker: &'a Faker,
}

impl<'a> Commerce<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Commerce { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Commerce, key)
    }

    /// One to three distinct departments in `String` order: `A`, or `A, B & C`.
    pub fn department(&self) -> Result<String> {
        let number_of_departments = self.faker.random().next_int_bounded(4)?.max(1) as usize;
        // A `TreeSet<String>`: UTF-16 order.
        let mut departments: Vec<String> = Vec::new();
        while departments.len() < number_of_departments {
            let department = self.faker.resolve("commerce.department")?;
            if let Err(i) = departments.binary_search_by(|d| d.encode_utf16().cmp(department.encode_utf16())) {
                departments.insert(i, department);
            }
        }
        let last = departments.pop().expect("at least one department");
        Ok(if departments.is_empty() { last } else { format!("{} & {last}", departments.join(", ")) })
    }

    /// An adjective, a material and a product.
    pub fn product_name(&self) -> Result<String> {
        Ok([
            self.resolve("commerce.product_name.adjective")?,
            self.resolve("commerce.product_name.material")?,
            self.resolve("commerce.product_name.product")?,
        ]
        .join(" "))
    }

    /// `commerce.product_name.material`.
    pub fn material(&self) -> Result<String> {
        self.resolve("commerce.product_name.material")
    }

    /// `commerce.brand`.
    pub fn brand(&self) -> Result<String> {
        self.resolve("commerce.brand")
    }

    /// `commerce.vendor`.
    pub fn vendor(&self) -> Result<String> {
        self.resolve("commerce.vendor")
    }

    /// `price()`: between 0 and 100.
    pub fn price(&self) -> String {
        self.price_between(0.0, 100.0)
    }

    /// `price(min, max)`: `min + nextDouble() * (max - min)` as `#0.00` in the faker's locale.
    pub fn price_between(&self, min: f64, max: f64) -> String {
        let price = min + self.faker.random().next_double() * (max - min);
        format_fixed(price, 2, &self.faker.locale())
    }

    /// `promotionCode()`: six digits.
    pub fn promotion_code(&self) -> Result<String> {
        self.promotion_code_with(6)
    }

    /// `promotionCode(digits)`: a noun and `digits` digits, joined by an adjective.
    pub fn promotion_code_with(&self, digits: i32) -> Result<String> {
        let adjective = self.faker.resolve("commerce.promotion_code.adjective")?;
        let noun = self.resolve("commerce.promotion_code.noun")?;
        if digits < 0 {
            return Err(crate::Error::new("NegativeArraySizeException", digits.to_string()));
        }
        let number = self.faker.number().digits(digits as usize);
        Ok(format!("{noun}{adjective}{number}"))
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("department", &[], |f, _| s(Commerce::new(f).department())),
    Method::new("productName", &[], |f, _| s(Commerce::new(f).product_name())),
    Method::new("material", &[], |f, _| s(Commerce::new(f).material())),
    Method::new("brand", &[], |f, _| s(Commerce::new(f).brand())),
    Method::new("vendor", &[], |f, _| s(Commerce::new(f).vendor())),
    Method::new("price", &[], |f, _| s(Ok(Commerce::new(f).price()))),
    Method::new("price", &[P::Double, P::Double], |f, a| s(Ok(Commerce::new(f).price_between(a[0].as_double(), a[1].as_double())))),
    Method::new("promotionCode", &[], |f, _| s(Commerce::new(f).promotion_code())),
    Method::new("promotionCode", &[P::Int], |f, a| s(Commerce::new(f).promotion_code_with(a[0].as_int()))),
];
