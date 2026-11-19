//! Tests of Datafaker 2.7.0's base provider classes (`providers/base/`) that `scripts/gen-providers.py` does not
//! read: the nested classes for other locales (`AncientTest.AncientInGreekTest` and the like), tests on other
//! locales' fakers, `ProviderListLocaleTest` subclasses, and the tests of the hand-written providers of tranche T1a.
//! Unseeded fakers as upstream; repeated tests run their body as often.

mod common;

use common::{assert_matches, provider_list};
use datafaker::{Faker, Locale};

fn greek() -> Faker {
    Faker::with_locale(Locale::new("el", "GR"))
}

/// `AncientTest.AncientInGreekTest`.
#[test]
fn ancient_in_greek_provider_list() {
    let f = greek();
    let ancient = f.ancient();
    provider_list(&f, "ancient.god", &ancient.god().unwrap(), "");
    provider_list(&f, "ancient.primordial", &ancient.primordial().unwrap(), "");
    provider_list(&f, "ancient.titan", &ancient.titan().unwrap(), "");
    provider_list(&f, "ancient.hero", &ancient.hero().unwrap(), "");
}

/// `GreekPhilosopherTest.GreekPhilosopherInGreekTest`.
#[test]
fn greek_philosopher_in_greek_provider_list() {
    let f = greek();
    provider_list(&f, "greek_philosophers.names", &f.greek_philosopher().name().unwrap(), "");
    provider_list(&f, "greek_philosophers.quotes", &f.greek_philosopher().quote().unwrap(), "");
}

/// `JobTest.JobInGreekTest`: its own regex for key skills.
#[test]
fn job_in_greek_provider_list() {
    let f = greek();
    let job = f.job();
    provider_list(&f, "job.field", &job.field().unwrap(), "");
    provider_list(&f, "job.seniority", &job.seniority().unwrap(), "");
    provider_list(&f, "job.position", &job.position().unwrap(), "");
    provider_list(&f, "job.key_skills", &job.key_skills().unwrap(), r"(?:\p{L}+ ?){1,4}");
}

/// `MeasurementTest.MeasurementInGreekTest`.
#[test]
fn measurement_in_greek_provider_list() {
    let f = greek();
    let m = f.measurement();
    provider_list(&f, "measurement.height", &m.height().unwrap(), "");
    provider_list(&f, "measurement.length", &m.length().unwrap(), "");
    provider_list(&f, "measurement.volume", &m.volume().unwrap(), "");
    provider_list(&f, "measurement.weight", &m.weight().unwrap(), "");
    provider_list(&f, "measurement.metric_length", &m.metric_length().unwrap(), "");
    provider_list(&f, "measurement.metric_height", &m.metric_height().unwrap(), "");
    provider_list(&f, "measurement.metric_volume", &m.metric_volume().unwrap(), "");
    provider_list(&f, "measurement.metric_weight", &m.metric_weight().unwrap(), "");
}

/// `HololiveTest.talent_jaLocale`.
#[test]
fn hololive_talent_ja_locale() {
    let ja = Faker::with_locale(Locale::language_only("ja"));
    assert_matches(r"^AZKi|[぀-ヿ一-龯]+$", &ja.hololive().talent().unwrap());
}

/// `TeamTest.testStateWithZaLocale`.
#[test]
fn team_state_with_za_locale() {
    let za = Faker::with_locale(Locale::new("en", "ZA"));
    assert!(!za.team().state().unwrap().is_empty());
}

/// `UniversityLocaleTest`: `university.name` on the Ukrainian, Uzbek and New Zealand fakers of
/// `ProviderListLocaleTest`.
#[test]
fn university_locale_provider_list() {
    for locale in [Locale::new("uk", "UA"), Locale::new("uz", "UZ"), Locale::new("en", "NZ")] {
        let f = Faker::with_locale(locale);
        provider_list(&f, "university.name", &f.university().name().unwrap(), ".{15,120}");
    }
}

/// `MbtiTest`: each field non-empty, three of them matching a regex.
#[test]
fn mbti() {
    let mbti = Faker::new();
    let mbti = mbti.mbti();
    assert!(!mbti.r#type().unwrap().is_empty());
    assert_matches(r"[A-Za-z,\-.() ]+", &mbti.r#type().unwrap());
    assert!(!mbti.name().unwrap().is_empty());
    assert_matches(r"[A-Za-z,\-.();:'$ ]+", &mbti.name().unwrap());
    assert!(!mbti.characteristic().unwrap().is_empty());
    assert!(!mbti.personage().unwrap().is_empty());
    assert_matches(r"[A-Za-z,\-.()';:$ ]+", &mbti.personage().unwrap());
    assert!(!mbti.merit().unwrap().is_empty());
    assert!(!mbti.weakness().unwrap().is_empty());
}

/// `RelationshipTest.anyTest`. Its two Mockito tests (a `FakeValuesService` that throws) have no Rust form: the
/// faker's service cannot be replaced.
#[test]
fn relationship_any() {
    let f = Faker::new();
    for _ in 0..10 {
        assert!(!f.relationships().any().unwrap().is_empty());
    }
}

/// `RestaurantTest.namePrefix` and `name`: bothified with upper-case letters.
#[test]
fn restaurant_name_prefix_and_name() {
    let f = Faker::new();
    for _ in 0..10 {
        for value in [f.restaurant().name_prefix().unwrap(), f.restaurant().name().unwrap()] {
            assert!(!value.is_empty() && !value.contains('#') && !value.contains('?'), "{value}");
            assert_matches("[A-Z0-9].*", &value);
        }
    }
}

/// `EmergencyTest.shouldReturnDryRunFlag` and `shouldReturnEmergencyCase`.
#[test]
fn emergency_dry_run_and_case() {
    let f = Faker::new();
    let _: bool = f.emergency().dry_run();
    let case = f.emergency().emergency_case().unwrap();
    assert!(!case.nature.is_empty() && !case.location.is_empty() && !case.instruction.is_empty());
}

/// `FinancialTermsTest.category`, for every `Category` (`@EnumSource`).
#[test]
fn financial_terms_category() {
    let f = Faker::new();
    for category in datafaker::providers::FinancialTermsCategory::VALUES {
        assert!(!f.financial_terms().noun_of(category).unwrap().trim().is_empty());
        assert!(!f.financial_terms().verb_of(category).unwrap().trim().is_empty());
        assert!(!f.financial_terms().adjective_of(category).unwrap().trim().is_empty());
    }
}

/// `NationTest.flag`: at least four UTF-16 units, starting with U+D83C.
#[test]
fn nation_flag() {
    let flag: Vec<u16> = Faker::new().nation().flag().unwrap().encode_utf16().collect();
    assert!(flag.len() >= 4);
    assert_eq!(flag[0], 0xD83C);
}

const CODE_PATTERN: &str = r"\d{3}/\d{2,3}R\d{2}\.?\d?";

/// `TireTest.testDefaultPrefixedCode`.
#[test]
fn tire_default_prefixed_code() {
    let tire = Faker::new();
    assert_matches(&format!("P{CODE_PATTERN}"), &tire.tire().code_with_default_prefix(true).unwrap());
    assert_matches(CODE_PATTERN, &tire.tire().code_with_default_prefix(false).unwrap());
}

/// `TireTest.testMiscPrefixedCode`.
#[test]
fn tire_misc_prefixed_code() {
    let prefix = "misc";
    assert_matches(&format!("{prefix}{CODE_PATTERN}"), &Faker::new().tire().code_with_prefix(prefix).unwrap());
}

/// `WeatherTest.temperatureCelsiusInRange` and `temperatureFahrenheitInRange`.
#[test]
fn weather_temperature_in_range() {
    let f = Faker::new();
    for _ in 1..100 {
        assert_matches("-?[0-5]°C", &f.weather().temperature_celsius_between(-5, 5).unwrap());
        assert_matches("-?[0-5]°F", &f.weather().temperature_fahrenheit_between(-5, 5).unwrap());
    }
}
