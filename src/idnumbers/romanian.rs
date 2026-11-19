//! `net.datafaker.idnumbers.RomanianIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, format_date, gender, multiply, random_gender};
use crate::java::LocalDate;
use crate::{Error, Faker, Result};

const CHECKSUM_WEIGHTS: [i32; 12] = [2, 7, 9, 1, 4, 6, 3, 5, 8, 2, 7, 9];

/// Romanian personal numeric codes (CNP): a digit for gender and century, `yyMMdd`, a county code (01–46, 51, 52),
/// a sequence number 001–999 and a mod-11 check digit.
#[derive(Debug, Clone, Copy, Default)]
pub struct RomanianIdNumber;

impl RomanianIdNumber {
    fn base_part(&self, faker: &Faker, birthday: LocalDate, gender: Gender) -> Result<String> {
        Ok(format!("{}{}{}{}", self.first_character(birthday, gender)?, self.date_of_birth(birthday)?, self.county_code(faker), self.sequence_number(faker)))
    }

    /// `firstCharacter(birthday, gender)`: 3/4 for the 1800s, 1/2 for the 1900s, 5/6 for the 2000s (men/women).
    pub fn first_character(&self, birthday: LocalDate, gender: Gender) -> Result<i32> {
        let digit = match birthday.year() / 100 {
            18 => 3,
            19 => 1,
            20 => 5,
            _ => return Err(Error::illegal_argument(format!("Too far in the past or future: {birthday}"))),
        };
        Ok(match gender {
            Gender::Female => digit + 1,
            Gender::Male => digit,
        })
    }

    /// `dateOfBirth(birthday)`: `yyMMdd`.
    pub fn date_of_birth(&self, birthday: LocalDate) -> Result<String> {
        format_date(birthday, "yyMMdd")
    }

    /// `countyCode(faker)`: 01–46, or 51 or 52.
    pub fn county_code(&self, faker: &Faker) -> String {
        let county_code = if faker.bool().bool() { faker.number().number_between(1, 47) } else { faker.number().number_between(51, 53) };
        format!("{county_code:02}")
    }

    /// `sequenceNumber(faker)`: 001–999.
    pub fn sequence_number(&self, faker: &Faker) -> String {
        format!("{:03}", faker.number().number_between(1, 1_000))
    }

    /// `checksum(basePart)`: the weighted sum mod 11, 1 for 10.
    pub fn checksum(&self, base_part: &str) -> i32 {
        let result = multiply(base_part, &CHECKSUM_WEIGHTS) % 11;
        if result == 10 { 1 } else { result }
    }
}

impl IdNumberGenerator for RomanianIdNumber {
    fn country_code(&self) -> &'static str {
        "RO"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let gender = gender(faker, request);
        let base_part = self.base_part(faker, birth_date, gender)?;
        let id_number = format!("{base_part}{}", self.checksum(&base_part));
        Ok(PersonIdNumber { id_number, birth_date, gender })
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let birthday = faker.time_and_date().birthday()?;
        let gender = random_gender(faker);
        let base_part = self.base_part(faker, birthday, gender)?;
        Ok(format!("{base_part}{}", (self.checksum(&base_part) + 1) % 10))
    }
}
