//! `net.datafaker.idnumbers.BulgarianIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, gender, multiply, random_gender};
use crate::java::LocalDate;
use crate::{Faker, Result};

const CHECKSUM_WEIGHTS: [i32; 9] = [2, 4, 8, 5, 10, 9, 7, 3, 6];
const EVEN_DIGITS: [i32; 5] = [0, 2, 4, 6, 8];
const ODD_DIGITS: [i32; 5] = [1, 3, 5, 7, 9];

/// Bulgarian unique citizenship numbers (EGN): `YYMMDD`, the month plus 20 before 1900 and plus 40 from 2000, three
/// digits whose last is even for men, and a mod-11 check digit.
#[derive(Debug, Clone, Copy, Default)]
pub struct BulgarianIdNumber;

impl BulgarianIdNumber {
    fn base_part(&self, faker: &Faker, birth_date: LocalDate, gender: Gender) -> String {
        let year = birth_date.year();
        let month_addition = if year < 1900 {
            20
        } else if year >= 2000 {
            40
        } else {
            0
        };
        let available_last_digits = match gender {
            Gender::Female => ODD_DIGITS,
            Gender::Male => EVEN_DIGITS,
        };
        let last_digit = available_last_digits[faker.number().number_between(0, 5) as usize];
        format!(
            "{:02}{:02}{:02}{}{last_digit}",
            year % 100,
            birth_date.month_value() as i32 + month_addition,
            birth_date.day_of_month(),
            faker.number().digits(2)
        )
    }

    /// `checksum(text)`: the weighted sum mod 11, mod 10.
    pub fn checksum(&self, text: &str) -> i32 {
        multiply(text, &CHECKSUM_WEIGHTS) % 11 % 10
    }
}

impl IdNumberGenerator for BulgarianIdNumber {
    fn country_code(&self) -> &'static str {
        "BG"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let gender = gender(faker, request);
        let base_part = self.base_part(faker, birth_date, gender);
        let id_number = format!("{base_part}{}", self.checksum(&base_part));
        Ok(PersonIdNumber { id_number, birth_date, gender })
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let birth_date = faker.time_and_date().birthday()?;
        let base_part = self.base_part(faker, birth_date, random_gender(faker));
        Ok(format!("{base_part}{}", (self.checksum(&base_part) + 1) % 10))
    }
}
