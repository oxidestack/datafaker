//! `net.datafaker.idnumbers.SouthAfricanIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, format_date, gender, parse_int, smart_date_differs};
use crate::{Faker, Result};

const VALID_PATTERN: [&str; 2] = ["##########08#", "##########18#"];
const CODE_PATTERN: [&str; 2] = ["18", "08"];

/// South African identity numbers: `yyMMdd`, a sequence (0000–4999 women, 5000–9999 men), `08` or `18` and a Luhn
/// check digit.
#[derive(Debug, Clone, Copy, Default)]
pub struct SouthAfricanIdNumber;

impl SouthAfricanIdNumber {
    /// `getValidSsn(faker)` (deprecated upstream): `generateValid(faker)`.
    pub fn get_valid_ssn(&self, faker: &Faker) -> Result<String> {
        self.generate_valid(faker)
    }

    /// `getInValidSsn(faker)` (deprecated upstream): `generateInvalid(faker)`.
    pub fn get_in_valid_ssn(&self, faker: &Faker) -> Result<String> {
        self.generate_invalid(faker)
    }

    /// `sequentialNumber(faker, gender)`: four digits, below 5000 for women.
    pub fn sequential_number(faker: &Faker, gender: Gender) -> String {
        let number = match gender {
            Gender::Female => faker.number().number_between(0, 5000),
            Gender::Male => faker.number().number_between(5000, 10_000),
        };
        format!("{number:04}")
    }

    /// `isValidEnZASsn(ssn)`: 13 characters, not a date `yyMMdd` the `SMART` resolver would change, and the right
    /// check digit. As upstream, a month or day out of range passes the date check (only the check digit decides).
    pub fn is_valid_en_za_ssn(ssn: &str) -> bool {
        let chars: Vec<char> = ssn.chars().collect();
        if chars.len() != 13 {
            return false;
        }
        let part = |from: usize, to: usize| chars[from..to].iter().collect::<String>();
        match Self::parse_date(&part(0, 2), &part(2, 4), &part(4, 6)) {
            Some(true) | None => return false,
            Some(false) => {}
        }
        chars[12] as i32 - '0' as i32 == Self::calculate_checksum(&chars, 12)
    }

    /// `parseDate(ssn)`: whether the date differs once parsed and formatted again; `None` where upstream's parsing
    /// throws (`NumberFormatException`, `DateTimeParseException`), which counts as invalid.
    fn parse_date(yy: &str, mm: &str, dd: &str) -> Option<bool> {
        let year = parse_int(yy).ok()?;
        let month = parse_int(mm).ok()?;
        let day = parse_int(dd).ok()?;
        if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return Some(false);
        }
        smart_date_differs(&format!("{yy}{mm}{dd}"), 2000 + year, month, day)
    }

    /// `calculateChecksum(number, length2Check)`: Luhn over the first `length` digits, from the right.
    fn calculate_checksum(number: &[char], length: usize) -> i32 {
        let digit = |i: usize| number[i] as i32 - '0' as i32;
        let mut total = 0;
        let mut i = length as i32 - 1;
        while i >= 0 {
            let doubled = Self::digit_sum(digit(i as usize) * 2);
            total += if i == 0 { doubled } else { doubled + digit(i as usize - 1) };
            i -= 2;
        }
        if (0..9).contains(&total) {
            10 - total
        } else {
            let res = total % 10;
            if res == 0 { res } else { 10 - res }
        }
    }

    /// `calculate(number)`: the sum of the decimal digits (0 for a number below 1).
    fn digit_sum(mut number: i32) -> i32 {
        let mut res = 0;
        while number > 0 {
            res += number % 10;
            number /= 10;
        }
        res
    }
}

impl IdNumberGenerator for SouthAfricanIdNumber {
    fn country_code(&self) -> &'static str {
        "ZA"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let gender = gender(faker, request);
        let date = format_date(birth_date, "yyMMdd")?;
        let sequence = Self::sequential_number(faker, gender);
        let base_part = format!("{date}{sequence}{}", faker.options().option(&CODE_PATTERN)?);
        let chars: Vec<char> = base_part.chars().collect();
        let id_number = format!("{base_part}{}", Self::calculate_checksum(&chars, 12));
        Ok(PersonIdNumber { id_number, birth_date, gender })
    }

    /// A random `##########08#` or `##########18#`, drawn again while it is valid.
    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let mut ssn = faker.numerify(faker.options().option(&VALID_PATTERN)?);
        while Self::is_valid_en_za_ssn(&ssn) {
            ssn = faker.numerify(faker.options().option(&VALID_PATTERN)?);
        }
        Ok(ssn)
    }
}
