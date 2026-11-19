//! `CredentialsTest` (Datafaker 2.7.0, `providers/base/CredentialsTest.java`). Unseeded fakers as upstream;
//! `@RepeatedTest(100)` bodies run 100 times.

mod common;

use common::assert_matches;
use datafaker::{Faker, Locale};

fn faker() -> Faker {
    Faker::new()
}

#[test]
fn test_username() {
    let f = faker();
    for _ in 0..100 {
        assert_matches(r"^(\w+)\.(\w+)$", &f.credentials().username().unwrap());
    }
}

/// Adapted: upstream mocks `Name` with Mockito; `Credentials::username_from` is the joining `username()` applies.
#[test]
fn test_username_with_spaces() {
    let tr = Locale::language_only("TR");
    assert_eq!(datafaker::providers::Credentials::username_from("Jin Quan", "D'Artagnan", &tr), "jinquan.dartagnan");
    assert_eq!(datafaker::providers::Credentials::username_from("Ivan Ivanov", "Другой Язык", &tr), "ıvanıvanov.другойязык");
}

fn sized(s: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&s.chars().count())
}

#[test]
fn test_password() {
    let f = faker();
    for _ in 0..100 {
        let p = f.credentials().password().unwrap();
        assert!(sized(&p, 8, 16));
        assert_matches("^[a-z0-9]+$", &p);
    }
}

#[test]
fn test_password_without_digits() {
    let f = faker();
    for _ in 0..100 {
        let p = f.credentials().password_with(8, 16, false, false, false).unwrap();
        assert!(sized(&p, 8, 16));
        assert_matches("^[a-z]+$", &p);
    }
}

#[test]
fn test_password_with_specific_size() {
    let f = faker();
    for _ in 0..100 {
        let p = f.credentials().password_with(3, 7, false, false, true).unwrap();
        assert!(sized(&p, 3, 7));
        assert_matches("^[a-z0-9]+$", &p);
    }
}

#[test]
fn test_password_with_specific_size_and_uppercase() {
    let f = faker();
    for _ in 0..100 {
        let p = f.credentials().password_with(8, 16, true, false, true).unwrap();
        assert!(sized(&p, 8, 16));
        assert_matches("^[a-zA-Z0-9]+$", &p);
    }
}

#[test]
fn test_password_1000() {
    let f = faker();
    for _ in 0..1000 {
        let p = f.credentials().password_with(8, 16, true, true, true).unwrap();
        assert!(p.chars().any(|c| c.is_ascii_digit()), "{p}");
        assert!(p.chars().any(|c| !c.is_ascii_alphanumeric()), "{p}");
    }
}

#[test]
fn password_special() {
    let f = faker();
    for _ in 0..10 {
        let p = f.credentials().password_with(8, 16, true, true, true).unwrap();
        assert!(p.chars().any(|c| !c.is_ascii_alphanumeric()), "{p}");
    }
}

#[test]
fn password_mix() {
    let f = faker();
    for _ in 0..10 {
        let p = f.credentials().password_with(8, 16, true, true, true).unwrap();
        assert!(p.chars().any(|c| !c.is_ascii_alphanumeric()) && p.chars().any(|c| c.is_ascii_digit()), "{p}");
    }
}

#[test]
fn weak_password() {
    let f = faker();
    let list: Vec<String> = match f.fake_values_service().fetch_object("credentials.weak_password") {
        Some(datafaker::yaml::Value::List(l)) => l.iter().filter_map(|v| v.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    };
    assert!(list.contains(&f.credentials().weak_password().unwrap()));
}

#[test]
fn test_user_id() {
    let f = faker();
    for _ in 0..100 {
        assert_matches("^([A-Z]{1})?[0-9]{5,6}$", &f.credentials().user_id().unwrap().unwrap());
    }
}

/// `userId(null)` has no Rust counterpart (the argument is a `&str`).
#[test]
fn user_id_with_parameter() {
    let f = faker();
    assert_eq!(f.credentials().user_id_matching("*").unwrap(), None);
    assert_eq!(f.credentials().user_id_matching("").unwrap().as_deref(), Some(""));
}
