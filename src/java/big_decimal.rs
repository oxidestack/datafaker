//! `java.math.BigDecimal`, as far as `Number.decimalBetween` and `Number.randomDouble` use it: `valueOf(long)`,
//! `valueOf(double)`, `add`, `subtract`, `multiply`, `setScale(n, HALF_DOWN)`, `doubleValue` and `longValue`.

use std::fmt;

use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive, Zero};

/// An exact decimal: `unscaled × 10^-scale`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigDecimal {
    unscaled: BigInt,
    scale: i32,
}

fn ten_pow(n: u32) -> BigInt {
    BigInt::from(10).pow(n)
}

impl BigDecimal {
    /// `BigDecimal.valueOf(long)`: scale 0.
    pub fn value_of_long(v: i64) -> BigDecimal {
        BigDecimal { unscaled: BigInt::from(v), scale: 0 }
    }

    /// `BigDecimal.valueOf(double)`: `new BigDecimal(Double.toString(d))`, so `0.1` is exactly one tenth and
    /// `1.0E-5` has scale 6. `None` for NaN and the infinities (a `NumberFormatException` in Java).
    pub fn value_of_double(d: f64) -> Option<BigDecimal> {
        if !d.is_finite() {
            return None;
        }
        BigDecimal::parse(&crate::java::double_to_string(d))
    }

    /// `new BigDecimal(String)` for the plain and `E` forms that `Double.toString` prints.
    pub fn parse(s: &str) -> Option<BigDecimal> {
        let (mantissa, exponent) = match s.find(['e', 'E']) {
            Some(i) => (&s[..i], s[i + 1..].parse::<i32>().ok()?),
            None => (s, 0),
        };
        let (int_part, frac_part) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        let digits = format!("{int_part}{frac_part}");
        let unscaled: BigInt = digits.parse().ok()?;
        Some(BigDecimal { unscaled, scale: frac_part.len() as i32 - exponent })
    }

    /// `BigDecimal.scale()`: the number of digits after the decimal point.
    pub fn scale(&self) -> i32 {
        self.scale
    }

    /// `BigDecimal.unscaledValue()`.
    pub fn unscaled_value(&self) -> &BigInt {
        &self.unscaled
    }

    /// The unscaled value rescaled up to `scale` (which must not be below the current one).
    fn widened(&self, scale: i32) -> BigInt {
        &self.unscaled * ten_pow((scale - self.scale) as u32)
    }

    /// `add(augend)`: the scale is the larger of the two.
    pub fn add(&self, other: &BigDecimal) -> BigDecimal {
        let scale = self.scale.max(other.scale);
        BigDecimal { unscaled: self.widened(scale) + other.widened(scale), scale }
    }

    /// `subtract(subtrahend)`: the scale is the larger of the two.
    pub fn subtract(&self, other: &BigDecimal) -> BigDecimal {
        let scale = self.scale.max(other.scale);
        BigDecimal { unscaled: self.widened(scale) - other.widened(scale), scale }
    }

    /// `multiply(multiplicand)`: the scale is the sum of the two.
    pub fn multiply(&self, other: &BigDecimal) -> BigDecimal {
        BigDecimal { unscaled: &self.unscaled * &other.unscaled, scale: self.scale + other.scale }
    }

    /// `setScale(newScale, RoundingMode.HALF_DOWN)`: to the nearest, a tie towards zero.
    pub fn set_scale_half_down(&self, new_scale: i32) -> BigDecimal {
        if new_scale >= self.scale {
            return BigDecimal { unscaled: self.widened(new_scale), scale: new_scale };
        }
        let divisor = ten_pow((self.scale - new_scale) as u32);
        // `BigInt` division truncates towards zero, as Java's does.
        let quotient = &self.unscaled / &divisor;
        let remainder = &self.unscaled % &divisor;
        let twice = remainder.abs() * 2;
        let unscaled = if twice > divisor {
            if self.unscaled.is_negative() { quotient - 1 } else { quotient + 1 }
        } else {
            quotient
        };
        BigDecimal { unscaled, scale: new_scale }
    }

    /// `doubleValue()`: the nearest double, ties to even (correctly rounded since JDK 21).
    pub fn double_value(&self) -> f64 {
        format!("{}e{}", self.unscaled, -(self.scale as i64)).parse().expect("a decimal literal")
    }

    /// `longValue()`: the integer part, truncated towards zero, then its low 64 bits.
    pub fn long_value(&self) -> i64 {
        let integer = if self.scale <= 0 {
            &self.unscaled * ten_pow(self.scale.unsigned_abs())
        } else {
            &self.unscaled / ten_pow(self.scale as u32)
        };
        let modulus = BigInt::from(1u128 << 64);
        let mut low = integer % &modulus;
        if low.is_negative() {
            low += &modulus;
        }
        low.to_u64().expect("below 2^64") as i64
    }

    /// Whether the value is zero, at any scale.
    pub fn is_zero(&self) -> bool {
        self.unscaled.is_zero()
    }
}

impl fmt::Display for BigDecimal {
    /// `toPlainString()`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.scale <= 0 {
            return write!(f, "{}", &self.unscaled * ten_pow(self.scale.unsigned_abs()));
        }
        let negative = self.unscaled.is_negative();
        let digits = self.unscaled.abs().to_string();
        let scale = self.scale as usize;
        let padded = if digits.len() <= scale { format!("{}{digits}", "0".repeat(scale + 1 - digits.len())) } else { digits };
        let (int_part, frac_part) = padded.split_at(padded.len() - scale);
        write!(f, "{}{int_part}.{frac_part}", if negative { "-" } else { "" })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> BigDecimal {
        BigDecimal::parse(s).unwrap()
    }

    #[test]
    fn value_of_double_reads_double_to_string() {
        assert_eq!(BigDecimal::value_of_double(0.1).unwrap(), d("0.1"));
        assert_eq!(BigDecimal::value_of_double(1.0e-5).unwrap().scale(), 6);
        assert_eq!(BigDecimal::value_of_double(0.0).unwrap().to_string(), "0.0");
    }

    #[test]
    fn half_down() {
        assert_eq!(d("2.5").set_scale_half_down(0).to_string(), "2");
        assert_eq!(d("2.51").set_scale_half_down(0).to_string(), "3");
        assert_eq!(d("-2.5").set_scale_half_down(0).to_string(), "-2");
        assert_eq!(d("-2.51").set_scale_half_down(0).to_string(), "-3");
        assert_eq!(d("1.25").set_scale_half_down(3).to_string(), "1.250");
        assert_eq!(d("125").set_scale_half_down(-1).double_value(), 120.0);
    }

    #[test]
    fn long_value_truncates_and_wraps() {
        assert_eq!(d("-7.9").long_value(), -7);
        assert_eq!(d("18446744073709551617").long_value(), 1);
        assert_eq!(d("-9223372036854775808.5").long_value(), i64::MIN);
    }
}
