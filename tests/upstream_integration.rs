//! `FakerIntegrationTest`, `FakerRepeatabilityIntegrationTest`, `Issue194SlashFormatRegexTest`,
//! `MostSpecificLocaleTest` and `UkLocalDirectivesTest` (Datafaker 2.7.0, `integration/`). Java walks the faker's
//! providers and their methods by reflection; here the dispatch tables (`ProviderKind::ALL`, each provider's
//! public methods) are that list. A method whose Java return type is `String` is one that returns a string here.

mod common;

use std::collections::HashMap;

use common::assert_matches;
use datafaker::dispatch::{JValue, Target};
use datafaker::java::{Instant, time::with_fixed_clock};
use datafaker::providers::ProviderKind;
use datafaker::{Faker, Locale};

// FakerIntegrationTest

/// `exceptions`: methods allowed to return an empty string in a locale.
fn excepted(locale: &Locale, provider: &str, method: &str) -> bool {
    let address_city = ["cityPrefix", "citySuffix"];
    let uk = ["cityPrefix", "citySuffix", "stateAbbr", "streetSuffix"];
    let skipped: &[(&str, &str, &str, &[&str])] = &[
        ("it", "", "Name", &["suffix"]),
        ("es", "MX", "Address", &address_city),
        ("pt", "", "Address", &address_city),
        ("uk", "", "Address", &uk),
        ("uk", "UA", "Address", &uk),
        ("id", "", "App", &["author"]),
        ("id", "ID", "App", &["author"]),
        ("pt", "BR", "Address", &address_city),
    ];
    // `new Locale("pt", "Br", "x2")` has a variant, so it equals no other locale and its own entry only matches
    // itself; Datafaker reads it as pt_BR. Its exception list is pt_BR's.
    skipped.iter().any(|(language, country, class, methods)| {
        locale.language() == *language && locale.country() == *country && provider == *class && methods.contains(&method)
    })
}

/// `dataParameters`: the locales (a `null` one is the default), the language of every top-level locale file.
fn data_parameters() -> Vec<Option<Locale>> {
    let mut locales = vec![
        Some(Locale::english()),
        Some(Locale::new("en", "US")),
        Some(Locale::new("en", "GB")),
        Some(Locale::new("pt", "BR")),
        Some(Locale::new("pt", "br")),
        Some(Locale::new("Pt", "br")),
        Some(Locale::new("pt", "Br")),
        None,
        None,
    ];
    for (path, _) in datafaker::resources::FILES {
        if !path.contains('/') && path.ends_with(".yml") && !path.starts_with('_') {
            locales.push(Some(Locale::language_only(&path[..path.find('.').unwrap()].replace('-', "_"))));
        }
    }
    locales
}

fn init(locale: Option<Locale>) -> Faker {
    match locale {
        Some(locale) => Faker::with_locale(locale),
        None => Faker::new(),
    }
}

#[test]
fn test_all_faker_methods_that_return_strings() {
    for locale in data_parameters() {
        let faker = init(locale.clone());
        let context_locale = faker.locale();
        for kind in ProviderKind::ALL {
            for m in Target::Provider(*kind).methods().iter().filter(|m| m.params.is_empty()) {
                if excepted(&context_locale, kind.simple_name(), m.name) {
                    continue;
                }
                let value = (m.invoke)(&faker, &[]).unwrap_or_else(|e| {
                    panic!("Test for method {}.{} was failed for locale {:?}: {} {}", kind.simple_name(), m.name, locale, e.simple_class(), e.message())
                });
                if let JValue::Str(s) = value {
                    assert!(!s.is_empty(), "For method {}.{}(), value is '' in {:?}", kind.simple_name(), m.name, locale);
                }
            }
        }
    }
}

#[test]
fn test_exceptions_not_covered_in_above_test() {
    for locale in data_parameters() {
        let faker = init(locale);
        faker.bothify("####???");
        faker.letterify("????");
        faker.numerify("####");
        faker.lorem().paragraph_of(1).unwrap();
        faker.lorem().paragraphs(1).unwrap();
        faker.lorem().sentence_of(1).unwrap();
        faker.lorem().sentences(1).unwrap();
        faker.address().street_address().unwrap();
        faker.lorem().words(3).unwrap();
        faker.lorem().words(1).unwrap();
    }
}

// FakerRepeatabilityIntegrationTest

/// `buildReport`: every public no-argument method of every provider, by `provider.method`. Upstream skips the
/// methods returning `Instant` (they read the clock, which moves between the two reports); here the clock is
/// fixed while both reports are built instead, so those run too.
fn build_report(faker: &Faker) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut kinds: Vec<&ProviderKind> = ProviderKind::ALL.iter().collect();
    kinds.sort_by_key(|k| k.simple_name());
    for kind in kinds {
        let mut methods: Vec<_> = Target::Provider(*kind).methods().iter().filter(|m| m.params.is_empty()).collect();
        methods.sort_by_key(|m| m.name);
        for m in methods {
            let value = (m.invoke)(faker, &[]).map(|v| v.to_java_string(faker)).unwrap_or_else(|e| format!("ERR {}", e.message()));
            result.insert(format!("{}.{}", kind.simple_name(), m.name), value);
        }
    }
    result
}

#[test]
fn should_create_repeatable_values() {
    with_fixed_clock(Instant::now(), || {
        let report1 = build_report(&Faker::with_seed(0));
        let report2 = build_report(&Faker::with_seed(0));
        assert_eq!(report1, report2);
    });
}

#[test]
fn should_create_unique_values() {
    let report1 = build_report(&Faker::new());
    let report2 = build_report(&Faker::new());
    let differing = report1.iter().filter(|(k, v)| report2.get(*k) != Some(*v)).count();
    let common = report1.len() - differing;
    assert!(differing > common, "{differing} differing, {common} in common");
}

// Issue194SlashFormatRegexTest

#[test]
fn en_gb_zip_code_returns_proper_regexified_value() {
    let postal_code = Faker::with_locale(Locale::new("en", "GB")).address().zip_code().unwrap();
    assert_matches("[A-PR-UWYZ]([A-HK-Y][0-9][ABEHMNPRVWXY0-9]?|[0-9][ABCDEFGHJKPSTUW0-9]?) [0-9][ABD-HJLNP-UW-Z]{2}", &postal_code);
}

#[test]
fn en_ca_zip_code_returns_proper_regexified_value() {
    let postal_code = Faker::with_locale(Locale::new("en", "CA")).address().zip_code().unwrap();
    assert_matches("[A-CEJ-NPR-TVXY][0-9][A-CEJ-NPR-TV-Z] ?[0-9][A-CEJ-NPR-TV-Z][0-9]", &postal_code);
}

#[test]
fn vi_zip_code_returns_proper_regexified_value() {
    let postal_code = Faker::with_locale(Locale::language_only("vi")).address().zip_code().unwrap();
    assert_matches("[A-PR-UWYZ0-9][A-HK-Y0-9][AEHMNPRTVXY0-9]?[ABEHMNPRVWXY0-9]? {1,2}[0-9][ABD-HJLN-UW-Z]{2}", &postal_code);
}

// MostSpecificLocaleTest

#[test]
fn resolves_the_most_specific_locale() {
    let en = Faker::with_locale(Locale::language_only("en"));
    let en_us = Faker::with_locale(Locale::new("en", "US"));
    let countries = |f: &Faker| match f.fake_values_service().fetch_object("address.default_country") {
        Some(datafaker::yaml::Value::List(l)) => l.clone(),
        other => panic!("{other:?}"),
    };
    let (en_default, en_us_default) = (countries(&en), countries(&en_us));
    assert_eq!(en_default.len(), 1);
    assert_eq!(en_us_default.len(), 3);
    assert_ne!(en_default, en_us_default, "the default country for en is not en_US");
}

// UkLocalDirectivesTest

#[test]
fn resolves_directives_only_in_yml_file() {
    let street_name = Faker::with_locale(Locale::language_only("uk")).address().street_name().unwrap();
    let prefixes = ["пр.", "проспект", "пров.", "провулок", "вул.", "вулиця", "пл.", "площа"];
    assert!(prefixes.iter().any(|p| street_name.starts_with(p)), "the street name should start with a fem or masc prefix, but received: {street_name}");
}
