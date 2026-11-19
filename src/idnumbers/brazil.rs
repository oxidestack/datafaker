//! `net.datafaker.idnumbers.BrazilIdNumber`.

use super::{IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, gender, pt_br};
use crate::{Faker, Result};

/// Brazilian individual taxpayer numbers (CPF), formatted `###.###.###-##`.
#[derive(Debug, Clone, Copy, Default)]
pub struct BrazilIdNumber;

impl IdNumberGenerator for BrazilIdNumber {
    fn country_code(&self) -> &'static str {
        "BR"
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        pt_br::cpf(faker, true, false)
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let id_number = pt_br::cpf(faker, true, true)?;
        let birth_date = birthday(faker, request)?;
        Ok(PersonIdNumber { id_number, birth_date, gender: gender(faker, request) })
    }
}
