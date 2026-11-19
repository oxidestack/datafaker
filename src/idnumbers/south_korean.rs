//! `net.datafaker.idnumbers.SouthKoreanIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, gender};
use crate::{Faker, Result};

/// South Korean resident registration numbers `yyMMdd-GNNNNNN`: the birthday, a digit for century, gender and
/// nationality (from a random ISO country), and six random digits.
#[derive(Debug, Clone, Copy, Default)]
pub struct SouthKoreanIdNumber;

impl SouthKoreanIdNumber {
    /// `getValidRrn(faker)` (deprecated upstream): `generateValid(faker)`.
    pub fn get_valid_rrn(&self, faker: &Faker) -> Result<String> {
        self.generate_valid(faker)
    }

    fn seventh_digit(year: i32, gender: Gender, iso_country: &str) -> i32 {
        let locality = if iso_country.eq_ignore_ascii_case("kr") { 1 } else { 5 };
        let male = gender == Gender::Male;
        if year < 1900 {
            if male { 9 } else { 0 }
        } else if year < 2000 {
            locality + if male { 0 } else { 1 }
        } else {
            locality + if male { 2 } else { 3 }
        }
    }
}

impl IdNumberGenerator for SouthKoreanIdNumber {
    fn country_code(&self) -> &'static str {
        "KR"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let iso = faker.nation().iso_country()?;
        let gender = gender(faker, request);
        let (year, month, day) = (birth_date.year() % 100, birth_date.month_value() as i32, birth_date.day_of_month() as i32);
        let digit = |v: i32| char::from_u32(('0' as i32 + v) as u32).unwrap_or('\u{FFFD}');
        let day_part: String = [year / 10, year % 10, month / 10, month % 10, day / 10, day % 10].into_iter().map(digit).collect();
        let pattern = format!("{day_part}-{}######", Self::seventh_digit(birth_date.year(), gender, &iso));
        Ok(PersonIdNumber { id_number: faker.numerify(&pattern), birth_date, gender })
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        Ok(format!("{}42", self.generate_valid(faker)?))
    }
}
