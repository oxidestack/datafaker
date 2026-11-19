//! `net.datafaker.idnumbers.FrenchIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, format_date, gender, substring};
use crate::java::LocalDate;
use crate::{Error, Faker, Result};

/// French INSEE codes: `syymmlloookkk` (gender, year and month of birth, department and commune, birth order) and a
/// two-digit control key, 97 minus the number mod 97 (Corsica's `2A`/`2B` read as 19/18).
#[derive(Debug, Clone, Copy, Default)]
pub struct FrenchIdNumber;

impl FrenchIdNumber {
    fn base_part(faker: &Faker, birthday: LocalDate, gender: Gender) -> Result<String> {
        let gender_number = if gender == Gender::Male { "1" } else { "2" };
        let birth_date = format_date(birthday, "yyMM")?;
        let birth_location = faker.regexify("\\d{5}|2[AB]\\d{3}|97[1-6]\\d{2}")?;
        let birth_number = faker.regexify("\\d{3}")?;
        Ok(format!("{gender_number}{birth_date}{birth_location}{birth_number}"))
    }

    /// `controlKey(basePart)`.
    pub fn control_key(&self, base_part: &str) -> Result<String> {
        let digits = base_part.replace("2A", "19").replace("2B", "18");
        let base_part_long = parse_long(&digits)?;
        Ok(format!("{:02}", 97 - base_part_long % 97))
    }
}

/// `Long.parseLong(text)` for ASCII text.
fn parse_long(text: &str) -> Result<i64> {
    let unsigned = text.strip_prefix(['+', '-']).unwrap_or(text);
    if unsigned.is_empty() || !unsigned.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Error::new("NumberFormatException", format!("For input string: \"{text}\"")));
    }
    text.parse::<i64>().map_err(|_| Error::new("NumberFormatException", format!("For input string: \"{text}\"")))
}

impl IdNumberGenerator for FrenchIdNumber {
    fn country_code(&self) -> &'static str {
        "FR"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let gender = gender(faker, request);
        let base_part = Self::base_part(faker, birth_date, gender)?;
        let id_number = format!("{base_part}{}", self.control_key(&base_part)?);
        Ok(PersonIdNumber { id_number, birth_date, gender })
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        Ok(format!("{}98", substring(&self.generate_valid(faker)?, 0, 13)?))
    }
}
