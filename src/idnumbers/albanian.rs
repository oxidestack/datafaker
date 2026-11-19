//! `net.datafaker.idnumbers.AlbanianIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, char_at, digit, gender};
use crate::{Faker, Result};

const FIRST_CHAR: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const CHECKSUM_CHAR: &str = "WABCDEFGHIJKLMNOPQRSTUV";

/// Albanian identity numbers: ten characters `YYMMDDSSSC`, the decade of the year as a letter from 1900 on, the
/// month plus 50 for women and a mod-23 check letter.
#[derive(Debug, Clone, Copy, Default)]
pub struct AlbanianIdNumber;

impl AlbanianIdNumber {
    /// `yy(year)`: the decade since 1800 as `0`–`9`, `A`–`Z`, then the year's last digit.
    pub fn yy(&self, year: i32) -> Result<String> {
        Ok(format!("{}{}", char_at(FIRST_CHAR, ((year - 1800) / 10) as i64)?, year % 10))
    }

    /// `mm(month, gender)`.
    pub fn mm(&self, month: u32, gender: Gender) -> String {
        format!("{:02}", if gender == Gender::Female { 50 } else { 0 } + month as i32)
    }

    /// `dd(dayOfMonth)`.
    pub fn dd(&self, day_of_month: u32) -> String {
        format!("{day_of_month:02}")
    }

    /// `checksum(text)`: the first character's value plus each later digit times its index, mod 23, as a letter
    /// (a negative sum is upstream's `StringIndexOutOfBoundsException`).
    pub fn checksum(&self, text: &str) -> Result<char> {
        let mut checksum = self.checksum_of_first_char(char_at(text, 0)?);
        for (i, c) in text.chars().enumerate().skip(1) {
            checksum += digit(c) * i as i32;
        }
        char_at(CHECKSUM_CHAR, (checksum % 23) as i64)
    }

    /// `checksumOfFirstChar(c)`: a letter's index in `WABCDEFGHIJKLMNOPQRSTUV` (-1 past `V`), a digit's value.
    pub fn checksum_of_first_char(&self, c: char) -> i32 {
        if c.is_alphabetic() { CHECKSUM_CHAR.find(c).map_or(-1, |i| i as i32) } else { digit(c) }
    }
}

impl IdNumberGenerator for AlbanianIdNumber {
    fn country_code(&self) -> &'static str {
        "AL"
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let pin = self.generate_valid(faker)?;
        let invalid_month = faker.number().number_between(93, 99);
        Ok(format!("{}{invalid_month}{}", &pin[..2], &pin[4..]))
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let gender = gender(faker, request);
        let base_part = format!(
            "{}{}{}{}",
            self.yy(birth_date.year())?,
            self.mm(birth_date.month_value(), gender),
            self.dd(birth_date.day_of_month()),
            faker.number().digits(3)
        );
        let id_number = format!("{base_part}{}", self.checksum(&base_part)?);
        Ok(PersonIdNumber { id_number, birth_date, gender })
    }
}
