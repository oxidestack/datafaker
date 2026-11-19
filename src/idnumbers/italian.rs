//! `net.datafaker.idnumbers.ItalianIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, char_at, gender, is_consonant, remove_non_latin_letters, substring};
use crate::providers::text::TextSymbolsBuilder;
use crate::{Error, Faker, Result};

const REGION_CODE_FIRST_LETTERS: &str = "ABCDEFGHIJKLM";
const MONTH_LETTER: &str = "_ABCDEHLMPRST";

/// `ODD_CHARACTERS`: the value of a character at an odd position (1st, 3rd, …).
fn odd_value(c: char) -> Option<i32> {
    const DIGITS: [i32; 10] = [1, 0, 5, 7, 9, 13, 15, 17, 19, 21];
    const LETTERS: [i32; 26] = [1, 0, 5, 7, 9, 13, 15, 17, 19, 21, 2, 4, 18, 20, 11, 3, 6, 8, 12, 14, 16, 10, 22, 25, 24, 23];
    match c {
        '0'..='9' => Some(DIGITS[c as usize - '0' as usize]),
        'A'..='Z' => Some(LETTERS[c as usize - 'A' as usize]),
        _ => None,
    }
}

/// `EVEN_CHARACTERS`: the value of a character at an even position: a digit's value, a letter's place from 0.
fn even_value(c: char) -> Option<i32> {
    match c {
        '0'..='9' => Some(c as i32 - '0' as i32),
        'A'..='Z' => Some(c as i32 - 'A' as i32),
        _ => None,
    }
}

/// Italian fiscal codes (codice fiscale): three letters of the last name and of the first name (upstream takes the
/// first name first), the year, a month letter, the day (plus 40 for women), a region code and a check letter.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItalianIdNumber;

impl ItalianIdNumber {
    /// `encodeName(name)`: the consonants of the (upper-case) name's Latin letters, then its vowels, then `XXX`;
    /// the first three.
    pub fn encode_name(&self, name: &str) -> String {
        let latin_letters = remove_non_latin_letters(name);
        let consonants = latin_letters.chars().filter(|c| is_consonant(*c));
        let vowels = latin_letters.chars().filter(|c| !is_consonant(*c));
        consonants.chain(vowels).chain("XXX".chars()).take(3).collect()
    }

    /// `checksum(basePart)`: the odd and even positions' values summed, mod 26, as a letter. A character outside
    /// `0-9A-Z` is upstream's `NullPointerException` (an unboxed missing map entry).
    pub fn checksum(&self, base_part: &str) -> Result<char> {
        let npe = || Error::new("NullPointerException", "Cannot invoke \"java.lang.Integer.intValue()\" because the return value of \"java.util.Map.get(Object)\" is null");
        let mut sum = 0;
        for c in base_part.chars().step_by(2) {
            sum += odd_value(c).ok_or_else(npe)?;
        }
        for c in base_part.chars().skip(1).step_by(2) {
            sum += even_value(c).ok_or_else(npe)?;
        }
        Ok(char::from(b'A' + (sum % 26) as u8))
    }

    fn encode_region(faker: &Faker) -> Result<String> {
        let region_letter = faker.text().text_from(&TextSymbolsBuilder::builder().len(1).with(REGION_CODE_FIRST_LETTERS).build()?)?;
        Ok(format!("{region_letter}{:03}", faker.number().number_between(1, 1000)))
    }
}

impl IdNumberGenerator for ItalianIdNumber {
    fn country_code(&self) -> &'static str {
        "IT"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let gender = gender(faker, request);
        let first_name = faker.name().first_name()?.to_uppercase();
        let last_name = faker.name().last_name()?.to_uppercase();
        let year = substring(&birth_date.year().to_string(), 2, birth_date.year().to_string().chars().count())?;
        let month = char_at(MONTH_LETTER, birth_date.month_value() as i64)?;
        let day = match gender {
            Gender::Female => 40 + birth_date.day_of_month(),
            Gender::Male => birth_date.day_of_month(),
        };
        let base_part = format!(
            "{}{}{year}{month}{day:02}{}",
            self.encode_name(&first_name),
            self.encode_name(&last_name),
            Self::encode_region(faker)?
        );
        let id_number = format!("{base_part}{}", self.checksum(&base_part)?);
        Ok(PersonIdNumber { id_number, birth_date, gender })
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let valid = self.generate_valid(faker)?;
        let mut chars: Vec<char> = valid.chars().collect();
        chars.pop();
        chars.push('9');
        Ok(chars.into_iter().collect())
    }
}
