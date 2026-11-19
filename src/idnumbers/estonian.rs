//! `net.datafaker.idnumbers.EstonianIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, format_date, gender, numeric_value, random_gender};
use crate::java::LocalDate;
use crate::{Error, Faker, Result};

const CHECKSUM_COEFFICIENTS: [i32; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 1];
const CHECKSUM_COEFFICIENTS2: [i32; 10] = [3, 4, 5, 6, 7, 8, 9, 1, 2, 3];

/// Estonian personal identification codes (isikukood): a digit for century and gender, `yyMMdd`, three digits and a
/// mod-11 check digit.
#[derive(Debug, Clone, Copy, Default)]
pub struct EstonianIdNumber;

impl EstonianIdNumber {
    fn base_part(faker: &Faker, birthday: LocalDate, gender: Gender) -> Result<String> {
        Ok(format!("{}{}{}", Self::first_digit(birthday.year(), gender)?, format_date(birthday, "yyMMdd")?, faker.number().digits(3)))
    }

    /// `firstDigit(birthYear, gender)`: 1, 3, 5, 7 for men born in the 1800s to 2100s, one more for women.
    pub fn first_digit(birth_year: i32, gender: Gender) -> Result<i32> {
        let digit = match birth_year / 100 {
            18 => 1,
            19 => 3,
            20 => 5,
            21 => 7,
            _ => return Err(Error::new("IllegalStateException", format!("Birth year {birth_year} is out of allowed range [1800, 2199]"))),
        };
        Ok(match gender {
            Gender::Female => digit + 1,
            Gender::Male => digit,
        })
    }

    /// `checksum(numbers)`: mod 11 with the first weights, or the second when that is 10 (then mod 10).
    pub fn checksum(numbers: &str) -> i32 {
        let checksum = Self::checksum_with(numbers, &CHECKSUM_COEFFICIENTS);
        if checksum != 10 { checksum } else { Self::checksum_with(numbers, &CHECKSUM_COEFFICIENTS2) % 10 }
    }

    fn checksum_with(numbers: &str, coefficients: &[i32]) -> i32 {
        numbers.chars().zip(coefficients).map(|(c, k)| k * numeric_value(c)).sum::<i32>() % 11
    }
}

impl IdNumberGenerator for EstonianIdNumber {
    fn country_code(&self) -> &'static str {
        "EE"
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let birthday = faker.time_and_date().birthday()?;
        let digits = Self::base_part(faker, birthday, random_gender(faker))?;
        Ok(format!("{digits}{}", (Self::checksum(&digits) + 1) % 10))
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let gender = gender(faker, request);
        let digits = Self::base_part(faker, birth_date, gender)?;
        let id_number = format!("{digits}{}", Self::checksum(&digits));
        Ok(PersonIdNumber { id_number, birth_date, gender })
    }
}
