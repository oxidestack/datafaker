//! `net.datafaker.idnumbers.NorwegianIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, format_date, gender, numeric_value};
use crate::java::LocalDate;
use crate::{Error, Faker, Result};

const CHECKSUM_COEFFICIENTS_K1: [i32; 9] = [3, 7, 6, 1, 8, 9, 4, 5, 2];
const CHECKSUM_COEFFICIENTS_K2: [i32; 10] = [5, 4, 3, 2, 7, 6, 5, 4, 3, 2];

/// Norwegian national identity numbers (fødselsnummer) `ddMMyyCCGkk`: the birthday, a sequence whose range depends
/// on the century, a digit even for women and odd for men, and two mod-11 check digits.
#[derive(Debug, Clone, Copy, Default)]
pub struct NorwegianIdNumber;

impl NorwegianIdNumber {
    /// `basePart(faker, birthday, gender)`: the nine digits before the check digits.
    pub fn base_part(&self, faker: &Faker, birthday: LocalDate, gender: Gender) -> Result<String> {
        let birthday_digits = format_date(birthday, "ddMMyy")?;
        let sequence_number = Self::sequence_number(faker, birthday.year())?;
        let gender_digit = match gender {
            Gender::Female => faker.options().option(&[0, 2, 4, 6, 8])?,
            Gender::Male => faker.options().option(&[1, 3, 5, 7, 9])?,
        };
        Ok(format!("{birthday_digits}{sequence_number:02}{gender_digit}"))
    }

    /// `generateSequenceNumber(faker, year)`: 50–74 for 1854–1899, 90–99 for 1940–1999, 0–49 for 1900–1939,
    /// 50–99 for 2000–2039.
    fn sequence_number(faker: &Faker, year: i32) -> Result<i32> {
        let random = faker.random();
        match year {
            1854..=1899 => random.next_int_between(50, 74),
            1940..=1999 => random.next_int_between(90, 99),
            1900..=1999 => random.next_int_between(0, 49),
            2000..=2039 => random.next_int_between(50, 99),
            _ => Err(Error::illegal_argument("Birthday is not in range of supported in Norway")),
        }
    }

    /// `checksum(numbers)`: the two check digits as a number, `k1 * 10 + k2`.
    pub fn checksum(&self, numbers: &str) -> i32 {
        let k1 = Self::modulo11(numbers, &CHECKSUM_COEFFICIENTS_K1);
        let k2 = Self::modulo11(&format!("{numbers}{k1}"), &CHECKSUM_COEFFICIENTS_K2);
        k1 * 10 + k2
    }

    fn modulo11(numbers: &str, coefficients: &[i32]) -> i32 {
        let sum: i32 = numbers.chars().zip(coefficients).map(|(c, k)| k * numeric_value(c)).sum();
        (11 - sum % 11) % 10
    }
}

impl IdNumberGenerator for NorwegianIdNumber {
    fn country_code(&self) -> &'static str {
        "NO"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let gender = gender(faker, request);
        let base_part = self.base_part(faker, birth_date, gender)?;
        let id_number = format!("{base_part}{:02}", self.checksum(&base_part));
        Ok(PersonIdNumber { id_number, birth_date, gender })
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let valid: Vec<char> = self.generate_valid(faker)?.chars().collect();
        let base_part: String = valid[..valid.len() - 2].iter().collect();
        let invalid_checksum = (self.checksum(&base_part) + 1) % 100;
        Ok(format!("{base_part}{invalid_checksum:02}"))
    }
}
