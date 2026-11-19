//! `net.datafaker.idnumbers.UkrainianIdNumber`.

use super::{IdNumberGenerator, IdNumberRequest, PersonIdNumber, format_date, gender, multiply};
use crate::{Faker, Result};

const CHECKSUM_WEIGHTS: [i32; 12] = [7, 3, 1, 7, 3, 1, 7, 3, 1, 7, 3, 1];

/// Ukrainian unique registry record numbers (UNZR) `yyyyMMdd-NNNNC`: the birthday, four digits and a check digit
/// (weights 7, 3, 1). The request's ages are ignored upstream: an 18 to 65 years old person.
#[derive(Debug, Clone, Copy, Default)]
pub struct UkrainianIdNumber;

impl IdNumberGenerator for UkrainianIdNumber {
    fn country_code(&self) -> &'static str {
        "UA"
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let birthday = faker.time_and_date().birthday()?;
        let dob = format_date(birthday, "yyyyMMdd")?;
        let numbers = faker.numerify("####");
        let checksum = (multiply(&format!("{dob}{numbers}"), &CHECKSUM_WEIGHTS) + 1) % 10;
        Ok(format!("{dob}-{numbers}{checksum}"))
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = faker.time_and_date().birthday()?;
        let dob = format_date(birth_date, "yyyyMMdd")?;
        let numbers = faker.numerify("####");
        let checksum = multiply(&format!("{dob}{numbers}"), &CHECKSUM_WEIGHTS) % 10;
        Ok(PersonIdNumber { id_number: format!("{dob}-{numbers}{checksum}"), birth_date, gender: gender(faker, request) })
    }
}
