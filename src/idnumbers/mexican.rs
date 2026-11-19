//! `net.datafaker.idnumbers.MexicanIdNumber`.

use super::{Gender, IdNumberGenerator, IdNumberRequest, PersonIdNumber, birthday, gender};
use crate::{Error, Faker, Result};

/// The invalid CURPs `generateInvalid` draws from.
const CHA: [&str; 12] = [
    "HEFA560427MVZRRL04",
    "DKEM193827HDQWEF05",
    "KAKS142444HNSSFAW6",
    "KSDF414424HNSDFAW6",
    "AKDF414424MSDSFAW6",
    "ADKF144424MNSDFCD6",
    "MYDF144424MDNFAW37",
    "AKKS414424MDAFDFW6",
    "WKDF144244HSDCNFA2",
    "AKSK414244HSDATT56",
    "QWDF414424HNSDVAW4",
    "AKDF144424MDEFVFA1",
];

const CONSONANT: [char; 24] = ['B', 'C', 'C', 'D', 'F', 'G', 'H', 'J', 'K', 'L', 'L', 'M', 'N', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X', 'Y', 'Z'];
const VOWEL: [char; 5] = ['A', 'E', 'I', 'O', 'U'];
const STATES: [&str; 33] = [
    "AG", "BC", "BS", "CM", "CS", "CH", "CO", "CL", "DF", "DG", "GT", "GR", "HG", "JA", "EM", "MI", "MO", "NA", "NL", "OA", "PU", "QT", "QR", "SL",
    "SI", "SO", "TB", "TM", "TL", "VE", "YU", "ZA", "NE",
];

/// Mexican CURPs: four letters, six characters of the birthday, `H`/`M`, a state, three vowels, a homonym character
/// and a check digit. As upstream, the six birthday characters are the first six of `yyyyMMdd` (year and month),
/// and a `9` counts as 2 in the checksum.
#[derive(Debug, Clone, Copy, Default)]
pub struct MexicanIdNumber;

impl MexicanIdNumber {
    /// `get(faker)` (deprecated upstream): `generateValid(faker)`.
    pub fn get(&self, faker: &Faker) -> Result<String> {
        self.generate_valid(faker)
    }

    /// `getWrong(faker)` (deprecated upstream): `generateInvalid(faker)`.
    pub fn get_wrong(&self, faker: &Faker) -> Result<String> {
        self.generate_invalid(faker)
    }
}

/// `getChecksum(str)`: over all but the last character, digits below `9` by value and other characters as
/// `c - 'A' + 10`, weighted 18 down to 2; `|sum mod 10 - 10|`, 0 for 10.
fn checksum(chars: &[char]) -> i32 {
    let mut sum = 0;
    let mut v = chars.len() as i32;
    for c in &chars[..chars.len() - 1] {
        let number = if *c < '9' { *c as i32 - '0' as i32 } else { *c as i32 - 'A' as i32 + 10 };
        sum += number * v;
        v -= 1;
    }
    let sum = (sum % 10 - 10).abs();
    if sum == 10 { 0 } else { sum }
}

impl IdNumberGenerator for MexicanIdNumber {
    fn country_code(&self) -> &'static str {
        "MX"
    }

    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        let gender = gender(faker, request);
        let birth_date = birthday(faker, request)?;
        let birth_day: Vec<char> =
            (birth_date.year() * 10000 + birth_date.month_value() as i32 * 100 + birth_date.day_of_month() as i32).to_string().chars().collect();
        let options = faker.options();
        let mut ssn = ['\0'; 18];
        ssn[0] = options.option(&CONSONANT)?;
        ssn[1] = options.option(&VOWEL)?;
        ssn[2] = options.option(&CONSONANT)?;
        ssn[3] = options.option(&CONSONANT)?;
        if birth_day.len() < 6 {
            let message = format!("arraycopy: last source index 6 out of bounds for char[{}]", birth_day.len());
            return Err(Error::new("ArrayIndexOutOfBoundsException", message));
        }
        ssn[4..10].copy_from_slice(&birth_day[..6]);
        ssn[10] = match gender {
            Gender::Female => 'M',
            Gender::Male => 'H',
        };
        let state: Vec<char> = options.option(&STATES)?.chars().collect();
        ssn[11..13].copy_from_slice(&state);
        ssn[13] = options.option(&VOWEL)?;
        ssn[14] = options.option(&VOWEL)?;
        ssn[15] = options.option(&VOWEL)?;
        ssn[16] = if birth_day[0] == '1' { '0' } else { options.option(&CONSONANT)? };
        ssn[17] = checksum(&ssn).to_string().chars().next().unwrap_or('0');
        Ok(PersonIdNumber { id_number: ssn.iter().collect(), birth_date, gender })
    }

    /// One of twelve fixed invalid CURPs.
    fn generate_invalid(&self, faker: &Faker) -> Result<String> {
        Ok(faker.options().option(&CHA)?.to_string())
    }
}
