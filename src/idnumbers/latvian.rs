//! `net.datafaker.idnumbers.LatvianIdNumber`.

use super::{IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, format_date, gender, numeric_value};
use crate::java::LocalDate;
use crate::{Error, Faker, Result};

const CHECKSUM_COEFFICIENTS: [i32; 10] = [1, 6, 3, 7, 9, 10, 5, 8, 4, 2];

/// Latvian personal codes: `ddMMyy-CNNNK` (century digit, three digits, check digit) for people born before
/// 2017-07-01, then `3` and ten random digits.
#[derive(Debug, Clone, Copy, Default)]
pub struct LatvianIdNumber;

impl LatvianIdNumber {
    fn base_part(faker: &Faker, birthday: LocalDate) -> Result<String> {
        Ok(format!("{}-{}{}", format_date(birthday, "ddMMyy")?, Self::century_digit(birthday.year())?, faker.number().digits(3)))
    }

    /// `centuryDigit(birthYear)`: 0, 1, 2 for the 1800s, 1900s, 2000s.
    pub fn century_digit(birth_year: i32) -> Result<i32> {
        match birth_year / 100 {
            18 => Ok(0),
            19 => Ok(1),
            20 => Ok(2),
            _ => Err(Error::new("IllegalStateException", format!("Birth year {birth_year} is out of allowed range [1800, 2017]"))),
        }
    }

    /// `checksum(numbers)`: `(1101 - weighted sum) mod 11 mod 10` over the digits (the `-` dropped).
    pub fn checksum(numbers: &str) -> i32 {
        let sum: i32 = numbers.chars().filter(|c| *c != '-').zip(CHECKSUM_COEFFICIENTS).map(|(c, k)| k * numeric_value(c)).sum();
        (1101 - sum) % 11 % 10
    }
}

impl IdNumberGenerator for LatvianIdNumber {
    fn country_code(&self) -> &'static str {
        "LV"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let gender = gender(faker, request);
        let change_to_new_format = LocalDate::of(2017, 7, 1)?;
        let id_number = if birth_date.is_before(&change_to_new_format) {
            let digits = Self::base_part(faker, birth_date)?;
            format!("{digits}{}", Self::checksum(&digits))
        } else {
            format!("3{}", faker.number().number_between_long(2_000_000_000, 9_999_999_999)?)
        };
        Ok(PersonIdNumber { id_number, birth_date, gender })
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let birthday = faker.time_and_date().birthday()?;
        let digits = Self::base_part(faker, birthday)?;
        Ok(format!("{digits}{}", (Self::checksum(&digits) + 1) % 10))
    }
}
