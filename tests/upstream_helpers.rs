//! `JavaNamesTest`, `WordUtilsTest`, `FakerIDNTest` (`internal/helper/`) and `FakerContextTest` (`service/`) of
//! Datafaker 2.7.0. Java's `null` arguments have no counterpart for `&str` parameters.

use datafaker::{Locale, capitalize, to_java_names};

#[test]
fn single_word() {
    assert_eq!(to_java_names("variety", false), "Variety");
    assert_eq!(to_java_names("variety", true), "variety");
}

#[test]
fn empty_string() {
    assert_eq!(to_java_names("", false), "");
    assert_eq!(to_java_names("", true), "");
}

#[test]
fn word_with_digit() {
    assert_eq!(to_java_names("dota2", false), "Dota2");
    assert_eq!(to_java_names("dota2", true), "dota2");
}

#[test]
fn word_starting_with_digit() {
    assert_eq!(to_java_names("1love", false), "1love");
    assert_eq!(to_java_names("1love", true), "1love");
}

#[test]
fn single_character() {
    assert_eq!(to_java_names("x", false), "X");
    assert_eq!(to_java_names("x", true), "x");
}

#[test]
fn single_character_underscore() {
    assert_eq!(to_java_names("_", false), "");
    assert_eq!(to_java_names("_", true), "");
}

#[test]
fn only_underscores() {
    assert_eq!(to_java_names("__", false), "");
    assert_eq!(to_java_names("___", true), "");
}

#[test]
fn multiple_words_separated_by_underscores() {
    assert_eq!(to_java_names("big_bang_theory", false), "BigBangTheory");
    assert_eq!(to_java_names("big_bang_theory", true), "bigBangTheory");
}

#[test]
fn multiple_underscores_in_row() {
    assert_eq!(to_java_names("big__bang___theory", false), "BigBangTheory");
    assert_eq!(to_java_names("big__bang___theory", true), "bigBangTheory");
}

#[test]
fn upper_first_character_without_underscores() {
    assert_eq!(to_java_names("A2013", false), "A2013");
    assert_eq!(to_java_names("A2013", true), "a2013");
}

#[test]
fn upper_first_character_with_underscores() {
    assert_eq!(to_java_names("IATA_airline", false), "IATAAirline");
    assert_eq!(to_java_names("IATA_airline", true), "iATAAirline");
}

#[test]
fn ending_with_underscore() {
    assert_eq!(to_java_names("name_", false), "Name");
    assert_eq!(to_java_names("name_", true), "name");
}

#[test]
fn capitalize_word() {
    for (input, want) in [("w", "W"), ("W", "W"), ("word", "Word"), ("Word", "Word"), ("WORD", "WORD"), ("wORD", "WORD")] {
        assert_eq!(capitalize(input), want);
    }
}

#[test]
fn capitalize_null() {
    assert_eq!(capitalize(""), "");
}

#[test]
fn to_ascii_no_error() {
    assert_eq!(datafaker::idn::faker_to_ascii("hello").unwrap(), "hello");
}

/// `Ⱥ` (http://Ⱥbby.com) is unassigned in Unicode 3.2, so `IDN.toASCII` refuses it, alone too.
#[test]
fn to_ascii_result_is_empty_exception() {
    assert_eq!(datafaker::idn::faker_to_ascii("Ⱥ").unwrap_err().simple_class(), "RuntimeException");
}

fn chain(locale: Locale) -> Vec<String> {
    locale.chain().iter().map(ToString::to_string).collect()
}

#[test]
fn locale_chain() {
    assert_eq!(chain(Locale::english()), ["en"]);
    assert_eq!(chain(Locale::new("et", "EE")), ["et_EE", "et", "_EE", "en"]);
    assert_eq!(chain(Locale::language_only("hu_HU")), ["hu_HU", "hu", "_HU", "en"]);
}

/// `Locality.allSupportedLocales()` is not ported yet (T1); the two-letter languages are read from the embedded
/// resource names instead, which is where `Locality` reads them from.
#[test]
fn appends_default_country_for_given_language() {
    let defaults = [
        ("be", "BY"),
        ("cs", "CZ"),
        ("da", "DK"),
        ("el", "GR"),
        ("et", "EE"),
        ("he", "IL"),
        ("hy", "AM"),
        ("ja", "JP"),
        ("ka", "GE"),
        ("ko", "KR"),
        ("nb", "NO"),
        ("sq", "AL"),
        ("sv", "SE"),
        ("ta", "IN"),
        ("uk", "UA"),
        ("vi", "VN"),
        ("zh", "CN"),
    ];
    let languages: Vec<&str> = datafaker::resources::FILES
        .iter()
        .filter_map(|(p, _)| p.strip_suffix(".yml"))
        .filter(|p| p.len() == 2 && *p != "en")
        .collect();
    assert!(languages.len() > 30);
    for language in languages {
        let country = defaults.iter().find(|(l, _)| *l == language).map_or(language.to_uppercase(), |(_, c)| c.to_string());
        assert_eq!(chain(Locale::language_only(language)), [language.to_string(), format!("_{country}"), "en".to_string()]);
    }
}
