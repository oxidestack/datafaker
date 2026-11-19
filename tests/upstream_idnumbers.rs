//! Tests of Datafaker 2.7.0's national identity numbers (tranche T2): the test classes of `idnumbers` and the cases
//! of the `IdNumber` facade's test class that `scripts/gen-providers.py` leaves to hand (it generates the others).
//! Unseeded fakers as upstream; repeated tests run their body as often; parameterized tests loop over their
//! arguments.
//!
//! Adapted: upstream parses dates back with `LocalDate.parse(text, ofPattern("ddMMyy"))`, which datafaker does not
//! port; `parses_ddmmyy`/`yymmdd_year` check the same fields. `FrenchIdNumberTest.controlKey` draws its numbers from
//! an unseeded `RandomService` in place of `new Random()`.

mod common;

use std::collections::HashSet;

use common::{assert_matches, matches};
use datafaker::idnumbers::{
    self, AlbanianIdNumber, BulgarianIdNumber, ChineseIdNumber, EstonianIdNumber, FrenchIdNumber, Gender, GenderRequest, HungarianIdNumber,
    IdNumberGenerator, IdNumberRequest, IrishIdNumber, ItalianIdNumber, LatvianIdNumber, MacedonianIdNumber, MoldovanIdNumber,
    NorwegianIdNumber, PolishGender, PolishIdNumber, RomanianIdNumber, SingaporeIdNumber, SingaporeType, SouthAfricanIdNumber,
    SwedenIdNumber, UkrainianIdNumber, pt_br,
};
use datafaker::java::LocalDate;
use datafaker::{Faker, Locale, RandomService};

/// `helpers/IdNumberPatterns`.
mod patterns {
    pub const SWEDISH: &str = r"\d{6}[-+]\d{4}";
    pub const UKRAINIAN: &str = r"\d{8}-\d{5}";
    pub const SOUTH_AFRICAN: &str = r"[0-9]{10}([01])8[0-9]";
    pub const FRENCH: &str = r"[12]\d{2}(1[0-2]|0[1-9])\d[0-9a-zA-Z]\d{8}";
    pub const ITALIAN: &str = r"[A-Z]{6}\d{2}[ABCDEHLMPRST]\d{2}[\dA-Z]{5}";
    pub const POLISH: &str = r"\d{11}";
    pub const IRISH: &str = r"\d{7}[A-Z]{1,2}";
    pub const HUNGARIAN: &str = r"[1-4]\d{9}";
    pub const ESTONIAN: &str = r"[1-6][0-9]{10}";
    pub const BRAZILIAN: &str = r"\d{3}\.\d{3}\.\d{3}-\d{2}";
    pub const NORWEGIAN: &str = r"\d{11}";
}

fn faker_in(language: &str, country: &str) -> Faker {
    Faker::with_locale(Locale::new(language, country))
}

fn date(text: &str) -> LocalDate {
    LocalDate::parse(text).unwrap()
}

fn parse_int(s: &str) -> i32 {
    s.parse().unwrap()
}

/// `LocalDate.parse(text, ofPattern("ddMMyy"))` does not throw: two-digit day, month and year, in range.
fn parses_ddmmyy(text: &str) -> bool {
    text.len() == 6 && text.bytes().all(|b| b.is_ascii_digit()) && (1..=31).contains(&parse_int(&text[..2])) && (1..=12).contains(&parse_int(&text[2..4]))
}

/// `LocalDate.parse(text, ofPattern("yyMMdd")).getYear()`: 2000 plus the two digits, the month and day in range.
fn yymmdd_year(text: &str) -> i32 {
    assert!(text.len() == 6 && text.bytes().all(|b| b.is_ascii_digit()), "{text}");
    assert!((1..=12).contains(&parse_int(&text[2..4])) && (1..=31).contains(&parse_int(&text[4..])), "{text}");
    2000 + parse_int(&text[..2])
}

// ---------------------------------------------------------------------------------------------------------------
// AlbanianIdNumberTest

#[test]
fn albanian_valid() {
    let faker = Faker::new();
    for _ in 0..100 {
        assert_eq!(AlbanianIdNumber.generate_valid(&faker).unwrap().len(), 10);
    }
}

#[test]
fn albanian_invalid() {
    let faker = Faker::new();
    for _ in 0..100 {
        let pin = AlbanianIdNumber.generate_invalid(&faker).unwrap();
        assert_eq!(pin.len(), 10);
        assert!(parse_int(&pin[2..4]) > 92, "Presumably invalid PIN: '{pin}'");
    }
}

#[test]
fn albanian_yy() {
    let g = AlbanianIdNumber;
    for (year, want) in
        [(1806, "06"), (1812, "12"), (1829, "29"), (1880, "80"), (1888, "88"), (1900, "A0"), (1911, "B1"), (1929, "C9"), (1981, "I1"), (2003, "K3"), (2016, "L6"), (2099, "T9")]
    {
        assert_eq!(g.yy(year).unwrap(), want);
    }
}

#[test]
fn albanian_mm() {
    let g = AlbanianIdNumber;
    for (month, gender, want) in [(1, Gender::Male, "01"), (2, Gender::Male, "02"), (9, Gender::Male, "09"), (12, Gender::Male, "12")] {
        assert_eq!(g.mm(month, gender), want);
    }
    for (month, want) in [(1, "51"), (2, "52"), (8, "58"), (12, "62")] {
        assert_eq!(g.mm(month, Gender::Female), want);
    }
}

#[test]
fn albanian_dd() {
    for (day, want) in [(1, "01"), (9, "09"), (10, "10"), (31, "31")] {
        assert_eq!(AlbanianIdNumber.dd(day), want);
    }
}

#[test]
fn albanian_checksum() {
    for (text, want) in [("J11024000", 'R'), ("J45405000", 'O'), ("J00923000", 'N'), ("I05101999", 'I')] {
        assert_eq!(AlbanianIdNumber.checksum(text).unwrap(), want);
    }
}

#[test]
fn albanian_checksum_of_first_char() {
    for (c, want) in [('0', 0), ('1', 1), ('3', 3), ('9', 9), ('A', 1), ('B', 2), ('V', 22), ('W', 0)] {
        assert_eq!(AlbanianIdNumber.checksum_of_first_char(c), want);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// BulgarianIdNumberTest

#[test]
fn bulgarian_checksum() {
    for (text, want) in
        [("803205603", 1), ("800101000", 8), ("750102001", 8), ("820630876", 0), ("560628204", 7), ("752316926", 3), ("755201000", 5), ("754201103", 0)]
    {
        assert_eq!(BulgarianIdNumber.checksum(text), want, "{text}");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// ChineseIdNumberTest

fn assert_that_ssn_number_valid(id_number: &str) {
    let chars: Vec<char> = id_number.chars().collect();
    let last = chars.len().saturating_sub(1);
    let satisfied = chars.len() == 18 && chars.iter().enumerate().all(|(j, c)| c.is_ascii_digit() || j == last && *c == 'X');
    assert!(satisfied, "{id_number}");
}

#[test]
fn chinese_test_valid_chinese_id_number() {
    let faker = faker_in("zh", "CN");
    for _ in 0..10 {
        assert_that_ssn_number_valid(&faker.id_number().valid().unwrap());
    }
}

#[test]
fn chinese_test_checksum_of_chinese_id_number() {
    for id_number in [
        "420302198411073380",
        "310104196302135471",
        "330109197812225952",
        "130303200202132943",
        "370405198908011564",
        "350403198610147315",
        "640106198409178736",
        "469002199907068677",
        "440307199907065608",
        "150301196606308919",
        "21030319900605273X",
    ] {
        let first17: Vec<char> = id_number[..17].chars().collect();
        assert_eq!(ChineseIdNumber::id_number(&first17), id_number);
    }
}

#[test]
fn chinese_test_valid_zh_cn_id_number() {
    let faker = faker_in("zh", "CN");
    for _ in 0..100 {
        assert_that_ssn_number_valid(&ChineseIdNumber.generate_valid(&faker).unwrap());
    }
}

// ---------------------------------------------------------------------------------------------------------------
// EstonianIdNumberTest

#[test]
fn estonian_checksum() {
    for (text, want) in [("3760503029", 9), ("3450123421", 5), ("4940313652", 6), ("5110712176", 0), ("6110712176", 0)] {
        assert_eq!(EstonianIdNumber::checksum(text), want);
    }
}

fn estonian_first_digits(years: [i32; 6], male: i32, female: i32) {
    for year in years {
        assert_eq!(EstonianIdNumber::first_digit(year, Gender::Male).unwrap(), male);
        assert_eq!(EstonianIdNumber::first_digit(year, Gender::Female).unwrap(), female);
    }
}

#[test]
fn estonian_first_digit_18xx() {
    estonian_first_digits([1800, 1801, 1802, 1888, 1898, 1899], 1, 2);
}

#[test]
fn estonian_first_digit_19xx() {
    estonian_first_digits([1900, 1901, 1902, 1988, 1998, 1999], 3, 4);
}

#[test]
fn estonian_first_digit_20xx() {
    estonian_first_digits([2000, 2001, 2002, 2088, 2098, 2099], 5, 6);
}

#[test]
fn estonian_first_digit_21xx() {
    estonian_first_digits([2100, 2101, 2102, 2188, 2198, 2199], 7, 8);
}

// ---------------------------------------------------------------------------------------------------------------
// FrenchIdNumberTest

#[test]
fn french_control_key_samples() {
    let g = FrenchIdNumber;
    for (base, want) in [
        ("1510246102043", "25"),
        ("1850572070008", "58"),
        ("1211277131232", "50"),
        ("121122A131232", "64"),
        ("121122B010232", "35"),
        ("1790332046090", "01"),
        ("2757548135470", "48"),
        ("2092006990120", "61"),
    ] {
        assert_eq!(g.control_key(base).unwrap(), want, "{base}");
    }
}

#[test]
fn french_control_key() {
    let random = RandomService::default();
    for _ in 0..100 {
        let control_key = random.next_int_between(1, 96).unwrap() as i64;
        let factor_number = random.next_long_between(0, 99_999_999_998).unwrap();
        let base_part = 97 * factor_number + (97 - control_key);
        assert_eq!(FrenchIdNumber.control_key(&format!("{base_part:013}")).unwrap(), format!("{control_key:02}"));
    }
}

#[test]
fn french_valid_french_id_number() {
    let faker = faker_in("fr", "FR");
    for _ in 0..100 {
        let actual = FrenchIdNumber.generate_valid(&faker).unwrap();
        assert_matches(patterns::FRENCH, &actual);
        assert_eq!(FrenchIdNumber.control_key(&actual[..13]).unwrap(), &actual[13..]);
    }
}

#[test]
fn french_invalid_french_id_number() {
    let faker = faker_in("fr", "FR");
    for _ in 0..100 {
        let actual = FrenchIdNumber.generate_invalid(&faker).unwrap();
        assert_matches(patterns::FRENCH, &actual);
        assert_ne!(FrenchIdNumber.control_key(&actual[..13]).unwrap(), &actual[13..]);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// HungarianIdNumberTest

#[test]
fn hungarian_first_digit() {
    use Gender::{Female, Male};
    for (year, gender, want) in
        [(1899, Male, 3), (1900, Male, 1), (1901, Male, 1), (1899, Female, 4), (1900, Female, 2), (1901, Female, 2), (1999, Male, 1), (2000, Male, 3), (1999, Female, 2), (2000, Female, 4)]
    {
        assert_eq!(HungarianIdNumber::first_digit(year, gender), want);
    }
}

#[test]
fn hungarian_check_digit() {
    for (base, want) in [("807159215", 3), ("100000000", 1), ("010000000", 2), ("001000000", 3), ("000000001", 9), ("170010103", 10)] {
        assert_eq!(HungarianIdNumber::get_check_digit(base), want);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// IrishIdNumberTest

#[test]
fn irish_valid_irish_id_number() {
    let faker = faker_in("en", "IE");
    for _ in 0..100 {
        let actual = IrishIdNumber.generate_valid(&faker).unwrap();
        assert_matches(patterns::IRISH, &actual);
        assert!(IrishIdNumber.validate_and_check_modulo23(&actual), "{actual}");
    }
}

#[test]
fn irish_invalid_irish_id_number() {
    let faker = faker_in("en", "IE");
    for _ in 0..100 {
        let actual = IrishIdNumber.generate_invalid(&faker).unwrap();
        assert_matches(patterns::IRISH, &actual);
        assert!(!IrishIdNumber.validate_and_check_modulo23(&actual), "{actual}");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// ItalianIdNumberTest

#[test]
fn italian_consonants() {
    let g = ItalianIdNumber;
    for (name, want) in [
        ("OOOOOP", "POO"),
        ("HELLO", "HLL"),
        ("HELLO", "HLL"),
        ("ANNA", "NNA"),
        ("AIROLI", "RLA"),
        ("AIROUE", "RAI"),
        ("FO", "FOX"),
        ("OF", "FOX"),
        ("F", "FXX"),
        ("D'AMICO", "DMC"),
        ("D`AMICO", "DMC"),
        ("D_AMICO", "DMC"),
        ("DE ROSA", "DRS"),
        ("ÜÜLAR ÄHO", "LRH"),
        ("", "XXX"),
    ] {
        assert_eq!(g.encode_name(name), want, "{name}");
    }
}

#[test]
fn italian_checksum() {
    let g = ItalianIdNumber;
    assert_eq!(g.checksum("BBBTTT20H12X122").unwrap(), 'H');
    assert_eq!(g.checksum("AAAAAAAAAAAAAAA").unwrap(), 'I');
    assert_eq!(g.checksum("101010101010101").unwrap(), 'A');
    assert_eq!(g.checksum("H01010101010101").unwrap(), 'R');
}

#[test]
fn italian_checksum_should_match_for_valid_codes() {
    for _ in 0..100 {
        let person = ItalianIdNumber.generate_valid_for(&faker_in("it", "IT"), &IdNumberRequest::new(1, 200, GenderRequest::Any)).unwrap();
        let chars: Vec<char> = person.id_number.chars().collect();
        assert_eq!(ItalianIdNumber.checksum(&chars[..15].iter().collect::<String>()).unwrap(), chars[15], "{}", person.id_number);
    }
}

#[test]
fn italian_checksum_should_not_match_for_invalid_codes() {
    for _ in 0..10 {
        let id_number = ItalianIdNumber.generate_invalid(&faker_in("it", "IT")).unwrap();
        let chars: Vec<char> = id_number.chars().collect();
        assert_ne!(ItalianIdNumber.checksum(&chars[..15].iter().collect::<String>()).unwrap(), chars[15], "{id_number}");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// LatinLettersTest

#[test]
fn latin_letters_vowels() {
    for vowel in ['a', 'e', 'i', 'o', 'u'] {
        assert!(!idnumbers::is_consonant(vowel));
        assert!(!idnumbers::is_consonant(vowel.to_ascii_uppercase()));
    }
}

#[test]
fn latin_letters_consonants() {
    for consonant in "bcdfghjklmnpqrstvwxyz".chars() {
        assert!(idnumbers::is_consonant(consonant));
        assert!(idnumbers::is_consonant(consonant.to_ascii_uppercase()));
    }
}

#[test]
fn latin_letters_keeps_only_latin_letters() {
    assert_eq!(idnumbers::remove_non_latin_letters("John 1 Malkovi4"), "JohnMalkovi");
    assert_eq!(idnumbers::remove_non_latin_letters("πüäÄöÖ"), "");
    assert_eq!(idnumbers::remove_non_latin_letters("Ülar"), "lar");
    assert_eq!(idnumbers::remove_non_latin_letters("Андрей the Творожок"), "the");
}

// ---------------------------------------------------------------------------------------------------------------
// LatvianIdNumberTest

#[test]
fn latvian_checksum() {
    assert_eq!(LatvianIdNumber::checksum("080594-1684"), 1);
    assert_eq!(LatvianIdNumber::checksum("121282-1121"), 0);
    assert_eq!(LatvianIdNumber::checksum("121282-8888"), 2);
}

fn latvian_century_digits(years: [i32; 6], want: i32) {
    for year in years {
        assert_eq!(LatvianIdNumber::century_digit(year).unwrap(), want);
    }
}

#[test]
fn latvian_century_digit_18xx() {
    latvian_century_digits([1800, 1801, 1802, 1888, 1898, 1899], 0);
}

#[test]
fn latvian_century_digit_19xx() {
    latvian_century_digits([1900, 1901, 1902, 1988, 1998, 1999], 1);
}

#[test]
fn latvian_century_digit_20xx() {
    latvian_century_digits([2000, 2001, 2002, 2088, 2098, 2099], 2);
}

// ---------------------------------------------------------------------------------------------------------------
// MacedonianIdNumberTest

#[test]
fn macedonian_checksum() {
    assert_eq!(MacedonianIdNumber.checksum("010100650000"), 6);
    assert_eq!(MacedonianIdNumber.checksum("923456789012"), 4);
}

// ---------------------------------------------------------------------------------------------------------------
// MexicanIdNumberTest

const MEXICAN_REGEX: &str = r"[A-Z]{4}\d{6}[HM][A-Z]{5}[A-Z,0-9]\d";

#[test]
fn mexican_valid() {
    let faker = faker_in("es", "MX");
    for _ in 0..100 {
        assert_matches(MEXICAN_REGEX, &faker.id_number().valid().unwrap());
    }
}

#[test]
fn mexican_valid_es_mx_ssn() {
    let faker = faker_in("es", "MX");
    for _ in 0..100 {
        assert_matches(MEXICAN_REGEX, &faker.id_number().valid_es_mx_ssn().unwrap());
    }
}

#[test]
fn mexican_invalid() {
    let faker = faker_in("es", "MX");
    for _ in 0..100 {
        assert_matches(MEXICAN_REGEX, &faker.id_number().invalid().unwrap());
    }
}

#[test]
fn mexican_invalid_es_mx_ssn() {
    let faker = faker_in("es", "MX");
    for _ in 0..100 {
        assert_matches(MEXICAN_REGEX, &faker.id_number().invalid_es_mx_ssn().unwrap());
    }
}

// ---------------------------------------------------------------------------------------------------------------
// MoldovanIdNumberTest

#[test]
fn moldovan_valid() {
    let faker = Faker::new();
    for _ in 0..100 {
        let pin = MoldovanIdNumber.generate_valid(&faker).unwrap();
        assert_eq!(pin.len(), 13, "Presumably valid PIN: '{pin}'");
        assert_matches(r"\d{13}", &pin);
    }
}

#[test]
fn moldovan_checksum() {
    for (text, want) in [
        ("293270095431", '9'),
        ("201403700084", '3'),
        ("099220624701", '8'),
        ("200504212980", '9'),
        ("200504401269", '3'),
        ("200201100696", '1'),
        ("200403612722", '9'),
    ] {
        assert_eq!(MoldovanIdNumber.checksum(text), want);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// NorwegianIdNumberTest

#[test]
fn norwegian_valid() {
    let faker = Faker::new();
    for _ in 0..100 {
        let pin = NorwegianIdNumber.generate_valid(&faker).unwrap();
        assert_matches(r"\d{11}", &pin);
    }
}

#[test]
fn norwegian_checksum() {
    assert_eq!(NorwegianIdNumber.checksum("110779410"), 12);
    assert_eq!(NorwegianIdNumber.checksum("010203987"), 67);
}

#[test]
fn norwegian_base_part() {
    let faker = Faker::new();
    for (birthday, gender, expected_regex, min, max) in [
        ("1854-12-31", Gender::Male, "311254..ODD", 50, 74),
        ("1854-12-31", Gender::Female, "311254..EVEN", 50, 74),
        ("1900-01-01", Gender::Male, "010100..ODD", 0, 49),
        ("1940-12-31", Gender::Male, "311240..ODD", 90, 99),
        ("2010-12-31", Gender::Male, "311210..ODD", 50, 99),
    ] {
        let pin = NorwegianIdNumber.base_part(&faker, date(birthday), gender).unwrap();
        assert_eq!(pin.len(), 9);
        assert_matches(&expected_regex.replace("ODD", "[13579]").replace("EVEN", "[02468]").replace('.', r"\d"), &pin);
        assert!((min..=max).contains(&parse_int(&pin[6..8])), "{pin}");
    }
}

#[test]
fn norwegian_invalid() {
    let faker = Faker::new();
    for _ in 0..100 {
        let generated = NorwegianIdNumber.generate_invalid(&faker).unwrap();
        let base_part = &generated[..generated.len() - 2];
        let checksum = parse_int(&generated[generated.len() - 2..]);
        assert_eq!(generated.len(), 11);
        assert_matches(r"\d+", &generated);
        assert_ne!(checksum, NorwegianIdNumber.checksum(base_part));
    }
}

// ---------------------------------------------------------------------------------------------------------------
// PolishIdNumberTest

fn assert_control_digit(pesel: &str) {
    let c: Vec<i32> = pesel.chars().map(|c| c as i32).collect();
    let sum = (c[0] + c[4] + c[8] + c[10] + 3 * (c[1] + c[5] + c[9]) + 7 * (c[2] + c[6]) + 9 * (c[3] + c[7])) % 10;
    assert_eq!(sum, 0, "{pesel}");
}

#[test]
fn polish_id_number_consists_of_11_digits() {
    let faker = faker_in("pl", "PL");
    for _ in 0..100 {
        let actual = PolishIdNumber.generate_valid_for(&faker, &IdNumberRequest::new(0, 222, GenderRequest::Any)).unwrap();
        assert_matches(patterns::POLISH, &actual.id_number);
        assert_eq!(parse_int(&actual.id_number[..2]) % 100, actual.birth_date.year() % 100);
        assert_eq!(parse_int(&actual.id_number[4..6]), actual.birth_date.day_of_month() as i32);
        assert_control_digit(&actual.id_number);
    }
}

#[test]
fn polish_generate_id_number_for_female() {
    let birth_date = Faker::new().time_and_date().birthday_between(0, 100).unwrap();
    let actual = PolishIdNumber.get(&faker_in("pl", "PL"), birth_date, Some(PolishGender::Female)).unwrap();
    assert_eq!((actual.as_bytes()[9] - b'0') % 2, 0);
}

#[test]
fn polish_generate_id_number_for_male() {
    let birth_date = Faker::new().time_and_date().birthday_between(0, 100).unwrap();
    let actual = PolishIdNumber.get(&faker_in("pl", "PL"), birth_date, Some(PolishGender::Male)).unwrap();
    assert_eq!((actual.as_bytes()[9] - b'0') % 2, 1);
}

fn born_at(month: u32, year: i32) -> datafaker::Result<String> {
    PolishIdNumber.get(&faker_in("pl", "PL"), LocalDate::of(year, month, 28).unwrap(), Some(PolishGender::Any))
}

#[test]
fn polish_century_is_encoded_in_month_number() {
    for (month, year, want) in [
        (6, 1800, 86),
        (6, 1801, 86),
        (6, 1899, 86),
        (3, 1900, 3),
        (3, 1950, 3),
        (3, 1999, 3),
        (5, 2000, 25),
        (5, 2001, 25),
        (5, 2050, 25),
        (5, 2099, 25),
        (5, 2100, 45),
        (5, 2199, 45),
        (5, 2200, 65),
        (5, 2299, 65),
    ] {
        assert_eq!(parse_int(&born_at(month, year).unwrap()[2..4]), want, "{year}-{month}");
    }
}

#[test]
fn polish_too_early_centuries() {
    for year in [-100, 1699, 1799] {
        let e = born_at(6, year).unwrap_err();
        assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", format!("Year is before 1800: {year}").as_str()));
    }
}

#[test]
fn polish_too_late_centuries() {
    for year in [2300, 2399, 9999] {
        let e = born_at(6, year).unwrap_err();
        assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", format!("Year is after 2300: {year}").as_str()));
    }
}

// ---------------------------------------------------------------------------------------------------------------
// PortugueseIdNumberTest

#[test]
fn portuguese_test_invalid() {
    let faker = faker_in("pt", "PT");
    for _ in 0..100 {
        assert_matches("[0-9]{9,10}", &faker.id_number().invalid().unwrap());
    }
}

#[test]
fn portuguese_test_valid() {
    let faker = faker_in("pt", "PT");
    for _ in 0..100 {
        assert_matches("[0-9]{9,10}", &faker.id_number().valid().unwrap());
    }
}

// ---------------------------------------------------------------------------------------------------------------
// RomanianIdNumberTest

#[test]
fn romanian_sample() {
    let faker = Faker::new();
    for _ in 0..100 {
        assert_matches(r"\d{13}", &RomanianIdNumber.generate_valid(&faker).unwrap());
    }
}

fn romanian_first_digits(years: std::ops::RangeInclusive<i32>, male: i32, female: i32) {
    for year in years {
        let birthday = LocalDate::of(year, 1, 1).unwrap();
        assert_eq!(RomanianIdNumber.first_character(birthday, Gender::Male).unwrap(), male);
        assert_eq!(RomanianIdNumber.first_character(birthday, Gender::Female).unwrap(), female);
    }
}

#[test]
fn romanian_first_digit_18xx() {
    romanian_first_digits(1800..=1899, 3, 4);
}

#[test]
fn romanian_first_digit_19xx() {
    romanian_first_digits(1900..=1999, 1, 2);
}

#[test]
fn romanian_first_digit_20xx() {
    romanian_first_digits(2000..=2099, 5, 6);
}

#[test]
fn romanian_date_of_birth() {
    assert_eq!(RomanianIdNumber.date_of_birth(date("1990-01-01")).unwrap(), "900101");
    assert_eq!(RomanianIdNumber.date_of_birth(date("1234-12-31")).unwrap(), "341231");
}

#[test]
fn romanian_county_code() {
    let faker = Faker::new();
    let mut all_codes = HashSet::new();
    for _ in 0..10_000 {
        let county_code = RomanianIdNumber.county_code(&faker);
        assert_matches(r"\d{2}", &county_code);
        all_codes.insert(county_code);
    }
    assert_eq!(all_codes.len(), 48);
    for code in ["01", "09", "10", "11", "19", "20", "21", "45", "46", "51", "52"] {
        assert!(all_codes.contains(code), "{code}");
    }
    for code in ["53", "47", "49", "50"] {
        assert!(!all_codes.contains(code), "{code}");
    }
}

#[test]
fn romanian_sequence_number() {
    let faker = Faker::new();
    for _ in 0..10 {
        assert_matches(r"\d{3}", &RomanianIdNumber.sequence_number(&faker));
    }
}

#[test]
fn romanian_checksum() {
    assert_eq!(RomanianIdNumber.checksum("198081945678"), 1);
    assert_eq!(RomanianIdNumber.checksum("293052637289"), 4);
}

// ---------------------------------------------------------------------------------------------------------------
// SingaporeIdNumberTest

#[test]
fn singapore_century_prefix_for_citizens() {
    for (d, want) in [("1999-12-31", 'S'), ("2000-12-31", 'T'), ("2001-01-01", 'T'), ("2101-01-01", 'U'), ("2201-01-01", 'V')] {
        assert_eq!(SingaporeIdNumber::century_prefix_citizen(date(d)), want, "{d}");
    }
}

#[test]
fn singapore_century_prefix_for_foreigner() {
    for (d, want) in [("1999-12-31", 'F'), ("2000-12-31", 'G'), ("2001-01-01", 'G'), ("2101-01-01", 'H'), ("2201-01-01", 'I')] {
        assert_eq!(SingaporeIdNumber::century_prefix_foreigner(date(d)), want, "{d}");
    }
}

#[test]
fn singapore_random_birth_date_20th_century() {
    let faker = Faker::new();
    for _ in 0..100 {
        assert_eq!(SingaporeIdNumber::random_birth_date(&faker, SingaporeType::SingaporeanTwentiethCentury).unwrap().year() / 100, 19);
        assert_eq!(SingaporeIdNumber::random_birth_date(&faker, SingaporeType::ForeignerTwentiethCentury).unwrap().year() / 100, 19);
    }
}

#[test]
fn singapore_random_birth_date_21th_century() {
    let faker = Faker::new();
    for _ in 0..100 {
        assert_eq!(SingaporeIdNumber::random_birth_date(&faker, SingaporeType::SingaporeanTwentyFirstCentury).unwrap().year() / 100, 20);
        assert_eq!(SingaporeIdNumber::random_birth_date(&faker, SingaporeType::ForeignerTwentyFirstCentury).unwrap().year() / 100, 20);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// SouthAfricanIdNumberTest

#[test]
fn south_african_test_exist_ssn() {
    assert!(!SouthAfricanIdNumber::is_valid_en_za_ssn("9202204720085"));
    assert!(!SouthAfricanIdNumber::is_valid_en_za_ssn("foo2204720082"));
    assert!(!SouthAfricanIdNumber::is_valid_en_za_ssn("9232454720082"));
    assert!(SouthAfricanIdNumber::is_valid_en_za_ssn("9202204720083"));
    assert!(SouthAfricanIdNumber::is_valid_en_za_ssn("8801235111088"));
}

#[test]
fn south_african_test_faker_ssn() {
    for _ in 0..100 {
        let f = faker_in("en", "ZA");
        assert!(SouthAfricanIdNumber::is_valid_en_za_ssn(&f.id_number().valid().unwrap()));
        assert!(!SouthAfricanIdNumber::is_valid_en_za_ssn(&f.id_number().invalid().unwrap()));
    }
}

#[test]
fn south_african_test_ssn_format() {
    for _ in 0..100 {
        let f = faker_in("en", "ZA");
        assert_matches(r"\d{10}[01]8\d", &f.id_number().valid().unwrap());
        assert_matches(r"\d{10}[01]8\d", &f.id_number().invalid().unwrap());
    }
}

#[test]
fn south_african_sequential_number_for_males() {
    for _ in 0..100 {
        let sequential_number = SouthAfricanIdNumber::sequential_number(&faker_in("en", "ZA"), Gender::Male);
        assert_matches(r"\d{4}", &sequential_number);
        assert!(parse_int(&sequential_number) >= 5000);
    }
}

#[test]
fn south_african_sequential_number_for_females() {
    for _ in 0..100 {
        let sequential_number = SouthAfricanIdNumber::sequential_number(&faker_in("en", "ZA"), Gender::Female);
        assert_matches(r"\d{4}", &sequential_number);
        assert!(parse_int(&sequential_number) < 5000);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// SouthKoreanIdNumberTest

#[test]
fn south_korean_test_valid_ko_kr_rrn() {
    for _ in 0..100 {
        let rrn = faker_in("en", "KR").id_number().valid().unwrap();
        assert_matches(r"\d{6}-\d{7}", &rrn);
        let year = yymmdd_year(&rrn[..6]);
        let current_year = LocalDate::now().year();
        assert!((current_year - 80..=current_year + 80).contains(&year), "{rrn}");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// SwedishIdNumberTest

#[test]
fn swedish_ssn_contains_plus_for_persons_older_than_100_years() {
    let faker = Faker::new();
    for _ in 0..20 {
        let person = SwedenIdNumber.generate_valid_for(&faker, &IdNumberRequest::new(101, 125, GenderRequest::Any)).unwrap();
        assert_matches(r"(\d{6})\+(\d{4})", &person.id_number);
        assert!(SwedenIdNumber::is_valid_swedish_ssn(&person.id_number).unwrap(), "{}", person.id_number);
    }
}

#[test]
fn swedish_ssn_contains_minus_for_persons_younger_than_100_years() {
    let faker = Faker::new();
    for _ in 0..20 {
        let person = SwedenIdNumber.generate_valid_for(&faker, &IdNumberRequest::new(23, 99, GenderRequest::Any)).unwrap();
        assert_matches(r"(\d{6})-(\d{4})", &person.id_number);
        assert!(SwedenIdNumber::is_valid_swedish_ssn(&person.id_number).unwrap(), "{}", person.id_number);
    }
}

#[test]
fn swedish_valid_swedish_ssn() {
    for ssn in ["670919-9530", "811228-9874", "000229-9873", "991221+4146", "991227+0262"] {
        assert!(SwedenIdNumber::is_valid_swedish_ssn(ssn).unwrap(), "{ssn}");
    }
}

#[test]
fn swedish_invalid_swedish_ssn() {
    for ssn in ["020914-0003", "8112289873", "foo228-9873", "811228-9873", "811228-9875", "811200-9874", "810028-9874", "000229+9873"] {
        assert!(!SwedenIdNumber::is_valid_swedish_ssn(ssn).unwrap(), "{ssn}");
    }
}

#[test]
fn swedish_is_year_over_100_years_ago() {
    let today = LocalDate::of(2024, 6, 1).unwrap();
    for (year, want) in [("1900", true), ("1918", true), ("1924", true), ("1925", false), ("1990", false), ("2003", false), ("2035", false)] {
        assert_eq!(SwedenIdNumber::is_year_over_100_years_ago(year, today).unwrap(), want, "{year}");
    }
}

#[test]
fn swedish_find_year_beginning_from_ssn() {
    for (ssn, want) in
        [("670919-9530", "19"), ("811228-9874", "19"), ("000225-9873", "20"), ("000225+9877", "19"), ("991221+4146", "18"), ("981227+0262", "18")]
    {
        assert_eq!(SwedenIdNumber::find_year_beginning_from_ssn(ssn).unwrap(), want, "{ssn}");
    }
}

// ---------------------------------------------------------------------------------------------------------------
// UkrainianIdNumberTest

fn is_unzr_valid(unzr: &str) -> bool {
    let d: Vec<i32> = unzr.replace('-', "").chars().map(|c| c.to_digit(10).map_or(-1, |v| v as i32)).collect();
    let checksum = d[0] * 7 + d[1] * 3 + d[2] + d[3] * 7 + d[4] * 3 + d[5] + d[6] * 7 + d[7] * 3 + d[8] + d[9] * 7 + d[10] * 3 + d[11];
    checksum % 10 == d[12]
}

#[test]
fn ukrainian_valid() {
    let person = UkrainianIdNumber.generate_valid_for(&Faker::new(), &IdNumberRequest::new(1, 65, GenderRequest::Any)).unwrap();
    let (unzr, bod) = (&person.id_number, person.birth_date);
    assert_eq!(bod.year(), parse_int(&unzr[..4]));
    assert_eq!(bod.month_value() as i32, parse_int(&unzr[4..6]));
    assert_eq!(bod.day_of_month() as i32, parse_int(&unzr[6..8]));
    assert!(is_unzr_valid(unzr), "{unzr}");
}

#[test]
fn ukrainian_invalid() {
    assert!(!is_unzr_valid(&UkrainianIdNumber.generate_invalid(&Faker::new()).unwrap()));
}

// ---------------------------------------------------------------------------------------------------------------
// UtilsTest

#[test]
fn utils_digit_parses_given_char_to_number() {
    for (c, want) in [('0', 0), ('1', 1), ('2', 2), ('8', 8), ('9', 9)] {
        assert_eq!(idnumbers::digit(c), want);
    }
}

#[test]
fn utils_digit_at_parses_given_char_to_number() {
    for (i, want) in [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)] {
        assert_eq!(idnumbers::digit_at("12345", i), want);
    }
}

#[test]
fn utils_multiply_digits() {
    assert_eq!(idnumbers::multiply("1", &[1]), 1);
    assert_eq!(idnumbers::multiply("1", &[2]), 2);
    assert_eq!(idnumbers::multiply("23", &[4, 5]), 2 * 4 + 3 * 5);
}

// ---------------------------------------------------------------------------------------------------------------
// The facade's test class, `providers/base`: the cases the generator leaves to hand.

#[test]
fn id_number_test_ssn_valid() {
    let (faker, us) = (Faker::new(), faker_in("en", "US"));
    for _ in 0..100 {
        assert_matches(r"[0-8]\d{2}-\d{2}-\d{4}", &faker.id_number().ssn_valid().unwrap());
        assert_matches(r"[0-8]\d{2}-\d{2}-\d{4}", &us.id_number().valid().unwrap());
    }
}

#[test]
fn id_number_test_ssn_invalid() {
    let us = faker_in("en", "US");
    for _ in 0..100 {
        let invalid_ssn = us.id_number().invalid().unwrap();
        if !invalid_ssn.starts_with('9') {
            assert_matches(r"[0-8]\d{2}-\d{2}-\d{4}", &invalid_ssn);
        }
    }
}

#[test]
fn id_number_test_valid_swedish_ssn() {
    let swedish = faker_in("sv", "SE");
    for _ in 0..100 {
        let actual = swedish.id_number().valid().unwrap();
        assert_matches(patterns::SWEDISH, &actual);
        assert!(SwedenIdNumber::is_valid_swedish_ssn(&actual).unwrap(), "{actual}");
    }
}

#[test]
fn id_number_test_invalid_swedish_ssn() {
    let swedish = faker_in("sv", "SE");
    for _ in 0..100 {
        let actual = swedish.id_number().invalid().unwrap();
        assert_matches(patterns::SWEDISH, &actual);
        assert!(!SwedenIdNumber::is_valid_swedish_ssn(&actual).unwrap(), "{actual}");
    }
}

#[test]
fn id_number_test_south_africa_valid() {
    let south_africa = faker_in("en", "ZA");
    for _ in 0..100 {
        let actual = south_africa.id_number().valid().unwrap();
        assert_matches(patterns::SOUTH_AFRICAN, &actual);
        assert!(SouthAfricanIdNumber::is_valid_en_za_ssn(&actual), "{actual}");
    }
}

#[test]
fn id_number_test_south_africa_invalid() {
    let south_africa = faker_in("en", "ZA");
    for _ in 0..100 {
        assert_matches(patterns::SOUTH_AFRICAN, &south_africa.id_number().invalid().unwrap());
    }
}

/// Each of `faker`'s `valid()` (or `invalid()`) results, a hundred times, matches `regex`.
fn hundred(faker: &Faker, valid: bool, regex: &str) {
    for _ in 0..100 {
        let pin = if valid { faker.id_number().valid() } else { faker.id_number().invalid() }.unwrap();
        assert!(matches(regex, &pin), "PIN: {pin} does not match {regex}");
    }
}

#[test]
fn id_number_test_estonian_personal_code_valid() {
    hundred(&faker_in("et", "EE"), true, patterns::ESTONIAN);
}

#[test]
fn id_number_test_estonian_personal_code_invalid() {
    hundred(&faker_in("et", "EE"), false, patterns::ESTONIAN);
}

#[test]
fn id_number_test_brazilian_personal_code_valid() {
    let brazilian = faker_in("pt", "BR");
    for _ in 0..100 {
        let actual = brazilian.id_number().valid().unwrap();
        assert_matches(patterns::BRAZILIAN, &actual);
        assert!(pt_br::is_cpf_valid(&actual), "Current value {actual}");
    }
}

#[test]
fn id_number_test_brazilian_personal_code_invalid() {
    let brazilian = faker_in("pt", "BR");
    for _ in 0..100 {
        let actual = brazilian.id_number().invalid().unwrap();
        assert_matches(patterns::BRAZILIAN, &actual);
        assert!(!pt_br::is_cpf_valid(&actual), "Current value {actual}");
    }
}

#[test]
fn id_number_test_latvian_personal_code_valid_old_format_before_2017_07_01() {
    let latvian = faker_in("lv", "LV");
    let min_age = LocalDate::now().year() - 2017 + 1;
    for _ in 0..100 {
        let id_number = latvian.id_number().valid_for(&IdNumberRequest::new(min_age, min_age + 50, GenderRequest::Any)).unwrap().id_number;
        assert_matches(r"[0-3]\d[0-1]\d{3}-[0-2]\d{4}", &id_number);
        assert!(parses_ddmmyy(&id_number[..6]), "{id_number}");
    }
}

#[test]
fn id_number_test_latvian_personal_code_valid_new_format_since_2017_07_01() {
    let latvian = faker_in("lv", "LV");
    let max_age = LocalDate::now().year() - 2017 - 1;
    for _ in 0..100 {
        let id_number = latvian.id_number().valid_for(&IdNumberRequest::new(0, max_age, GenderRequest::Any)).unwrap().id_number;
        assert_matches(r"3[2-9]\d{9}", &id_number);
    }
}

#[test]
fn id_number_test_latvian_personal_code_invalid() {
    hundred(&faker_in("lv", "LV"), false, r"[0-3]\d[0-1]\d{3}-[0-2]\d{4}");
}

#[test]
fn id_number_test_albanian_personal_code_valid() {
    let albanian = faker_in("sq", "AL");
    for _ in 0..100 {
        let pin = albanian.id_number().valid().unwrap();
        assert_matches(r"\w\d{8}\w", &pin);
        assert!((1..=12).contains(&(parse_int(&pin[2..4]) % 50)), "Valid PIN {pin} should have month number between 1..12 or 51..62");
    }
}

#[test]
fn id_number_test_albanian_personal_code_invalid() {
    let albanian = faker_in("sq", "AL");
    for _ in 0..100 {
        let pin = albanian.id_number().invalid().unwrap();
        assert_matches(r"\w\d{8}\w", &pin);
        assert!(parse_int(&pin[2..4]) > 62, "Invalid PIN {pin} should have month greater than (any month + 50)");
    }
}

#[test]
fn id_number_test_moldova_personal_code_valid() {
    for language in ["en", "ro", "ru"] {
        hundred(&faker_in(language, "MD"), true, r"\d{13}");
    }
}

#[test]
fn id_number_test_moldova_personal_code_invalid() {
    for language in ["en", "ro", "ru"] {
        hundred(&faker_in(language, "MD"), false, r"\d{13}");
    }
}

#[test]
fn id_number_test_bulgarian_personal_code_valid() {
    hundred(&faker_in("bg", "BG"), true, r"\d{10}");
}

#[test]
fn id_number_test_bulgarian_personal_code_invalid() {
    hundred(&faker_in("bg", "BG"), false, r"\d{10}");
}

#[test]
fn id_number_test_macedonian_personal_code_valid() {
    hundred(&faker_in("mk", "MK"), true, r"\d{13}");
}

#[test]
fn id_number_test_romanian_personal_code_valid() {
    hundred(&faker_in("ro", "RO"), true, r"\d{13}");
}

#[test]
fn id_number_test_macedonian_personal_code_invalid() {
    hundred(&faker_in("mk", "MK"), false, r"\d{13}");
}

#[test]
fn id_number_test_ukrainian_uznr_valid() {
    hundred(&faker_in("uk", "UA"), true, patterns::UKRAINIAN);
}

#[test]
fn id_number_test_ukrainian_uznr_invalid() {
    hundred(&faker_in("uk", "UA"), false, patterns::UKRAINIAN);
}

#[test]
fn id_number_test_french_id_number() {
    assert_matches(patterns::FRENCH, &faker_in("fr", "FR").id_number().valid().unwrap());
}

#[test]
fn id_number_test_italian_id_number_sample() {
    assert_matches(patterns::ITALIAN, "PTRJHN89T04Z222B");
    assert_matches(patterns::ITALIAN, "BBBTTT20H12X122H");
}

#[test]
fn id_number_test_italian_id_number() {
    hundred(&faker_in("it", "IT"), true, patterns::ITALIAN);
}

#[test]
fn id_number_test_hungarian_id_number_valid() {
    hundred(&faker_in("hu", "HU"), true, patterns::HUNGARIAN);
}

#[test]
fn id_number_test_hungarian_id_number_invalid() {
    hundred(&faker_in("hu", "HU"), false, patterns::HUNGARIAN);
}

#[test]
fn id_number_test_norwegian_id_number_valid() {
    hundred(&faker_in("no", "NO"), true, patterns::NORWEGIAN);
}

#[test]
fn id_number_test_norwegian_id_number_invalid() {
    hundred(&faker_in("no", "NO"), false, patterns::NORWEGIAN);
}

/// Not upstream: the registry holds one generator per country of `META-INF/services` (Irish is not listed), and the
/// facade's request overload fails for a country without one.
#[test]
fn registry_and_unsupported_country() {
    let codes: Vec<&str> = idnumbers::generators().iter().map(|g| g.country_code()).collect();
    assert_eq!(codes.len(), 23);
    assert_eq!(codes.iter().collect::<HashSet<_>>().len(), 23);
    assert!(idnumbers::generator_for("IE").is_none());
    let e = faker_in("en", "IE").id_number().valid_for(&IdNumberRequest::new(18, 65, GenderRequest::Any)).unwrap_err();
    assert_eq!(e.message(), "ID Number generation not supported for country 'IE'");
}
