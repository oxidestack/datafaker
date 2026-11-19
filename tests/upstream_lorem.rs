//! `LoremTest` (Datafaker 2.7.0, `providers/base/LoremTest.java`), with its provider-list cases. Unseeded fakers
//! as upstream; repeated tests run their body ten times.

mod common;

use common::{assert_matches, provider_list};
use datafaker::Faker;

fn faker() -> Faker {
    Faker::new()
}

#[track_caller]
fn assert_err<T: std::fmt::Debug>(r: datafaker::Result<T>, class: &str, message: &str) {
    let e = r.unwrap_err();
    assert_eq!((e.simple_class(), e.message()), (class, message));
}

#[test]
fn provider_list_test() {
    let f = faker();
    provider_list(&f, "lorem.words", &f.lorem().word().unwrap(), "");
    provider_list(&f, "lorem.supplemental", &f.lorem().supplemental().unwrap(), "");
}

#[test]
fn should_create_fixed_length_string() {
    let l = faker();
    assert_eq!(l.lorem().fixed_string(10).unwrap().chars().count(), 10);
    assert_eq!(l.lorem().fixed_string(50).unwrap().chars().count(), 50);
    assert!(l.lorem().fixed_string(0).unwrap().is_empty());
    assert!(l.lorem().fixed_string(-1).unwrap().is_empty());
}

#[test]
fn test_character() {
    assert_matches(r"[a-z\d]", &faker().lorem().character().unwrap().to_string());
}

#[test]
fn test_character_include_upper_case() {
    let f = faker();
    assert_matches(r"[a-z\d]", &f.lorem().character_with(false).unwrap().to_string());
    assert_matches(r"[a-zA-Z\d]", &f.lorem().character_with(true).unwrap().to_string());
}

#[test]
fn test_characters_should_include_min_and_max_length() {
    let f = faker();
    let mut lengths = std::collections::HashSet::new();
    for _ in 0..300 {
        let c = f.lorem().characters_between(1, 10).unwrap();
        assert_matches(r"[a-z\d]{1,10}", &c);
        lengths.insert(c.len());
    }
    assert!(lengths.contains(&1) && lengths.contains(&10));
}

#[test]
fn test_characters() {
    assert_matches(r"[a-z\d]{255}", &faker().lorem().characters().unwrap());
}

#[test]
fn test_characters_include_upper_case() {
    let f = faker();
    assert_matches(r"[a-z\d]{255}", &f.lorem().characters_fixed(255, false, false, false).unwrap());
    assert_matches(r"[a-zA-Z\d]{255}", &f.lorem().characters_fixed(255, true, false, false).unwrap());
}

#[test]
fn test_characters_with_length() {
    let f = faker();
    assert_matches(r"[a-z\d]{2}", &f.lorem().characters_fixed(2, false, false, false).unwrap());
    assert_matches(r"[a-z\d]{500}", &f.lorem().characters_fixed(500, false, false, false).unwrap());
    assert!(f.lorem().characters_fixed(0, false, false, false).unwrap().is_empty());
    assert_err(
        f.lorem().characters_fixed(-1, false, false, false),
        "IllegalArgumentException",
        "Minimum number of required characters (0) should not be greater than max length (-1)",
    );
}

#[test]
fn test_characters_with_length_include_uppercase() {
    let f = faker();
    let l = f.lorem();
    assert_matches(r"[a-z\d]{2}", &l.characters_fixed(2, false, false, false).unwrap());
    assert_matches(r"[a-z\d]{500}", &l.characters_fixed(500, false, false, false).unwrap());
    assert_matches(r"[a-zA-Z\d]{2}", &l.characters_fixed(2, true, false, false).unwrap());
    assert_matches(r"[a-zA-Z\d]{500}", &l.characters_fixed(500, true, false, false).unwrap());
    assert!(l.characters_fixed(0, false, false, false).unwrap().is_empty());
    assert_matches("[A-Z]", &l.characters_fixed(1, true, false, false).unwrap());
    assert_err(
        l.characters_fixed(-1, false, false, false),
        "IllegalArgumentException",
        "Minimum number of required characters (0) should not be greater than max length (-1)",
    );
    assert_err(
        l.characters_fixed(0, true, false, false),
        "IllegalArgumentException",
        "Minimum number of required characters (1) should not be greater than max length (0)",
    );
}

#[test]
fn test_characters_minimum_maximum_length() {
    assert_matches(r"[a-z\d]{1,10}", &faker().lorem().characters_between(1, 10).unwrap());
}

#[test]
fn test_characters_minimum_maximum_length_equals() {
    for _ in 0..10 {
        assert_matches(r"[a-z\d]{5}", &faker().lorem().characters_between(5, 5).unwrap());
    }
}

/// `characters(6, 10, true, true)`: upper case and digits.
#[test]
fn test_characters_minimum_maximum_length_equals_including_uppercase_and_including_digit() {
    for _ in 0..10 {
        assert_matches(r"[a-zA-Z\d]{6,10}", &faker().lorem().characters_with(6, 10, true, false, true).unwrap());
    }
}

#[test]
fn test_characters_fixed_length_including_uppercase_and_including_digit() {
    for _ in 0..10 {
        assert_matches(r"[a-zA-Z\d]{10}", &faker().lorem().characters_fixed(10, true, false, true).unwrap());
    }
}

#[test]
fn test_fixed_number_of_characters_empty() {
    let f = faker();
    assert!(f.lorem().characters_fixed(0, false, false, false).unwrap().is_empty());
    assert_err(
        f.lorem().characters_fixed(-1, true, true, true),
        "IllegalArgumentException",
        "Minimum number of required characters (3) should not be greater than max length (-1)",
    );
    assert_err(
        f.lorem().characters_fixed(-1, false, false, false),
        "IllegalArgumentException",
        "Minimum number of required characters (0) should not be greater than max length (-1)",
    );
}

#[test]
fn test_characters_minimum_maximum_length_include_uppercase() {
    assert_matches(r"[a-zA-Z\d]{1,10}", &faker().lorem().characters_with(1, 10, true, false, false).unwrap());
}

#[test]
fn test_characters_minimum_maximum_length_include_uppercase_include_digit() {
    let f = faker();
    assert_matches("[a-zA-Z]{1,10}", &f.lorem().characters_with(1, 10, false, false, false).unwrap());
    assert_matches(r"[a-zA-Z\d]{1,10}", &f.lorem().characters_with(2, 10, true, false, true).unwrap());
}

#[test]
fn test_sentence() {
    let s = faker().lorem().sentence().unwrap();
    assert!((3..=9).contains(&s.split(' ').count()));
    assert!(s.ends_with('.'));
}

#[test]
fn test_sentence_with_word_count() {
    let s = faker().lorem().sentence_of(10).unwrap();
    assert!((9..=15).contains(&s.split(' ').count()));
    assert!(s.ends_with('.'));
}

#[test]
fn test_sentence_with_word_count_and_random_words_to_add() {
    for _ in 0..10 {
        assert_matches(r"(\w+\s?){10,20}\.", &faker().lorem().sentence_with(10, 10).unwrap());
    }
}

#[test]
fn test_sentence_fixed_number_of_words() {
    for _ in 0..10 {
        assert_matches(r"(\w+\s?){10}\.", &faker().lorem().sentence_with(10, 0).unwrap());
    }
}

#[test]
fn test_words() {
    assert!(!faker().lorem().words(3).unwrap().is_empty());
}

#[test]
fn test_max_length_sentence() {
    let rand = datafaker::java::Random::from_entropy();
    for _ in 0..10 {
        let length = (rand.next_int_bounded(10000).unwrap()).abs();
        assert_eq!(faker().lorem().max_length_sentence(length).unwrap().chars().count(), length as usize);
    }
}

#[test]
fn test_max_length_with_empty_sentence() {
    assert!(faker().lorem().max_length_sentence(0).unwrap().is_empty());
}

#[test]
fn test_max_length_with_negative_length_sentence() {
    assert!(faker().lorem().max_length_sentence(-1).unwrap().is_empty());
}

#[test]
fn test_sentences() {
    for _ in 0..10 {
        let p = faker().lorem().paragraph().unwrap();
        assert!((3..=6).contains(&p.matches('.').count()));
    }
}

#[test]
fn test_sentences_with_count() {
    for _ in 0..10 {
        let p = faker().lorem().paragraph_of(1).unwrap();
        assert!((1..=3).contains(&p.matches('.').count()));
    }
}
