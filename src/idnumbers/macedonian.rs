//! `net.datafaker.idnumbers.MacedonianIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, digit_at, gender, random_gender};
use crate::java::LocalDate;
use crate::{Faker, Result};

const REGIONS: [&str; 9] = ["41", "42", "43", "44", "45", "46", "47", "48", "49"];

/// Macedonian unique master citizen numbers (EMBG) `DDMMYYYRRBBBK`: the birthday with a three-digit year, a
/// region, 000–499 for men and 500–999 for women, and a mod-11 check digit.
#[derive(Debug, Clone, Copy, Default)]
pub struct MacedonianIdNumber;

impl MacedonianIdNumber {
    fn base_part(faker: &Faker, birthday: LocalDate, gender: Gender) -> String {
        let region = REGIONS[faker.number().number_between(0, REGIONS.len() as i32) as usize];
        let ordinal = match gender {
            Gender::Female => faker.number().number_between(500, 1000),
            Gender::Male => faker.number().number_between(0, 500),
        };
        format!("{:02}{:02}{:03}{region}{ordinal:03}", birthday.day_of_month(), birthday.month_value(), birthday.year() % 1000)
    }

    /// `checksum(text)`: `11 - (7(a+g) + 6(b+h) + 5(c+i) + 4(d+j) + 3(e+k) + 2(f+l)) mod 11`, 0 for 10 and 11.
    pub fn checksum(&self, text: &str) -> i32 {
        let d = |i| digit_at(text, i);
        let m = 11 - (7 * (d(0) + d(6)) + 6 * (d(1) + d(7)) + 5 * (d(2) + d(8)) + 4 * (d(3) + d(9)) + 3 * (d(4) + d(10)) + 2 * (d(5) + d(11))) % 11;
        match m {
            10 | 11 => 0,
            _ => m,
        }
    }
}

impl IdNumberGenerator for MacedonianIdNumber {
    fn country_code(&self) -> &'static str {
        "MK"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let gender = gender(faker, request);
        let base_part = Self::base_part(faker, birth_date, gender);
        let id_number = format!("{base_part}{}", self.checksum(&base_part));
        Ok(PersonIdNumber { id_number, birth_date, gender })
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let birthday = faker.time_and_date().birthday()?;
        let gender = random_gender(faker);
        let base_part = Self::base_part(faker, birthday, gender);
        Ok(format!("{base_part}{}", (self.checksum(&base_part) + 1) % 10))
    }
}
