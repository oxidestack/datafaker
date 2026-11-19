//! `net.datafaker.idnumbers.HungarianIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, format_date, numeric_value, random_gender};
use crate::java::LocalDate;
use crate::{Faker, Result};

/// Hungarian personal identification numbers `GYYMMDDXXC`: a digit for century and gender, the birthday, a serial
/// number and a mod-11 check digit (numbers whose check would be 10 are not issued). The request's ages and gender
/// are ignored upstream: an 18 to 65 years old person of a random gender.
#[derive(Debug, Clone, Copy, Default)]
pub struct HungarianIdNumber;

impl HungarianIdNumber {
    fn base_part(faker: &Faker, birthday: LocalDate, gender: Gender) -> Result<String> {
        Ok(format!("{}{}{}", Self::first_digit(birthday.year(), gender), format_date(birthday, "yyMMdd")?, faker.number().digits(2)))
    }

    /// `getCheckDigit(basePart)`: the sum of each digit times its position (from 1), mod 11.
    pub fn get_check_digit(base_part: &str) -> i32 {
        base_part.chars().enumerate().map(|(i, c)| numeric_value(c) * (i as i32 + 1)).sum::<i32>() % 11
    }

    /// `firstDigit(birthYear, gender)`: 1 (men) or 2 (women) for 1900–1999, else 3 or 4.
    pub fn first_digit(birth_year: i32, gender: Gender) -> i32 {
        let in_range = (1900..=1999).contains(&birth_year);
        match gender {
            Gender::Male => if in_range { 1 } else { 3 },
            Gender::Female => if in_range { 2 } else { 4 },
        }
    }
}

impl IdNumberGenerator for HungarianIdNumber {
    fn country_code(&self) -> &'static str {
        "HU"
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let valid: Vec<char> = self.generate_valid(faker)?.chars().collect();
        let base_part: String = valid[..9].iter().collect();
        let invalid_check_digit = (numeric_value(valid[9]) + 1) % 10;
        Ok(format!("{base_part}{invalid_check_digit}"))
    }

    fn generate_valid_for(&self, faker: &Faker, _request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = faker.time_and_date().birthday()?;
        let gender = random_gender(faker);
        loop {
            let base_part = Self::base_part(faker, birth_date, gender)?;
            let check_digit = Self::get_check_digit(&base_part);
            if check_digit != 10 {
                return Ok(PersonIdNumber { id_number: format!("{base_part}{check_digit}"), birth_date, gender });
            }
        }
    }
}
