//! `net.datafaker.providers.base.Number`.

use crate::dispatch::{JValue, Method, P};
use crate::java::BigDecimal;
use crate::random_service::Range;
use crate::{Error, Faker, Result};

/// Random numbers.
#[derive(Clone, Copy)]
pub struct Number<'a> {
    faker: &'a Faker,
}

impl<'a> Number<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Number { faker }
    }

    /// A digit, 0 to 9.
    pub fn random_digit(&self) -> i32 {
        self.faker.random().next_int_between(0, 9).expect("0 <= 9")
    }

    /// `length` random digits.
    pub fn random_digits(&self, length: usize) -> Vec<i32> {
        (0..length).map(|_| self.random_digit()).collect()
    }

    /// A digit, 1 to 9.
    pub fn random_digit_not_zero(&self) -> i32 {
        self.faker.random().next_int_between(1, 9).expect("1 <= 9")
    }

    /// `positive()`: `numberBetween(1, Integer.MAX_VALUE)`.
    pub fn positive(&self) -> i32 {
        self.number_between(1, i32::MAX)
    }

    /// `negative()`: `numberBetween(0, Integer.MIN_VALUE)`, a value at or below 0.
    pub fn negative(&self) -> i32 {
        self.number_between(0, i32::MIN)
    }

    /// `numberBetween(int min, int max)`: `min` inclusive, `max` exclusive (the bounds may come in either order).
    pub fn number_between(&self, min: i32, max: i32) -> i32 {
        if min == max {
            return min;
        }
        let real_min = min.min(max);
        let real_max = min.max(max);
        let amplitude = real_max.wrapping_sub(real_min);
        if amplitude >= 0 {
            return self.faker.random().next_int_bounded(amplitude).expect("positive amplitude") + real_min;
        }
        self.number_between_long(real_min as i64, real_max as i64).expect("range fits") as i32
    }

    /// `numberBetween(double min, double max)`.
    pub fn number_between_double(&self, min: f64, max: f64) -> f64 {
        self.faker.random().next_double_between(min, max)
    }

    /// `numberBetween(long min, long max)`: `min` inclusive, `max` exclusive (either order). A range wider than
    /// `Long.MAX_VALUE` goes through `decimalBetween`, so its upper end can come out too.
    pub fn number_between_long(&self, min: i64, max: i64) -> Result<i64> {
        if min == max {
            return Ok(min);
        }
        let real_min = min.min(max);
        let real_max = min.max(max);
        let amplitude = real_max.wrapping_sub(real_min);
        if amplitude >= 0 {
            return self.faker.random().next_long_range(Range::inclusive_exclusive(real_min, real_max)?);
        }
        Ok(self.decimal_between(real_min, real_max).long_value())
    }

    /// `randomDouble(maxNumberOfDecimals, int min, int max)`.
    pub fn random_double_int(&self, max_number_of_decimals: i32, min: i32, max: i32) -> f64 {
        self.random_double(max_number_of_decimals, min as i64, max as i64)
    }

    /// `randomDouble(maxNumberOfDecimals, long min, long max)`: `min + (max - min) * nextDouble()` in exact decimal
    /// arithmetic, rounded half-down to `maxNumberOfDecimals` places.
    pub fn random_double(&self, max_number_of_decimals: i32, min: i64, max: i64) -> f64 {
        self.decimal_between(min, max).set_scale_half_down(max_number_of_decimals).double_value()
    }

    /// `decimalBetween(min, max)` (private upstream): `min` itself, without a draw, when the bounds are equal.
    pub fn decimal_between(&self, min: i64, max: i64) -> BigDecimal {
        if min == max {
            return BigDecimal::value_of_long(min);
        }
        let true_min = BigDecimal::value_of_long(min);
        let true_max = BigDecimal::value_of_long(max);
        let random = BigDecimal::value_of_double(self.faker.random().next_double()).expect("a finite double");
        true_min.add(&true_max.subtract(&true_min).multiply(&random))
    }

    /// `randomNumber(numberOfDigits)`: a number with exactly that many digits.
    pub fn random_number_digits(&self, number_of_digits: i32) -> Result<i64> {
        if number_of_digits <= 0 {
            return Err(Error::illegal_argument("Number of digits must be positive"));
        }
        let min = pow(10, number_of_digits - 1);
        let max = min.wrapping_mul(10);
        self.faker.random().next_long_between(min, max)
    }

    /// `randomNumber()`.
    pub fn random_number(&self) -> i64 {
        self.faker.random().next_long()
    }

    /// `digits(count)`.
    pub fn digits(&self, count: usize) -> String {
        self.faker.random().next_random_bytes(count).iter().map(|b| (b'0' + ((*b as i32).abs() % 10) as u8) as char).collect()
    }

    /// `digit()`: one random digit, as a string.
    pub fn digit(&self) -> String {
        self.digits(1)
    }
}

fn pow(value: i64, d: i32) -> i64 {
    match d {
        0 => 1,
        1 => value,
        _ if d & 1 == 0 => {
            let p = pow(value, d >> 1);
            p.wrapping_mul(p)
        }
        _ => value.wrapping_mul(pow(value, d - 1)),
    }
}

fn n(f: &Faker) -> Number<'_> {
    f.number()
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("randomDigit", &[], |f, _| Ok(JValue::Int(n(f).random_digit()))),
    Method::new("randomDigits", &[P::Int], |f, a| {
        Ok(JValue::List(n(f).random_digits(a[0].as_int().max(0) as usize).into_iter().map(JValue::Int).collect()))
    }),
    Method::new("randomDigitNotZero", &[], |f, _| Ok(JValue::Int(n(f).random_digit_not_zero()))),
    Method::new("positive", &[], |f, _| Ok(JValue::Int(n(f).positive()))),
    Method::new("negative", &[], |f, _| Ok(JValue::Int(n(f).negative()))),
    Method::new("numberBetween", &[P::Int, P::Int], |f, a| Ok(JValue::Int(n(f).number_between(a[0].as_int(), a[1].as_int())))),
    Method::new("numberBetween", &[P::Double, P::Double], |f, a| {
        Ok(JValue::Double(n(f).number_between_double(a[0].as_double(), a[1].as_double())))
    }),
    Method::new("numberBetween", &[P::Long, P::Long], |f, a| Ok(JValue::Long(n(f).number_between_long(a[0].as_long(), a[1].as_long())?))),
    Method::new("randomNumber", &[P::Int, P::Boolean], |f, a| Ok(JValue::Long(n(f).random_number_digits(a[0].as_int())?))),
    Method::new("randomNumber", &[P::Int], |f, a| Ok(JValue::Long(n(f).random_number_digits(a[0].as_int())?))),
    Method::new("randomNumber", &[], |f, _| Ok(JValue::Long(n(f).random_number()))),
    Method::new("randomDouble", &[P::Int, P::Int, P::Int], |f, a| {
        Ok(JValue::Double(n(f).random_double_int(a[0].as_int(), a[1].as_int(), a[2].as_int())))
    }),
    Method::new("randomDouble", &[P::Int, P::Long, P::Long], |f, a| {
        Ok(JValue::Double(n(f).random_double(a[0].as_int(), a[1].as_long(), a[2].as_long())))
    }),
    Method::new("digits", &[P::Int], |f, a| Ok(JValue::Str(n(f).digits(a[0].as_int().max(0) as usize)))),
    Method::new("digit", &[], |f, _| Ok(JValue::Str(n(f).digit()))),
];
