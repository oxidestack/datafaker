//! `InternetTest` (Datafaker 2.7.0, `providers/base/InternetTest.java`). Unseeded fakers as upstream; repeated
//! tests run their body ten times. `EmailValidator` (commons-validator) is replaced by `common::is_valid_email`.

mod common;

use common::{assert_matches, is_valid_email};
use datafaker::providers::{BotUserAgent, PortRange, UserAgent};
use datafaker::{Faker, Locale};

const IPV6_HOST_ADDRESS: &str = "[0-9a-fA-F]{1,4}(:([0-9a-fA-F]{1,4})){1,7}";

fn faker() -> Faker {
    Faker::new()
}

fn count(s: &str, c: char) -> usize {
    s.chars().filter(|x| *x == c).count()
}

#[test]
fn test_username() {
    for _ in 0..10 {
        assert_matches(r"^(\w+)\.(\w+)$", &faker().internet().username().unwrap());
    }
}

#[test]
fn email_subject() {
    for _ in 0..10 {
        assert!(!faker().internet().email_subject().unwrap().trim().is_empty());
    }
}

/// Adapted: upstream mocks `Name` with Mockito; `Credentials.username`'s joining is checked through a locale whose
/// names contain spaces and apostrophes instead (`username_parts`), with the Turkish lower-casing of the case.
#[test]
fn test_username_with_spaces() {
    let tr = Locale::language_only("tr");
    assert_eq!(datafaker::providers::Credentials::username_from("Jin Quan", "D'Artagnan", &tr), "jinquan.dartagnan");
    assert_eq!(datafaker::providers::Credentials::username_from("Ivan Ivanov", "Другой Язык", &tr), "ıvanıvanov.другойязык");
}

#[test]
fn test_email_address() {
    assert!(is_valid_email(&faker().internet().email_address().unwrap()));
}

#[test]
fn test_email_address_with_name_parameter() {
    let f = faker();
    let username = f.internet().username().unwrap();
    let email = f.internet().email_address_for(&username).unwrap();
    assert!(email.starts_with(&format!("{username}@")), "{email}");
    assert!(is_valid_email(&email));
    for (name, prefix) in [
        ("Hal", "hal@"),
        ("T-800", "t800@"),
        ("John McClane", "john.mcclane@"),
        ("Tom Marvolo Riddle", "tom.riddle@"),
        ("Dr. Emmett Brown", "emmett.brown@"),
        ("Dr. Henry Indiana Jones Jr.", "henry.jones@"),
        ("Jeanne d'Arc", "jeanne.darc@"),
        ("Jeanne d’Arc", "jeanne.darc@"),
    ] {
        let email = f.internet().email_address_for(name).unwrap();
        assert!(email.starts_with(prefix), "{name}: {email}");
        assert!(is_valid_email(&email), "{email}");
    }
}

/// Adapted: upstream mocks `fetchObject` to return an empty set and a list of integers for `name.prefix` and
/// `name.suffix`; both mean "no prefixes", which the plain call below exercises with the real lists.
#[test]
fn test_email_address_type_safety() {
    let email = faker().internet().email_address_for("John McClane").unwrap();
    assert!(email.starts_with("john.mcclane@"));
    assert!(is_valid_email(&email));
}

#[test]
fn test_safe_email_address() {
    let f = faker();
    let mut emails = Vec::new();
    for _ in 0..100 {
        let email = f.internet().safe_email_address().unwrap();
        assert!(is_valid_email(&email), "{email}");
        emails.push(email);
    }
    let safe_domain = f.resolve("internet.safe_email").unwrap();
    assert!(emails.iter().any(|e| e.ends_with(&format!("@{safe_domain}"))));
}

#[test]
fn test_safe_email_address_with_name_parameter() {
    let f = faker();
    let mut emails = Vec::new();
    for _ in 0..100 {
        let email = f.internet().safe_email_address_for("John McClane").unwrap();
        assert!(email.starts_with("john.mcclane@"));
        assert!(is_valid_email(&email));
        emails.push(email);
    }
    let safe_domain = f.resolve("internet.safe_email").unwrap();
    assert!(emails.iter().any(|e| e.ends_with(&format!("@{safe_domain}"))));
}

#[test]
fn test_email_address_does_not_include_accents_in_the_local_part() {
    assert!(faker().internet().email_address_for("áéíóú").unwrap().starts_with("aeiou@"));
}

#[test]
fn test_safe_email_address_does_not_include_accents_in_the_local_part() {
    assert!(faker().internet().safe_email_address_for("áéíóú").unwrap().starts_with("aeiou@"));
}

#[test]
fn test_webdomain() {
    assert_matches(r"www\.[\w-]+\.\w+", &faker().internet().webdomain().unwrap());
}

/// Adapted: upstream parses with `java.net.URL`; here the shape `scheme://host[:port]/path…` is checked.
#[test]
fn test_url() {
    for _ in 0..10 {
        let url = faker().internet().url().unwrap();
        assert_matches(r"https?://www\.[\w.-]+(:\d+)?/[^\s]*", &url);
    }
}

#[test]
fn test_image() {
    assert_matches(r"^https://picsum\.photos/\d{1,4}/\d{1,4}$", &faker().internet().image().unwrap());
}

#[test]
fn test_domain_name() {
    assert_matches(r"[a-z]+\.\w{2,4}", &faker().internet().domain_name().unwrap());
}

#[test]
fn test_domain_word() {
    assert_matches("[a-z]+", &faker().internet().domain_word().unwrap());
}

#[test]
fn test_domain_suffix() {
    assert_matches(r"\w{2,4}", &faker().internet().domain_suffix().unwrap());
}

#[test]
fn test_image_with_explicit_params() {
    assert_matches(r"^https://picsum\.photos/seed/lorem/800/600$", &faker().internet().image_with_seed(800, 600, "lorem"));
}

#[test]
fn test_http_method() {
    assert!(!faker().internet().http_method().unwrap().is_empty());
}

#[test]
fn test_password() {
    assert_matches(r"[a-z\d]{8,16}", &faker().internet().password().unwrap());
}

#[test]
fn test_password_with_fixed_length() {
    assert_eq!(faker().internet().password_with(32, 32, true, true, true).unwrap().len(), 32);
}

#[test]
fn test_password_include_digit() {
    let f = faker();
    assert_matches(r"[a-z\d]{8,16}", &f.internet().password().unwrap());
    assert_matches("[a-z]{8,16}", &f.internet().password_with(8, 16, false, false, false).unwrap());
}

#[test]
fn test_password_min_length_max_length() {
    assert_matches(r"[a-z\d]{10,25}", &faker().internet().password_with(10, 25, false, false, true).unwrap());
}

#[test]
fn test_password_min_length_max_length_include_upper_case() {
    let f = faker();
    assert_matches(r"[a-z\d]{1,2}", &f.internet().password_with(1, 2, false, false, true).unwrap());
    assert_matches(r"[a-zA-Z\d]{10,25}", &f.internet().password_with(10, 25, true, false, true).unwrap());
}

#[test]
fn test_password_min_length_max_length_include_upper_case_include_special() {
    let f = faker();
    assert_matches(r"[a-z\d]{10,25}", &f.internet().password_with(10, 25, false, false, true).unwrap());
    assert_matches(r"[a-z\d!@#$%^&*]{10,25}", &f.internet().password_with(10, 25, false, true, true).unwrap());
    assert_matches(r"[a-zA-Z\d!@#$%^&*]{10,25}", &f.internet().password_with(10, 25, true, true, true).unwrap());
}

#[test]
fn test_port() {
    for _ in 0..10 {
        assert!((0..=65535).contains(&faker().internet().port()));
    }
}

#[test]
fn port_within_given_bounds() {
    for _ in 0..10 {
        let f = faker();
        assert!((0..=1).contains(&f.internet().port_between(0, 1).unwrap()));
        assert!((100..=200).contains(&f.internet().port_between(100, 200).unwrap()));
        assert_eq!(f.internet().port_between(1000, 1000).unwrap(), 1000);
        assert_eq!(f.internet().port_between(65535, 65535).unwrap(), 65535);
    }
}

#[test]
fn port_within_given_range() {
    for _ in 0..10 {
        let f = faker();
        assert!((0..=1023).contains(&f.internet().port_in(PortRange::WellKnownPorts).unwrap()));
        assert!((1024..=49151).contains(&f.internet().port_in(PortRange::RegisteredPorts).unwrap()));
        assert!((49152..=65535).contains(&f.internet().port_in(PortRange::DynamicPrivatePorts).unwrap()));
    }
}

#[test]
fn port_within_given_range_validation() {
    let f = faker();
    let e = f.internet().port_between(-1, 100).unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", "Port number -1 cannot be less than 0"));
    let e = f.internet().port_between(100, 99).unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", "Min (100) > Max (99)"));
    let e = f.internet().port_between(65535, 65536).unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", "Port number 65536 cannot be greater than 65535"));
}

#[test]
fn should_generate_a_password_with_min_and_max_length() {
    let f = faker();
    let results: Vec<String> = (0..300).map(|_| f.internet().password_with(1, 10, false, false, true).unwrap()).collect();
    assert!(results.iter().any(|x| x.len() == 1));
    assert!(results.iter().any(|x| x.len() == 10));
}

#[test]
fn test_password_min_length_max_length_include_upper_case_include_special_include_digit() {
    let f = faker();
    assert_matches("[a-z]{10,25}", &f.internet().password_with(10, 25, false, false, false).unwrap());
    assert_matches(r"[a-z\d!@#$%^&*]{10,25}", &f.internet().password_with(10, 25, false, true, true).unwrap());
    assert_matches("[a-zA-Z!@#$%^&*]{10,25}", &f.internet().password_with(10, 25, true, true, false).unwrap());
    assert_matches(r"[a-zA-Z\d!@#$%^&*]{10,25}", &f.internet().password_with(10, 25, true, true, true).unwrap());
}

#[test]
fn test_mac_address() {
    let f = faker();
    assert_eq!(count(&f.internet().mac_address(), ':'), 5);
    assert_eq!(count(&f.internet().mac_address_with(""), ':'), 5);
    let m = f.internet().mac_address_with("fa:fa:fa");
    assert!(m.starts_with("fa:fa:fa") && count(&m, ':') == 5, "{m}");
    let m = f.internet().mac_address_with("01:02");
    assert!(m.starts_with("01:02") && count(&m, ':') == 5, "{m}");
    for _ in 0..1000 {
        assert_matches("[0-9a-fA-F]{2}(:([0-9a-fA-F]{1,4})){5}", &f.internet().mac_address());
    }
}

#[test]
fn test_ip_v4_address() {
    let f = faker();
    assert_eq!(count(&f.internet().ip_v4_address(), '.'), 3);
    for _ in 0..100 {
        for octet in f.internet().ip_v4_address().split('.') {
            assert!((0..=255).contains(&octet.parse::<i32>().unwrap()));
        }
    }
}

#[test]
fn test_ip_v4_cidr() {
    let f = faker();
    let c = f.internet().ip_v4_cidr();
    assert!(count(&c, '.') == 3 && count(&c, '/') == 1);
    for _ in 0..1000 {
        let bits: i32 = f.internet().ip_v4_cidr().split('/').nth(1).unwrap().parse().unwrap();
        assert!((1..=32).contains(&bits));
    }
}

#[test]
fn test_private_ip_v4_address() {
    let f = faker();
    let patterns = [r"^10\..+", r"^127\..+", r"^169\.254\..+", r"^192\.168\..+", r"^172\.(16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31)\..+"];
    for _ in 0..1000 {
        let addr = f.internet().private_ip_v4_address();
        assert!(patterns.iter().any(|p| common::matches(p, &addr)), "{addr}");
    }
}

#[test]
fn test_public_ip_v4_address() {
    let f = faker();
    let patterns = [r"^10\.", r"^127\.", r"^169\.254", r"^192\.168\.", r"^172\.(16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31)\."];
    for _ in 0..1000 {
        let addr = f.internet().public_ip_v4_address();
        // `doesNotMatch` is a whole-string match, which these prefixes never are.
        assert!(!patterns.iter().any(|p| common::matches(p, &addr)), "{addr}");
    }
}

#[test]
fn test_ip_v6() {
    let f = faker();
    assert_eq!(count(&f.internet().ip_v6_address(), ':'), 7);
    for _ in 0..1000 {
        assert_matches(IPV6_HOST_ADDRESS, &f.internet().ip_v6_address());
    }
}

#[test]
fn test_ip_v6_cidr() {
    let f = faker();
    let c = f.internet().ip_v6_cidr();
    assert!(count(&c, ':') == 7 && count(&c, '/') == 1);
    for _ in 0..1000 {
        let bits: i32 = f.internet().ip_v6_cidr().split('/').nth(1).unwrap().parse().unwrap();
        assert!((1..=128).contains(&bits));
    }
}

#[test]
fn test_slug_with_params() {
    for _ in 0..10 {
        let words = ["a".to_string(), "b".to_string()];
        assert_matches("[a-zA-Z]+-[a-zA-Z]+", &faker().internet().slug_with(Some(&words), Some("-")).unwrap());
    }
}

#[test]
fn test_slug() {
    for _ in 0..10 {
        assert_matches("[a-zA-Z]+_[a-zA-Z]+", &faker().internet().slug().unwrap());
    }
}

#[test]
fn test_uuidv3_constant_random_seed() {
    let f1 = Faker::with_seed(42);
    let f2 = Faker::with_seed(42);
    for _ in 0..3 {
        assert_eq!(f1.internet().uuidv3(), f2.internet().uuidv3());
    }
}

#[test]
fn test_uuidv3() {
    for _ in 0..10 {
        assert_matches("^[0-9a-f]{8}-[0-9a-f]{4}-3[0-9a-f]{3}-[0-9a-f]{4}-[0-9a-f]{12}$", &faker().internet().uuidv3());
    }
}

#[test]
fn test_uuid() {
    for _ in 0..10 {
        assert_matches("^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$", &faker().internet().uuid());
    }
}

#[test]
fn test_uuidv4() {
    for _ in 0..10 {
        assert_matches("^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[0-9a-f]{4}-[0-9a-f]{12}$", &faker().internet().uuidv4());
    }
}

#[test]
fn test_uuidv7() {
    for _ in 0..10 {
        assert_matches("^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[0-9a-f]{4}-[0-9a-f]{12}$", &faker().internet().uuidv7());
    }
}

#[test]
fn test_farsi_idns() {
    for _ in 0..10 {
        let f = Faker::with_locale(Locale::language_only("fa"));
        assert!(!f.internet().domain_name().unwrap().is_empty());
        assert!(!f.internet().email_address().unwrap().is_empty());
        assert!(!f.internet().safe_email_address().unwrap().is_empty());
        assert!(!f.internet().webdomain().unwrap().is_empty());
    }
}

#[test]
fn test_user_agent() {
    let f = faker();
    for agent in UserAgent::VALUES {
        assert!(!f.internet().user_agent_of(Some(agent)).unwrap().is_empty());
    }
    assert!(!f.internet().user_agent().unwrap().is_empty());
}

#[test]
fn test_bot_user_agent() {
    let f = faker();
    for agent in BotUserAgent::VALUES {
        assert!(!f.internet().bot_user_agent(Some(agent)).unwrap().is_empty());
    }
    assert!(!f.internet().bot_user_agent_any().unwrap().is_empty());
}

#[test]
fn test_slug_with_null() {
    assert!(!faker().internet().slug_with(None, Some("_")).unwrap().is_empty());
}
