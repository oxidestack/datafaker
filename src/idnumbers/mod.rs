//! `net.datafaker.idnumbers`: national identity numbers, one generator per country, behind the `IdNumber` provider
//! (`faker.id_number()`). Upstream finds the generators with `ServiceLoader` (`META-INF/services`); here they are the
//! static registry [`generators`], in the same order.
//!
//! Each generator is a unit struct implementing [`IdNumberGenerator`]. The helpers upstream keeps package-private
//! (and its tests call) are public here: `Utils` and `LatinLetters` below, and each generator's checksum functions.
//! Birthdays come from `faker.time_and_date().birthday…`, which read the clock (`LocalDate.now()`) as upstream;
//! [`crate::java::time::with_fixed_clock`] pins it.

use std::fmt;

use crate::java::{DateTimeFormatter, LocalDate};
use crate::{Error, Faker, Result};

mod albanian;
mod american;
mod brazil;
mod bulgarian;
mod chinese;
mod estonian;
mod french;
mod georgian;
mod hungarian;
mod irish;
mod italian;
mod latvian;
mod macedonian;
mod mexican;
mod moldovan;
mod norwegian;
mod polish;
mod portuguese;
pub mod pt_br;
mod romanian;
mod singapore;
mod south_african;
mod south_korean;
mod sweden;
mod ukrainian;

pub use albanian::AlbanianIdNumber;
pub use american::AmericanIdNumber;
pub use brazil::BrazilIdNumber;
pub use bulgarian::BulgarianIdNumber;
pub use chinese::ChineseIdNumber;
pub use estonian::EstonianIdNumber;
pub use french::FrenchIdNumber;
pub use georgian::GeorgianIdNumber;
pub use hungarian::HungarianIdNumber;
pub use irish::IrishIdNumber;
pub use italian::ItalianIdNumber;
pub use latvian::LatvianIdNumber;
pub use macedonian::MacedonianIdNumber;
pub use mexican::MexicanIdNumber;
pub use moldovan::MoldovanIdNumber;
pub use norwegian::NorwegianIdNumber;
pub use polish::{PolishGender, PolishIdNumber};
pub use portuguese::PortugueseIdNumber;
pub use romanian::RomanianIdNumber;
pub use singapore::{SingaporeIdNumber, SingaporeType};
pub use south_african::SouthAfricanIdNumber;
pub use south_korean::SouthKoreanIdNumber;
pub use sweden::SwedenIdNumber;
pub use ukrainian::UkrainianIdNumber;

/// `PersonIdNumber.Gender`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    /// `FEMALE`.
    Female,
    /// `MALE`.
    Male,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Gender::Female => "FEMALE",
            Gender::Male => "MALE",
        })
    }
}

/// `IdNumber.GenderRequest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GenderRequest {
    /// A female's number.
    Female,
    /// A male's number.
    Male,
    /// Either gender, chosen at random.
    Any,
}

/// `IdNumber.IdNumberRequest(minAge, maxAge, gender)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdNumberRequest {
    /// The youngest age the birthday may give.
    pub min_age: i32,
    /// The oldest age the birthday may give.
    pub max_age: i32,
    /// The gender the number encodes.
    pub gender: GenderRequest,
}

impl IdNumberRequest {
    /// A request for a number of a person between `min_age` and `max_age` years old, of `gender`.
    pub fn new(min_age: i32, max_age: i32, gender: GenderRequest) -> IdNumberRequest {
        IdNumberRequest { min_age, max_age, gender }
    }
}

/// `PersonIdNumber(idNumber, birthDate, gender)`: an id number with the birthday and gender it encodes (random ones
/// where the country's number encodes neither).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonIdNumber {
    /// The id number.
    pub id_number: String,
    /// The birthday it encodes.
    pub birth_date: LocalDate,
    /// The gender it encodes.
    pub gender: Gender,
}

impl fmt::Display for PersonIdNumber {
    /// The record's `toString()`: `PersonIdNumber[idNumber=…, birthDate=…, gender=…]`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PersonIdNumber[idNumber={}, birthDate={}, gender={}]", self.id_number, self.birth_date, self.gender)
    }
}

/// `IdNumberGenerator`: the id numbers of one country.
pub trait IdNumberGenerator: Sync {
    /// The ISO-2 code of the country (`US`, `EE`, `MD`, …).
    fn country_code(&self) -> &'static str;

    /// `generateValid(faker)`: a valid id number, by default of an 18 to 65 years old person of any gender.
    fn generate_valid(&self, faker: &Faker) -> Result<String> {
        Ok(self.generate_valid_for(faker, &IdNumberRequest::new(18, 65, GenderRequest::Any))?.id_number)
    }

    /// `generateInvalid(faker)`.
    fn generate_invalid(&self, faker: &Faker) -> Result<String>;

    /// `generateValid(faker, request)`: a valid id number for the ages and gender asked for.
    fn generate_valid_for(&self, faker: &Faker, request: &IdNumberRequest) -> Result<PersonIdNumber>;
}

/// The generators `ServiceLoader` finds upstream (`META-INF/services/net.datafaker.idnumbers.IdNumberGenerator`),
/// in its order.
pub fn generators() -> &'static [&'static dyn IdNumberGenerator] {
    &[
        &AlbanianIdNumber,
        &AmericanIdNumber,
        &BulgarianIdNumber,
        &BrazilIdNumber,
        &ChineseIdNumber,
        &EstonianIdNumber,
        &FrenchIdNumber,
        &GeorgianIdNumber,
        &HungarianIdNumber,
        &ItalianIdNumber,
        &LatvianIdNumber,
        &MacedonianIdNumber,
        &MexicanIdNumber,
        &MoldovanIdNumber,
        &NorwegianIdNumber,
        &PolishIdNumber,
        &PortugueseIdNumber,
        &RomanianIdNumber,
        &SingaporeIdNumber,
        &SouthAfricanIdNumber,
        &SouthKoreanIdNumber,
        &SwedenIdNumber,
        &UkrainianIdNumber,
    ]
}

/// The generator for an ISO-2 country code (`IdNumber`'s `countryProviders` map). [`IrishIdNumber`] is not among
/// them: upstream does not list it in `META-INF/services`, so `en_IE` falls back to the YAML data, as here.
pub fn generator_for(country: &str) -> Option<&'static dyn IdNumberGenerator> {
    generators().iter().copied().find(|g| g.country_code() == country)
}

// ---------------------------------------------------------------------------------------------------------------
// Utils

/// `Utils.birthday(faker, request)`: `faker.timeAndDate().birthday(minAge, maxAge)`.
pub fn birthday(faker: &Faker, request: &IdNumberRequest) -> Result<LocalDate> {
    faker.time_and_date().birthday_between(request.min_age, request.max_age)
}

/// `Utils.gender(faker, request)`: the gender asked for, or a random one for `ANY`.
pub fn gender(faker: &Faker, request: &IdNumberRequest) -> Gender {
    match request.gender {
        GenderRequest::Female => Gender::Female,
        GenderRequest::Male => Gender::Male,
        GenderRequest::Any => random_gender(faker),
    }
}

/// `Utils.randomGender(faker)`: `faker.bool().bool() ? FEMALE : MALE`.
pub fn random_gender(faker: &Faker) -> Gender {
    if faker.bool().bool() { Gender::Female } else { Gender::Male }
}

/// `Utils.digitAt(text, index)`.
pub fn digit_at(text: &str, index: usize) -> i32 {
    digit(text.chars().nth(index).unwrap_or('\0'))
}

/// `Utils.digit(c)`: `c - '0'`.
pub fn digit(c: char) -> i32 {
    c as i32 - '0' as i32
}

/// `Utils.multiply(text, weights)`: the sum of each digit times its weight. (A text longer than the weights is an
/// `ArrayIndexOutOfBoundsException` upstream; its extra characters are ignored here.)
pub fn multiply(text: &str, weights: &[i32]) -> i32 {
    text.chars().zip(weights).map(|(c, w)| digit(c) * w).sum()
}

/// `Character.getNumericValue(c)` for the characters id numbers hold: ASCII digits, ASCII letters (10 to 35), else
/// -1.
pub(crate) fn numeric_value(c: char) -> i32 {
    match c {
        '0'..='9' => c as i32 - '0' as i32,
        'a'..='z' => c as i32 - 'a' as i32 + 10,
        'A'..='Z' => c as i32 - 'A' as i32 + 10,
        _ => -1,
    }
}

/// `DateTimeFormatter.ofPattern(pattern).format(date)`.
pub(crate) fn format_date(date: LocalDate, pattern: &str) -> Result<String> {
    date.format(&DateTimeFormatter::of_pattern(pattern)?)
}

/// `Integer.parseInt(text)` for ASCII text: an optional sign and decimal digits, in `int` range.
pub(crate) fn parse_int(text: &str) -> Result<i32> {
    let bad = || Error::new("NumberFormatException", format!("For input string: \"{text}\""));
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(bad());
    }
    text.parse::<i32>().map_err(|_| bad())
}

/// `text.substring(begin, end)` on the characters of `text`, with `String`'s exception.
pub(crate) fn substring(text: &str, begin: usize, end: usize) -> Result<String> {
    let length = text.chars().count();
    if begin > end || end > length {
        return Err(Error::new("StringIndexOutOfBoundsException", format!("begin {begin}, end {end}, length {length}")));
    }
    Ok(text.chars().skip(begin).take(end - begin).collect())
}

/// `text.charAt(index)`, with `String`'s exception.
pub(crate) fn char_at(text: &str, index: i64) -> Result<char> {
    let length = text.chars().count();
    usize::try_from(index)
        .ok()
        .and_then(|i| text.chars().nth(i))
        .ok_or_else(|| Error::new("StringIndexOutOfBoundsException", format!("Index {index} out of bounds for length {length}")))
}

/// Whether `LocalDate.parse(text, ofPattern(p)).format(ofPattern(p))` differs from `text`, for a pattern `p` of
/// fixed-width numeric fields (`yyMMdd`, `yyyyMMdd`) naming `year`, `month` and `day`: the default `SMART` resolver
/// moves a day past the end of its month back to the month's last day. `None` where parsing throws (a character
/// other than an ASCII digit, a month or day out of range).
pub(crate) fn smart_date_differs(text: &str, year: i32, month: i32, day: i32) -> Option<bool> {
    if !text.bytes().all(|b| b.is_ascii_digit()) || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let length = LocalDate::of(year, month as u32, 1).ok()?.length_of_month() as i32;
    Some(day > length)
}

/// The `DateTimeParseException` of a text `LocalDate.parse` rejects.
pub(crate) fn date_parse_error(text: &str) -> Error {
    Error::new("DateTimeParseException", format!("Text '{text}' could not be parsed at index 0"))
}

// ---------------------------------------------------------------------------------------------------------------
// LatinLetters

/// `LatinLetters.isConsonant(c)`: not one of `AEIOUaeiou` (any other character counts as a consonant).
pub fn is_consonant(c: char) -> bool {
    !matches!(c, 'A' | 'a' | 'E' | 'e' | 'I' | 'i' | 'O' | 'o' | 'U' | 'u')
}

/// `LatinLetters.removeNonLatinLetters(text)`: only the ASCII letters.
pub fn remove_non_latin_letters(text: &str) -> String {
    text.chars().filter(char::is_ascii_alphabetic).collect()
}
