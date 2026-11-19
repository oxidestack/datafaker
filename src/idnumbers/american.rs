//! `net.datafaker.idnumbers.AmericanIdNumber`.

use super::{IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, gender};
use crate::{Faker, Result};

/// The patterns of invalid social security numbers, drawn from by [`AmericanIdNumber`]'s `generateInvalid`.
pub const INVALID_SSNS: [&str; 5] = ["0{3}-\\d{2}-\\d{4}", "\\d{3}-0{2}-\\d{4}", "\\d{3}-\\d{2}-0{4}", "666-\\d{2}-\\d{4}", "9\\d{2}-\\d{2}-\\d{4}"];

/// United States social security numbers (`###-##-####`): area 000, 666 and 9xx, group 00 and serial 0000 are
/// invalid.
#[derive(Debug, Clone, Copy, Default)]
pub struct AmericanIdNumber;

impl AmericanIdNumber {
    /// `getValidSsn(faker)` (deprecated upstream): `generateValid(faker)`.
    pub fn get_valid_ssn(&self, faker: &Faker) -> Result<String> {
        self.generate_valid(faker)
    }
}

/// Whether `ssn` matches one of [`INVALID_SSNS`] (as `Pattern.matcher(ssn).matches()`).
fn is_invalid(ssn: &str) -> bool {
    let b = ssn.as_bytes();
    let digits = |r: std::ops::Range<usize>| b[r].iter().all(u8::is_ascii_digit);
    let shape = b.len() == 11 && b[3] == b'-' && b[6] == b'-' && digits(0..3) && digits(4..6) && digits(7..11);
    shape && (&ssn[..3] == "000" || &ssn[4..6] == "00" || &ssn[7..] == "0000" || &ssn[..3] == "666" || b[0] == b'9')
}

impl IdNumberGenerator for AmericanIdNumber {
    fn country_code(&self) -> &'static str {
        "US"
    }

    /// A random `[0-8]\d{2}-\d{2}-\d{4}`, drawn again while it matches an invalid pattern.
    fn generate_valid(&self, faker: &Faker) -> Result<String> {
        loop {
            let ssn = faker.regexify("[0-8]\\d{2}-\\d{2}-\\d{4}")?;
            if !is_invalid(&ssn) {
                return Ok(ssn);
            }
        }
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let id_number = self.generate_valid(faker)?;
        let birth_date = birthday(faker, request)?;
        Ok(PersonIdNumber { id_number, birth_date, gender: gender(faker, request) })
    }

    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        faker.regexify(faker.options().next_element(&INVALID_SSNS)?)
    }
}
