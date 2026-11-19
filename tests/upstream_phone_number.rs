//! `PhoneNumberTest` and `PhoneNumberValidityFinderTest` (Datafaker 2.7.0, `providers/base/`), all 23 tests, on
//! the `libphonenumber` crate (libphonenumber 9.0.41). Unseeded fakers as upstream; repeated tests run their
//! repetitions in a loop, parameterized ones each argument.

mod common;

use common::assert_matches;
use datafaker::{Faker, Locale};
use libphonenumber::{NumberParseException, PhoneNumber, PhoneNumberUtil};

/// Number of phone numbers to generate during a test.
const COUNT: usize = 100;

fn util() -> &'static PhoneNumberUtil {
    PhoneNumberUtil::get_instance()
}

/// `parse(generatedNumber, countryCode)`: drops one leading `0` except for Italy and Hungary.
fn parse(generated_number: &str, country_code: &str) -> Result<PhoneNumber, NumberParseException> {
    let normalized = if country_code == "IT" || country_code == "HU" {
        generated_number
    } else if generated_number.len() > 1 && generated_number.starts_with('0') {
        &generated_number[1..]
    } else {
        generated_number
    };
    util().parse(normalized, Some(country_code))
}

fn no_spaces(phone: &str) -> String {
    phone.chars().filter(|c| !c.is_whitespace()).collect()
}

/// `PhoneNumberTest#testCellPhone_enUS`.
#[test]
fn test_cell_phone_en_us() {
    let f = Faker::with_locale(Locale::new("en", "US"));
    assert_matches(r"\(?\d+\)?([- .]\d+){1,3}", &f.phone_number().cell_phone().unwrap());
}

/// `PhoneNumberTest#testPhone_esMx` (repeated 100 times).
#[test]
fn test_phone_es_mx() {
    for _ in 0..COUNT {
        let f = Faker::with_locale(Locale::new("es", "MX"));
        let phone_number = f.phone_number();
        assert_matches(r"(044 )?\(?\d+\)?([- .]\d+){1,3}", &phone_number.cell_phone().unwrap());
        assert_matches(r"\(?\d+\)?([- .]\d+){1,3}", &phone_number.phone_number().unwrap());
    }
}

/// `PhoneNumberTest#testPhone_CA`, for `Locale.CANADA` and `ca_CA`, with area code 273 that libphonenumber 9.0.41
/// added to Canada's mobile numbers (upstream's list gained it after 2.7.0, commit d158a5f2).
#[test]
fn test_phone_ca() {
    let area_code = "263|354|382|403|587|780|825|236|250|368|428|604|672|778|204|431|506|\
        709|782|902|226|249|289|343|365|416|437|519|548|613|647|705|807|905|367|\
        418|438|450|468|474|514|579|581|584|683|742|753|819|873|306|639|867|879|942|257|273";
    let canadian_phone = format!(r"((\+1)?(\(?({area_code})\)?)|({area_code}))[- .]\d{{3}}[- .]\d{{4}}");
    for locale in [Locale::new("en", "CA"), Locale::new("ca", "CA")] {
        let f = Faker::with_locale(locale);
        let phone_number = f.phone_number();
        for _ in 0..COUNT {
            assert_matches(&canadian_phone, &phone_number.cell_phone().unwrap());
        }
    }
}

/// `PhoneNumberTest.locales()`.
const LOCALES: &[(&str, &str)] = &[
    ("en", "US"),
    ("en", "GB"),
    ("en", "AU"),
    ("en", "CA"),
    ("en", "MS"),
    ("en", "NG"),
    ("en", "NZ"),
    ("et", "EE"),
    ("bg", "BG"),
    ("by", "BY"),
    ("ca", "CA"),
    ("cs", "CZ"),
    ("de", "DE"),
    ("de", "AT"),
    ("de", "CH"),
    ("en", "IN"),
    ("en", "NP"),
    ("en", "PK"),
    ("hu", "HU"),
    ("fi", "FI"),
    ("ko", "KR"),
    ("ja", "JP"),
    ("lv", "LV"),
    ("mk", "MK"),
    ("ca", "IT"),
    ("nl", "NL"),
    ("pl", "PL"),
    ("pt", "PT"),
    ("ro", "MD"),
    ("sq", "AL"),
    ("zh", "CN"),
    ("zh", "TW"),
    ("uk", "UA"),
    ("tr", "TR"),
    ("en", "SG"),
    ("en", "PH"),
    ("en", "UG"),
    ("en", "ZA"),
    ("sv", "SE"),
    ("th", "TH"),
    ("sk", "SK"),
    ("ru", "RU"),
    ("pt", "BR"),
    ("es", "AR"),
    ("es", "MX"),
    ("es", "PY"),
    ("es", "ES"),
    ("fr", "CA"),
    ("fr", "FR"),
    ("he", "IL"),
    ("hr", "HR"),
    ("hy", "AM"),
    ("id", "ID"),
    ("nb", "NO"),
    ("no", "NO"),
    ("da", "DK"),
    ("vi", "VI"),
    ("fr", "CH"),
    ("en", "IE"),
];

/// The body of the four `testAllPhoneNumber*` tests: `COUNT` numbers of one method per locale, each valid for the
/// locale's country.
fn all_valid_for_region(method: fn(&datafaker::providers::PhoneNumber<'_>) -> datafaker::Result<String>) {
    for (language, country) in LOCALES {
        let f = Faker::with_locale(Locale::new(language, country));
        let provider = f.phone_number();
        for _ in 0..COUNT {
            let phone_number = method(&provider).unwrap();
            let proto = parse(&phone_number, country).unwrap();
            assert!(
                util().is_valid_number_for_region(&proto, country),
                "Invalid phone {phone_number} for locale {language}_{country}"
            );
        }
    }
}

/// `PhoneNumberTest#testAllPhoneNumberNational`.
#[test]
fn test_all_phone_number_national() {
    all_valid_for_region(|p| p.phone_number());
}

/// `PhoneNumberTest#testAllPhoneNumberInternational`.
#[test]
fn test_all_phone_number_international() {
    all_valid_for_region(|p| p.phone_number_international());
}

/// `PhoneNumberTest#testAllPhoneNumberMobile`.
#[test]
fn test_all_phone_number_mobile() {
    all_valid_for_region(|p| p.cell_phone());
}

/// `PhoneNumberTest#testAllPhoneNumberMobileInternational`.
#[test]
fn test_all_phone_number_mobile_international() {
    all_valid_for_region(|p| p.cell_phone_international());
}

/// `PhoneNumberTest#testCellPhone`.
#[test]
fn test_cell_phone() {
    let f = Faker::new();
    assert_matches(r"\(?\d+\)?([- .]\d+){1,3}", &f.phone_number().cell_phone().unwrap());
}

/// `PhoneNumberTest#testPhoneNumber`.
#[test]
fn test_phone_number() {
    let f = Faker::new();
    assert_matches(r"\(?\d+\)?([- .]x?\d+){1,5}", &f.phone_number().phone_number().unwrap());
}

/// `PhoneNumberTest#testExtension`.
#[test]
fn test_extension() {
    let f = Faker::new();
    assert_matches(r"\d{4}", &f.phone_number().extension());
}

/// `PhoneNumberTest#testSubscriberNumber`.
#[test]
fn test_subscriber_number() {
    let f = Faker::new();
    assert_matches(r"\d{4}", &f.phone_number().subscriber_number());
}

/// `PhoneNumberTest#testSubscriberNumberWithLength`.
#[test]
fn test_subscriber_number_with_length() {
    let f = Faker::new();
    assert_matches(r"\d{10}", &f.phone_number().subscriber_number_with(10));
}

fn estonian() -> Faker {
    Faker::with_locale(Locale::new("et", "EE"))
}

fn moldovan() -> Faker {
    Faker::with_locale(Locale::new("ro", "MD"))
}

/// `PhoneNumberTest#cellPhone_estonia` (repeated 100 times).
#[test]
fn cell_phone_estonia() {
    let f = estonian();
    for _ in 0..COUNT {
        assert_matches(r"[358]\d{6,7}", &no_spaces(&f.phone_number().cell_phone().unwrap()));
    }
}

/// `PhoneNumberTest#phoneNumberNational_estonia` (repeated 100 times).
#[test]
fn phone_number_national_estonia() {
    let f = estonian();
    for _ in 0..COUNT {
        assert_matches(r"[34678]\d{6,7}", &no_spaces(&f.phone_number().phone_number_national().unwrap()));
    }
}

/// `PhoneNumberTest#phoneNumberInternational_estonia` (repeated 100 times).
#[test]
fn phone_number_international_estonia() {
    let f = estonian();
    for _ in 0..COUNT {
        assert_matches(r"\+372[34678]\d{6,7}", &no_spaces(&f.phone_number().phone_number_international().unwrap()));
    }
}

/// `PhoneNumberTest#cellPhone_moldova` (repeated 10 times).
#[test]
fn cell_phone_moldova() {
    let f = moldovan();
    for _ in 0..10 {
        assert_matches(r"0[567]\d{7}", &no_spaces(&f.phone_number().cell_phone().unwrap()));
    }
}

/// `PhoneNumberTest#phoneNumber_moldova` (repeated 10 times).
#[test]
fn phone_number_moldova() {
    let f = moldovan();
    for _ in 0..10 {
        assert_matches(r"0\d{8}", &no_spaces(&f.phone_number().phone_number().unwrap()));
    }
}

/// `PhoneNumberTest#phoneNumberInternational_moldova` (repeated 10 times).
#[test]
fn phone_number_international_moldova() {
    let f = moldovan();
    for _ in 0..10 {
        assert_matches(r"\+373\d{8}", &no_spaces(&f.phone_number().phone_number_international().unwrap()));
    }
}

/// `PhoneNumberValidityFinderTest#testAllCellPhoneForLocale` (repeated 100 times).
#[test]
fn test_all_cell_phone_for_locale() {
    for _ in 0..COUNT {
        let region = "GB";
        let f = Faker::with_locale(Locale::new("en", region));
        let generated_number = f.phone_number().phone_number().unwrap();
        let parsed = util().parse(&generated_number, Some(region)).unwrap();
        assert!(util().is_valid_number(&parsed), "Generated phone number {generated_number} for region {region}");
    }
}

/// `PhoneNumberValidityFinderTest#testValidNumber`.
#[test]
fn test_valid_number() {
    let parsed = util().parse("0140 123456", Some("SE")).unwrap();
    assert!(util().is_valid_number(&parsed));
}

/// `PhoneNumberValidityFinderTest#testLanguageOnlyPersianLocaleUsesIran`.
#[test]
fn test_language_only_persian_locale_uses_iran() {
    let f = Faker::with_locale(Locale::language_only("fa"));
    assert_eq!(f.phone_number().country_code_iso2(), "IR");
}

/// `PhoneNumberValidityFinderTest#detectsCountryByLanguage`.
#[test]
fn detects_country_by_language() {
    for (language, expected_country, description) in [
        ("en", "US", "English"),
        ("hy", "AM", "Armenian → Armenia"),
        ("uk", "UA", "Ukranian → Ukraine"),
        ("ja", "JP", "Japanese → Japan"),
        ("fa", "IR", "Persian (Farsi) → Iran"),
        ("ka", "GE", "Georgia"),
        ("sq", "AL", "Albanian → Albania"),
        ("cs", "CZ", "Czech"),
        ("be", "BY", "Belarusian"),
        ("he", "IL", "Hebrew → Israel"),
        ("ta", "IN", "Tamil → India"),
        ("et", "EE", "Estonian → Estonia"),
        ("el", "GR", "Greek → Greece"),
        ("eu", "ES", "Basque → Spain"),
        ("ca", "ES", "Catalan → Spain"),
        ("cy", "GB", "Welsh → United Kingdom"),
        ("ga", "IE", "Irish/Gaelic → Ireland"),
        ("is", "IS", "Icelandic → Iceland"),
        ("bs", "BA", "Bosnian → Bosnia & Herzegovina"),
        ("ar", "SA", "Arabic → Saudi Arabia"),
        ("hi", "IN", "Hindi → India"),
        ("zh", "CN", "Chinese → China"),
        ("am", "ET", "Amharic → Ethiopia"),
        ("sw", "TZ", "Swahili → Tanzania"),
        ("af", "ZA", "Afrikaans → South Africa"),
    ] {
        let f = Faker::with_locale(Locale::language_only(language));
        assert_eq!(f.phone_number().country_code_iso2(), expected_country, "{description}");
    }
}

/// `PhoneNumberValidityFinderTest#testAllPhoneNumbers`: every locale of `allSupportedLocales()`, read as
/// `createLocale` reads it.
#[test]
fn test_all_phone_numbers() {
    let faker = Faker::new();
    for raw in faker.locality().all_supported_locales() {
        let locale = if let Some(country) = raw.strip_prefix('_') {
            Locale::new("en", &country.to_uppercase())
        } else if raw.contains('-') {
            let parts: Vec<&str> = raw.split('-').collect();
            Locale::new(parts[0], parts[1])
        } else {
            Locale::new(raw, "")
        };
        let f = Faker::with_locale(locale.clone());
        let provider = f.phone_number();
        for _ in 0..COUNT {
            let generated_number = provider.phone_number().unwrap();
            let parsed = parse(&generated_number, &provider.country_code_iso2()).unwrap();
            assert!(
                util().is_valid_number(&parsed),
                "Generated phone number {generated_number} for locale {locale} (country: {})",
                provider.country_code_iso2()
            );
        }
    }
}
