//! `net.datafaker.idnumbers.MoldovanIdNumber`.

use super::{IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, multiply, random_gender};
use crate::java::LocalDate;
use crate::{Faker, Result};

const CHECKSUM_MASK: [i32; 12] = [7, 3, 1, 7, 3, 1, 7, 3, 1, 7, 3, 1];

/// Moldovan personal identification numbers (IDNP) `2YYYXXXNNNNNK`: `2`, the last three digits of the year, the
/// registrar's office, a registration number and a check digit (weights 7, 3, 1).
#[derive(Debug, Clone, Copy, Default)]
pub struct MoldovanIdNumber;

impl MoldovanIdNumber {
    fn base_part(faker: &Faker, birthday: LocalDate) -> String {
        format!("2{:03}{}{}", birthday.year() % 1000, faker.number().digits(3), faker.number().digits(5))
    }

    /// `checksum(text)`: the weighted sum mod 10, as a digit.
    pub fn checksum(&self, text: &str) -> char {
        char::from(b'0' + multiply(text, &CHECKSUM_MASK).rem_euclid(10) as u8)
    }
}

impl IdNumberGenerator for MoldovanIdNumber {
    fn country_code(&self) -> &'static str {
        "MD"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let base_part = Self::base_part(faker, birth_date);
        let id_number = format!("{base_part}{}", self.checksum(&base_part));
        Ok(PersonIdNumber { id_number, birth_date, gender: random_gender(faker) })
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let base_part = Self::base_part(faker, faker.time_and_date().birthday()?);
        // Upstream adds 1 to the check digit's `char` code, not to its value.
        Ok(format!("{base_part}{}", (self.checksum(&base_part) as i32 + 1) % 10))
    }
}
