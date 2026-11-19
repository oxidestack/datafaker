//! `NameTest` (Datafaker 2.7.0, `providers/base/NameTest.java`), with its nested Greek test. Unseeded fakers as
//! upstream; repeated tests run their body ten times.

mod common;

use common::assert_matches;
use datafaker::{Faker, Locale};

fn faker() -> Faker {
    Faker::new()
}

#[test]
fn test_name() {
    assert_matches(r"([\w']+\.?( )?){2,4}", &faker().name().name().unwrap());
}

#[test]
fn test_name_with_middle() {
    assert_matches(r"([\w']+\.?( )?){3,}", &faker().name().name_with_middle().unwrap());
}

#[test]
fn test_name_with_middle_does_not_have_repeated_name() {
    let f = faker();
    let total = 100;
    let same = (0..total)
        .filter(|_| {
            let n = f.name().name_with_middle().unwrap();
            let parts: Vec<&str> = n.split(' ').collect();
            parts[0] == parts[1]
        })
        .count();
    assert!(same < total / 10);
}

#[test]
fn test_full_name() {
    assert_matches(r"([\w']+\.?( )?){2,4}", &faker().name().full_name().unwrap());
}

#[test]
fn test_full_name_arabic() {
    let f = Faker::with_locale(Locale::language_only("ar"));
    for _ in 0..25 {
        assert_matches(r"^[\x{0600}-\x{06FF}\x{0750}-\x{077F} ]+$", &f.name().full_name().unwrap());
    }
}

#[test]
fn test_first_name() {
    assert_matches(r"\w+", &faker().name().first_name().unwrap());
}

#[test]
fn test_female_first_name() {
    for _ in 0..10 {
        assert_matches(r"\w+", &faker().name().female_first_name().unwrap());
    }
}

#[test]
fn test_male_first_name() {
    for _ in 0..10 {
        assert_matches(r"\w+", &faker().name().male_first_name().unwrap());
    }
}

#[test]
fn test_last_name() {
    assert_matches("[A-Za-z']+", &faker().name().last_name().unwrap());
}

#[test]
fn test_prefix() {
    assert_matches(r"\w+\.?", &faker().name().prefix().unwrap());
}

#[test]
fn test_suffix() {
    assert_matches(r"\w+\.?", &faker().name().suffix().unwrap());
}

#[test]
fn test_title() {
    assert_matches(r"(\w+\.?( )?){3}", &faker().name().title().unwrap());
}

mod name_in_greek {
    use super::common::assert_matches;
    use datafaker::{Faker, Locale};

    fn faker() -> Faker {
        Faker::with_locale(Locale::new("el", "GR"))
    }

    #[test]
    fn test_name() {
        for _ in 0..10 {
            assert_matches(r"(\p{L}+\.?( )?){2,3}", &faker().name().full_name().unwrap());
        }
    }

    #[test]
    fn test_full_name() {
        test_name();
    }

    #[test]
    fn test_first_name() {
        assert_matches(r"\p{L}+", &faker().name().first_name().unwrap());
    }

    #[test]
    fn test_female_first_name() {
        for _ in 0..10 {
            assert_matches(r"\p{L}+", &faker().name().female_first_name().unwrap());
        }
    }

    #[test]
    fn test_male_first_name() {
        for _ in 0..10 {
            assert_matches(r"\p{L}+", &faker().name().male_first_name().unwrap());
        }
    }

    #[test]
    fn test_prefix() {
        assert_matches(r"\p{L}+\.?", &faker().name().prefix().unwrap());
    }
}
