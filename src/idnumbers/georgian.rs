//! `net.datafaker.idnumbers.GeorgianIdNumber`.

use super::{IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, gender};
use crate::{Faker, Result};

/// Georgian personal numbers: eleven random digits (thirteen for an invalid one).
#[derive(Debug, Clone, Copy, Default)]
pub struct GeorgianIdNumber;

impl IdNumberGenerator for GeorgianIdNumber {
    fn country_code(&self) -> &'static str {
        "GE"
    }

    fn generate_valid(&self, faker: &Faker) -> Result<String> {
        Ok(faker.numerify("###########"))
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let id_number = self.generate_valid(faker)?;
        let birth_date = birthday(faker, request)?;
        Ok(PersonIdNumber { id_number, birth_date, gender: gender(faker, request) })
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        Ok(faker.numerify("###########42"))
    }
}
