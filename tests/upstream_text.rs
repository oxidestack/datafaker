//! `TextTest` (Datafaker 2.7.0, `providers/base/TextTest.java`). Unseeded fakers as upstream; repeated tests run
//! their body ten times.

mod common;

use common::assert_matches;
use datafaker::Faker;
use datafaker::providers::text::{DEFAULT_SPECIAL, DIGITS, EN_LOWERCASE, EN_UPPERCASE, TextSymbolsBuilder};

fn faker() -> Faker {
    Faker::new()
}

#[track_caller]
fn assert_err<T: std::fmt::Debug>(r: datafaker::Result<T>, message: &str) {
    let e = r.unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", message));
}

#[test]
fn text_should_contain_3_ru_lower_case_and_5_custom_special_symbols() {
    let f = faker();
    let ru = "абвгдеёжзийклмнопрстуфхцчшщъыьэюя";
    let special = "!@#$%^&*;'][{}";
    let config = TextSymbolsBuilder::builder()
        .len(f.number().number_between(8, 10))
        .with_times(ru, 3)
        .unwrap()
        .with_times(special, 5)
        .unwrap()
        .build()
        .unwrap();
    for _ in 0..10 {
        let text = f.text().text_from(&config).unwrap();
        assert!(text.chars().filter(|c| ru.contains(*c)).count() >= 3, "{text}");
        assert!(text.chars().filter(|c| special.contains(*c)).count() >= 5, "{text}");
    }
}

#[test]
fn exception_if_length_is_shorter_than_number_of_required_symbols() {
    let built = TextSymbolsBuilder::builder()
        .len(1)
        .with_times(EN_LOWERCASE, 1)
        .and_then(|b| b.with_times(EN_UPPERCASE, 1))
        .and_then(|b| b.with_times(DIGITS, 1))
        .and_then(|b| b.throw_if_length_small(true).build());
    assert_err(built, "Min length (1) should be not smaller than number of required characters (3)");
}

#[test]
fn every_text_should_contain_lower_case_upper_case_and_digit() {
    let f = faker();
    let config = TextSymbolsBuilder::builder()
        .len(f.number().number_between(6, 10))
        .with_times(EN_LOWERCASE, 1)
        .and_then(|b| b.with_times(EN_UPPERCASE, 1))
        .and_then(|b| b.with_times(DIGITS, 1))
        .and_then(|b| b.build())
        .unwrap();
    for _ in 0..1000 {
        let pw = f.text().text_from(&config).unwrap();
        assert!(pw.chars().any(|c| c.is_lowercase()), "{pw}");
        assert!(pw.chars().any(|c| c.is_uppercase()), "{pw}");
        assert!(pw.chars().any(|c| c.is_ascii_digit()), "{pw}");
    }
}

#[test]
fn test_character() {
    for _ in 0..10 {
        assert_matches("[A-Za-z]", &faker().text().character().unwrap().to_string());
    }
}

#[test]
fn test_uppercase_character() {
    for _ in 0..10 {
        assert!(faker().text().uppercase_character().unwrap().is_uppercase());
    }
}

#[test]
fn test_lowercase_character() {
    for _ in 0..10 {
        assert!(faker().text().lowercase_character().unwrap().is_lowercase());
    }
}

#[test]
fn test_fixed_length_text() {
    let f = faker();
    for i in 0..10 {
        let t = f.text().text_of_length(i).unwrap();
        assert_eq!(t.len(), i as usize);
        assert_matches("[a-z]*", &t);
    }
}

#[test]
fn test_default_length_text() {
    for _ in 0..10 {
        assert_matches("[a-z]{20,80}", &faker().text().text().unwrap());
    }
}

#[test]
fn up_to_64_lower_case() {
    for _ in 0..10 {
        let f = faker();
        assert_matches("[a-z]{1,64}", &f.text().text_with(1, 64, false, false, false).unwrap());
        assert_matches("[a-z]{2,64}", &f.text().text_with(2, 64, false, false, false).unwrap());
        assert_matches("[a-z]{64}", &f.text().text_with(64, 64, false, false, false).unwrap());
    }
}

#[test]
fn zero_length() {
    assert_eq!(faker().text().text_with(0, 0, false, false, false).unwrap(), "");
}

#[test]
fn one_lower_case() {
    let f = faker();
    assert_matches("[a-z]", &f.text().text_with(1, 1, false, false, false).unwrap());
    assert_matches("[a-z]?", &f.text().text_with(0, 1, false, false, false).unwrap());
}

#[test]
fn one_with_upper_case() {
    for _ in 0..10 {
        assert_matches("[A-Z]", &faker().text().text_with(1, 1, true, false, false).unwrap());
    }
}

#[test]
fn one_with_digit() {
    for _ in 0..10 {
        assert_matches("[0-9]", &faker().text().text_with(1, 1, false, false, true).unwrap());
    }
}

#[test]
fn one_with_special_symbol() {
    for _ in 0..10 {
        assert_matches(&format!("[{DEFAULT_SPECIAL}]"), &faker().text().text_with(1, 1, false, true, false).unwrap());
    }
}

#[test]
fn two_with_upper_case_and_digit() {
    for _ in 0..10 {
        assert_matches("[A-Z0-9]{2}", &faker().text().text_with(2, 2, true, false, true).unwrap());
    }
}

#[test]
fn two_with_lower_and_upper_case_and_digit() {
    for _ in 0..10 {
        let f = faker();
        assert_matches("[a-zA-Z0-9]{2}", &f.text().text_with(2, 2, true, false, true).unwrap());
        assert_matches(&format!("[a-zA-Z0-9{DEFAULT_SPECIAL}]{{3}}"), &f.text().text_with(3, 3, true, true, true).unwrap());
    }
}

#[test]
fn min_length_cannot_be_greater_than_max_length() {
    let f = faker();
    assert_err(f.text().text_with(22, 21, false, false, false), "Min length (22) should not be greater than max length (21)");
    assert_err(f.text().text_with(3, 2, true, true, true), "Min length (3) should not be greater than max length (2)");
}

#[test]
fn is_not_enough_length_to_contain_all_required_symbols() {
    let f = faker();
    assert_err(
        f.text().text_with(0, 0, true, false, false),
        "Minimum number of required characters (1) should not be greater than max length (0)",
    );
    assert_err(
        f.text().text_with(1, 2, true, true, true),
        "Minimum number of required characters (3) should not be greater than max length (2)",
    );
}

#[test]
fn minimum_length_is_not_enough_to_contain_all_required_symbols() {
    for _ in 0..10 {
        assert_matches("[a-zA-Z0-9]{2,4}", &faker().text().text_with(1, 4, true, false, true).unwrap());
    }
}
