//! `net.datafaker.idnumbers.PolishIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, gender, random_gender};
use crate::java::LocalDate;
use crate::{Error, Faker, Result};

/// `PolishIdNumber.Gender` (deprecated upstream in favour of `GenderRequest`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolishGender {
    /// `MALE`.
    Male,
    /// `FEMALE`.
    Female,
    /// `ANY`.
    Any,
}

/// Polish PESEL numbers: `yyMMdd` with the century in the month (+80 for the 1800s, +20, +40, +60 from 2000 on),
/// three random digits, a digit even for women and odd for men, and a check digit.
#[derive(Debug, Clone, Copy, Default)]
pub struct PolishIdNumber;

impl PolishIdNumber {
    /// `get(faker, birthDate, gender)` (deprecated upstream): a PESEL for `birth_date`; `None` or `Any` is a random
    /// gender.
    pub fn get(&self, faker: &Faker, birth_date: LocalDate, requested_gender: Option<PolishGender>) -> Result<String> {
        let gender = match requested_gender {
            None | Some(PolishGender::Any) => random_gender(faker),
            Some(PolishGender::Male) => Gender::Male,
            Some(PolishGender::Female) => Gender::Female,
        };
        Self::pesel(faker, birth_date, gender)
    }

    fn pesel(faker: &Faker, birth_date: LocalDate, gender: Gender) -> Result<String> {
        let digits = Self::generate_digits(faker, birth_date, gender)?;
        Ok(Self::to_string(&digits, Self::control_digit(&digits)))
    }

    fn generate_digits(faker: &Faker, birth_date: LocalDate, gender: Gender) -> Result<[i32; 10]> {
        let (year, day) = (birth_date.year(), birth_date.day_of_month() as i32);
        let month_encoded = Self::month_encoded(year, birth_date.month_value() as i32)?;
        let number = faker.number();
        let (r1, r2, r3) = (number.random_digit(), number.random_digit(), number.random_digit());
        let gender_digit = match gender {
            Gender::Female => faker.random().next_int_bounded(5)? * 2,
            Gender::Male => faker.random().next_int_bounded(5)? * 2 + 1,
        };
        Ok([year / 10 % 10, year % 10, month_encoded / 10, month_encoded % 10, day / 10, day % 10, r1, r2, r3, gender_digit])
    }

    fn to_string(digits: &[i32], control_digit: i32) -> String {
        digits.iter().chain([&control_digit]).map(i32::to_string).collect()
    }

    fn control_digit(d: &[i32; 10]) -> i32 {
        let sum = d[0] + d[4] + d[8] + (d[1] + d[5] + d[9]) * 3 + (d[2] + d[6]) * 7 + (d[3] + d[7]) * 9;
        (10 - sum % 10) % 10
    }

    fn month_encoded(year: i32, month: i32) -> Result<i32> {
        let modifier = match year {
            ..1800 => return Err(Error::illegal_argument(format!("Year is before 1800: {year}"))),
            1800..1900 => 80,
            1900..2000 => 0,
            2000..2100 => 20,
            2100..2200 => 40,
            2200..2300 => 60,
            _ => return Err(Error::illegal_argument(format!("Year is after 2300: {year}"))),
        };
        Ok(month + modifier)
    }
}

impl IdNumberGenerator for PolishIdNumber {
    fn country_code(&self) -> &'static str {
        "PL"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let birth_date = birthday(faker, request)?;
        let gender = gender(faker, request);
        Ok(PersonIdNumber { id_number: Self::pesel(faker, birth_date, gender)?, birth_date, gender })
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        let gender = random_gender(faker);
        let digits = Self::generate_digits(faker, faker.time_and_date().birthday()?, gender)?;
        let invalid_control_digit = (Self::control_digit(&digits) + 1) % 10;
        Ok(Self::to_string(&digits, invalid_control_digit))
    }
}
