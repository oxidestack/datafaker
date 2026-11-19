//! `net.datafaker.idnumbers.SingaporeIdNumber`.

use super::{Gender, GenderRequest, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, gender, random_gender};
use crate::java::LocalDate;
use crate::{Faker, Result};

const CODE: [i32; 7] = [2, 7, 6, 5, 4, 3, 2];
const FIN_LETTERS: &str = "XWUTRQPNMLK";
const UIN_LETTERS: &str = "JZIHGFEDCBA";

/// `SingaporeIdNumber.Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SingaporeType {
    /// `SINGAPOREAN_TWENTIETH_CENTURY`: a citizen's number issued in the 1900s (prefix S).
    SingaporeanTwentiethCentury,
    /// `FOREIGNER_TWENTIETH_CENTURY`: a foreigner's number issued in the 1900s (prefix F).
    ForeignerTwentiethCentury,
    /// `SINGAPOREAN_TWENTY_FIRST_CENTURY`: a citizen's number issued in the 2000s (prefix T).
    SingaporeanTwentyFirstCentury,
    /// `FOREIGNER_TWENTY_FIRST_CENTURY`: a foreigner's number issued in the 2000s (prefix G).
    ForeignerTwentyFirstCentury,
}

/// Singapore NRIC/FIN numbers: a century letter (`S`/`T` for citizens, `F`/`G` for foreigners), seven digits and a
/// check letter. By default for people 0 to 100 years old.
#[derive(Debug, Clone, Copy, Default)]
pub struct SingaporeIdNumber;

impl SingaporeIdNumber {
    fn format(issue_date: LocalDate, citizen: bool, random_digits: &[i32]) -> String {
        let first_letter = if citizen { Self::century_prefix_citizen(issue_date) } else { Self::century_prefix_foreigner(issue_date) };
        let match_letters = if citizen { UIN_LETTERS } else { FIN_LETTERS };
        let mut check_digit = if issue_date.year() < 2000 { 0 } else { 4 };
        let mut id = String::from(first_letter);
        for (digit, code) in random_digits.iter().zip(CODE) {
            check_digit += digit * code;
            id.push_str(&digit.to_string());
        }
        id.push(match_letters.as_bytes()[(check_digit % 11) as usize] as char);
        id
    }

    fn generate_valid_id_number(faker: &Faker, birth_date: LocalDate, citizen: bool, gender: Gender) -> PersonIdNumber {
        let number = faker.number().random_digits(7);
        PersonIdNumber { id_number: Self::format(birth_date, citizen, &number), birth_date, gender }
    }

    /// `getValidFIN(faker, type)`: a number of the given kind, for a birthday in its century.
    pub fn get_valid_fin(faker: &Faker, kind: SingaporeType) -> Result<String> {
        let birth_date = Self::random_birth_date(faker, kind)?;
        let citizen = matches!(kind, SingaporeType::SingaporeanTwentiethCentury | SingaporeType::SingaporeanTwentyFirstCentury);
        Ok(Self::generate_valid_id_number(faker, birth_date, citizen, random_gender(faker)).id_number)
    }

    /// `randomBirthDate(faker, type)`: a birthday in the 1900s, or in the 2000s up to today.
    pub fn random_birth_date(faker: &Faker, kind: SingaporeType) -> Result<LocalDate> {
        let now = LocalDate::now().year();
        match kind {
            SingaporeType::SingaporeanTwentiethCentury | SingaporeType::ForeignerTwentiethCentury => {
                faker.time_and_date().birthday_between(now - 1999, now - 1900)
            }
            SingaporeType::SingaporeanTwentyFirstCentury | SingaporeType::ForeignerTwentyFirstCentury => {
                faker.time_and_date().birthday_between((now - 2099).max(0), now - 2000)
            }
        }
    }

    /// `centuryPrefixCitizen(issueDate)`: `S` for the 1900s, `T` for the 2000s, …
    pub fn century_prefix_citizen(issue_date: LocalDate) -> char {
        char::from_u32(('A' as i32 + issue_date.year() / 100 - 1) as u32).unwrap_or('\u{FFFD}')
    }

    /// `centuryPrefixForeigner(issueDate)`: `F` for the 1900s, `G` for the 2000s, …
    pub fn century_prefix_foreigner(issue_date: LocalDate) -> char {
        char::from_u32(('A' as i32 + issue_date.year() / 100 - 14) as u32).unwrap_or('\u{FFFD}')
    }
}

impl IdNumberGenerator for SingaporeIdNumber {
    fn country_code(&self) -> &'static str {
        "SG"
    }

    fn generate_valid(&self, faker: &Faker) -> Result<String> {
        Ok(self.generate_valid_for(faker, &IdNumberRequest::new(0, 100, GenderRequest::Any))?.id_number)
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let citizen = faker.bool().bool();
        let gender = gender(faker, request);
        Ok(Self::generate_valid_id_number(faker, birth_date, citizen, gender))
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        Ok(format!("{}42", self.generate_valid(faker)?))
    }
}
