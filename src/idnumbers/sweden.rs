//! `net.datafaker.idnumbers.SwedenIdNumber`.

use super::{IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, format_date, gender, parse_int, smart_date_differs, substring};
use crate::java::LocalDate;
use crate::{Error, Faker, Result};

const VALID_PATTERNS: [&str; 2] = ["######-####", "######+####"];

/// Swedish personal identity numbers (personnummer) `yyMMdd-NNNC`: `+` instead of `-` for people more than 100
/// years old, a birth number 001–999 and a Luhn check digit.
#[derive(Debug, Clone, Copy, Default)]
pub struct SwedenIdNumber;

impl SwedenIdNumber {
    /// `getValidSsn(faker)` (deprecated upstream): `generateValid(faker)`.
    pub fn get_valid_ssn(&self, faker: &Faker) -> Result<String> {
        self.generate_valid(faker)
    }

    /// `getInvalidSsn(faker)` (deprecated upstream): `generateInvalid(faker)`.
    pub fn get_invalid_ssn(&self, faker: &Faker) -> Result<String> {
        self.generate_invalid(faker)
    }

    /// `isValidSwedishSsn(ssn)`: 11 characters, a date the `SMART` resolver keeps (its century found from today
    /// and the separator), a birth number other than 000 and the right check digit. As upstream, a check character
    /// other than a digit throws `NumberFormatException`, and a year with no century before today throws.
    pub fn is_valid_swedish_ssn(ssn: &str) -> Result<bool> {
        if ssn.chars().count() != 11 {
            return Ok(false);
        }
        match Self::is_date_incorrect(ssn) {
            Ok(true) => return Ok(false),
            Ok(false) => {}
            Err(e) if matches!(e.simple_class(), "DateTimeParseException" | "NumberFormatException") => return Ok(false),
            Err(e) => return Err(e),
        }
        if ssn.chars().skip(7).take(3).collect::<String>() == "000" {
            return Ok(false);
        }
        let calculated_checksum = Self::calculate_checksum(ssn);
        let checksum = parse_int(&substring(ssn, 10, 11)?)?;
        Ok(checksum == calculated_checksum)
    }

    fn is_date_incorrect(ssn: &str) -> Result<bool> {
        let date_string = substring(ssn, 0, 6)?;
        parse_int(&substring(&date_string, 0, 2)?)?;
        let month = parse_int(&substring(&date_string, 2, 4)?)?;
        if !(1..=12).contains(&month) {
            return Ok(true);
        }
        let day = parse_int(&substring(&date_string, 4, 6)?)?;
        if !(1..=31).contains(&day) {
            return Ok(true);
        }
        let full = format!("{}{date_string}", Self::find_year_beginning_from_ssn(ssn)?);
        let year = parse_int(&full[..4]).map_err(|_| super::date_parse_error(&full))?;
        smart_date_differs(&full, year, month, day).ok_or_else(|| super::date_parse_error(&full))
    }

    /// `findYearBeginningFromSsn(ssn)`: the century of the latest year up to today (with `+`, before 100 years
    /// ago) that ends in the number's two year digits.
    pub fn find_year_beginning_from_ssn(ssn: &str) -> Result<String> {
        let symbol = ssn.chars().nth(6).unwrap_or('\0');
        let year_end = substring(ssn, 0, 2)?;
        let now = LocalDate::now();
        let start_year = if symbol == '+' { now.minus_years(100)?.year() - 1 } else { now.year() };
        for year in (0..=start_year).rev() {
            let text = year.to_string();
            if text.ends_with(&year_end) {
                return Ok(text[..2].to_string());
            }
        }
        Err(Error::runtime(if symbol == '+' {
            format!("Cannot find year that ends with {year_end} and is before {}", start_year + 1)
        } else {
            format!("Cannot find year that ends with {year_end}")
        }))
    }

    /// `isYearOver100YearsAgo(year, currentDate)`: whether January 1st of `year` is before `current_date` minus 100
    /// years.
    pub fn is_year_over_100_years_ago(year: &str, current_date: LocalDate) -> Result<bool> {
        let hundred_years_ago = current_date.minus_years(100)?;
        Ok(LocalDate::of(parse_int(year)?, 1, 1)?.is_before(&hundred_years_ago))
    }

    /// `calculateChecksum(number)`: Luhn over `yyMMdd` and the birth number.
    fn calculate_checksum(number: &str) -> i32 {
        let chars: Vec<char> = number.chars().collect();
        let digits: Vec<char> = chars[..6].iter().chain(&chars[7..10]).copied().collect();
        let calculated: String = digits
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let n = *c as i32 - '0' as i32;
                if i % 2 == 0 { (n << 1).to_string() } else { n.to_string() }
            })
            .collect();
        let sum: i32 = calculated.chars().map(|c| c as i32 - '0' as i32).sum();
        (10 - sum % 10) % 10
    }
}

impl IdNumberGenerator for SwedenIdNumber {
    fn country_code(&self) -> &'static str {
        "SE"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let end = format!("{:03}", faker.number().number_between(1, 1000));
        let year = substring(&birth_date.to_string(), 0, 4)?;
        let symbol = if Self::is_year_over_100_years_ago(&year, LocalDate::now())? { "+" } else { "-" };
        let base_part = format!("{}{symbol}{end}", format_date(birth_date, "yyMMdd")?);
        let id_number = format!("{base_part}{}", Self::calculate_checksum(&base_part));
        Ok(PersonIdNumber { id_number, birth_date, gender: gender(faker, request) })
    }

    /// `121212-1212` replaced by random `######-####` or `######+####` until one is invalid.
    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let mut candidate = "121212-1212".to_string();
        while Self::is_valid_swedish_ssn(&candidate)? {
            candidate = faker.numerify(faker.options().option(&VALID_PATTERNS)?);
        }
        Ok(candidate)
    }
}
