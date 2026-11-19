//! `net.datafaker.idnumbers.PortugueseIdNumber`.

use super::{IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, gender, numeric_value};
use crate::{Faker, Result};

const VALID_FIRST_DIGITS: [char; 6] = ['1', '2', '3', '5', '6', '8'];
const VALID_FIRST_DOUBLE_DIGITS: [&str; 12] = ["45", "70", "71", "72", "74", "75", "77", "79", "90", "91", "98", "99"];

/// Portuguese tax identification numbers (NIF): eight digits with a valid one- or two-digit prefix and a mod-11
/// check digit.
#[derive(Debug, Clone, Copy, Default)]
pub struct PortugueseIdNumber;

/// `calculateDigitSum(numbers)`: the check digit, 11 minus the weighted sum (weights 9 down) mod 11, 0 for a
/// remainder of 0 or 1.
fn digit_sum(numbers: &str) -> i32 {
    let sum: i32 = numbers.chars().enumerate().map(|(i, c)| (9 - i as i32) * numeric_value(c)).sum();
    let remainder = sum - sum / 11 * 11;
    if remainder == 0 || remainder == 1 { 0 } else { 11 - remainder }
}

impl IdNumberGenerator for PortugueseIdNumber {
    fn country_code(&self) -> &'static str {
        "PT"
    }

    /// Eight random digits and their check digit plus 5 (one or two digits).
    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let digits = faker.number().digits(8);
        Ok(format!("{digits}{}", digit_sum(&digits) + 5))
    }

    fn generate_valid(&self, faker: &Faker) -> Result<String> {
        let digits = if faker.random().next_boolean() {
            let first_digit = faker.options().option(&VALID_FIRST_DIGITS)?;
            format!("{first_digit}{}", faker.number().digits(7))
        } else {
            let first_double_digit = faker.options().option(&VALID_FIRST_DOUBLE_DIGITS)?;
            format!("{first_double_digit}{}", faker.number().digits(6))
        };
        Ok(format!("{digits}{}", digit_sum(&digits)))
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let id_number = self.generate_valid(faker)?;
        let birth_date = birthday(faker, request)?;
        Ok(PersonIdNumber { id_number, birth_date, gender: gender(faker, request) })
    }
}
