//! `net.datafaker.idnumbers.IrishIdNumber`.

use super::{IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, gender, numeric_value};
use crate::{Error, Faker, Result};

const WEIGHTS: [i32; 7] = [8, 7, 6, 5, 4, 3, 2];

/// Irish Personal Public Service Numbers (PPSN): seven digits, a mod-23 check letter and an optional suffix (`A`,
/// `B`, `H`, `W`) that counts in the check. Upstream does not register it in `META-INF/services`, so
/// `faker.id_number()` never reaches it.
#[derive(Debug, Clone, Copy, Default)]
pub struct IrishIdNumber;

/// A suffix's value in the checksum: `A`, `B`, `H` by their place in the alphabet, `W` 0.
fn suffix_value(c: char) -> Option<i32> {
    match c {
        'A' | 'B' | 'H' => Some(c as i32 - 'A' as i32 + 1),
        'W' => Some(0),
        _ => None,
    }
}

/// `calculateCheckSumCharacter(sum)`: `W` for a remainder of 0, else the remainder's letter.
fn check_sum_character(sum: i32) -> char {
    let remainder = sum % 23;
    if remainder == 0 { 'W' } else { char::from(b'A' + remainder as u8 - 1) }
}

impl IrishIdNumber {
    /// `validateAndCheckModulo23(ppsn)`: seven digits and one or two upper-case letters, the first the check letter
    /// of the digits and the suffix.
    pub fn validate_and_check_modulo23(&self, ppsn: &str) -> bool {
        let chars: Vec<char> = ppsn.chars().collect();
        let shape = (8..=9).contains(&chars.len()) && chars[..7].iter().all(char::is_ascii_digit) && chars[7..].iter().all(char::is_ascii_uppercase);
        if !shape {
            return false;
        }
        let mut sum: i32 = chars[..7].iter().zip(WEIGHTS).map(|(c, w)| numeric_value(*c) * w).sum();
        if chars.len() == 9 {
            match suffix_value(chars[8]) {
                Some(value) => sum += value * 9,
                None => return false,
            }
        }
        chars[7] == check_sum_character(sum)
    }
}

impl IdNumberGenerator for IrishIdNumber {
    fn country_code(&self) -> &'static str {
        "IE"
    }

    /// Seven digits and `Z`, a letter no checksum gives.
    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        Ok(format!("{}Z", faker.number().digits(7)))
    }

    fn generate_valid(&self, faker: &Faker) -> Result<String> {
        let digits_ppsn = faker.number().digits(7);
        let suffix = if faker.bool().bool() { faker.options().option(&["A", "B", "H", "W"])? } else { "" };
        let mut sum: i32 = digits_ppsn.chars().zip(WEIGHTS).map(|(c, w)| numeric_value(c) * w).sum();
        if let Some(extra) = suffix.chars().next() {
            let value = suffix_value(extra).ok_or_else(|| Error::new("IllegalStateException", format!("Unexpected suffix: {suffix}")))?;
            sum += value * 9;
        }
        Ok(format!("{digits_ppsn}{}{suffix}", check_sum_character(sum)))
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let id_number = self.generate_valid(faker)?;
        let birth_date = birthday(faker, request)?;
        Ok(PersonIdNumber { id_number, birth_date, gender: gender(faker, request) })
    }
}
