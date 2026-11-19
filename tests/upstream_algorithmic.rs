//! Tests of Datafaker 2.7.0's algorithmic base providers (tranche T1b) that `scripts/gen-providers.py` does not read:
//! cases with arguments, other locales, and assertions other than a regex. Unseeded fakers as upstream; repeated
//! tests run their body as often.

mod common;

use common::assert_matches;
use datafaker::{Faker, Locale};

fn faker_in(language: &str, country: &str) -> Faker {
    if country.is_empty() { Faker::with_locale(Locale::language_only(language)) } else { Faker::with_locale(Locale::new(language, country)) }
}

/// `BoolTest.testBool`.
#[test]
fn bool_test_bool() {
    let f = Faker::new();
    let mut seen = [false; 2];
    for _ in 0..10 {
        seen[f.bool().bool() as usize] = true;
    }
    assert!(seen.iter().any(|s| *s));
}

/// `ColorTest.testHexNoHashSign`.
#[test]
fn color_test_hex_no_hash_sign() {
    assert_matches("^[0-9A-F]{6}$", &Faker::new().color().hex_with(false));
}

/// `ColorLocaleTest`: `color.name` in every international faker (`ProviderListLocaleTest.internationalFakers`).
#[test]
fn color_locale_provider_list() {
    for f in common::international_fakers() {
        common::provider_list(&f, "color.name", &f.color().name().unwrap(), "");
    }
}

/// `LanguageCodeTest.languageCodeISO639ShouldBe2LettersInLength`.
#[test]
fn language_code_iso639_should_be_2_letters_in_length() {
    assert_eq!(Faker::new().language_code().iso639().unwrap().chars().count(), 2);
}

/// `DrivingLicenseTest.drivingLicense` (`@EnumSource(LicensePattern.class)`): every state's pattern, 100 times.
#[test]
fn driving_license_driving_license() {
    let patterns = common::java_source_enum(
        "providers/base/DrivingLicenseTest.java",
        r#"(?m)^\s*([A-Z]{2})\("[A-Z]{2}", "((?:[^"\\]|\\.)*)"\)"#,
    );
    assert_eq!(patterns.len(), 51);
    let f = Faker::new();
    for (state, regex) in patterns {
        for _ in 0..100 {
            assert_matches(&regex, &f.driving_license().driving_license(&state).unwrap());
        }
    }
}

/// `MedicalTest.testDiagnosisCodeUS`.
#[test]
fn medical_test_diagnosis_code_us() {
    let f = faker_in("en", "US");
    for _ in 0..100 {
        assert_matches(r"[A-TV-Z][0-9][0-9AB](\.[0-9A-TV-Z]{0,4})?", &f.medical().diagnosis_code().unwrap());
    }
}

/// `MedicalTest.testDiagnosisCodeAU`.
#[test]
fn medical_test_diagnosis_code_au() {
    let f = faker_in("en", "AU");
    for _ in 0..100 {
        assert_matches(r"[A-Z][0-9]{1,2}\.[0-9]{1,2}", &f.medical().diagnosis_code().unwrap());
    }
}

/// `MedicalTest.testDiagnosisCodeNotAustraliaNorUS`.
#[test]
fn medical_test_diagnosis_code_not_australia_nor_us() {
    let f = faker_in("fr", "FR");
    for _ in 0..100 {
        assert_matches(r"^[A-Z][0-9]{1,2}(\.[0-9])?$", &f.medical().diagnosis_code().unwrap());
    }
}

/// `MedicalTest.testProcedureCodes`.
#[test]
fn medical_test_procedure_codes() {
    let f = Faker::new();
    for _ in 0..100 {
        assert_matches("^[a-hj-np-zA-HJ-NP-Z0-9]{7}$", &f.medical().procedure_code().unwrap());
    }
}

/// `PassportTest`: every locale case, ten times each (length, regex, and Dutch numbers without `O`).
#[test]
fn passport_tests() {
    let cases: &[(&str, &str, &str, usize, &str)] = &[
        ("testDefaultLocale", "en", "", 9, ""),
        ("testValidDutch", "nl", "NL", 9, r".*?\d$"),
        ("testValidChinese", "zh", "CN", 9, ""),
        ("testValidUnitedStates", "en", "US", 9, ""),
        ("testValidAustralia", "en", "AU", 8, "[A-Z][0-9]{7}"),
        ("testValidCanada", "en", "CA", 8, ""),
        ("testValidUnitedKingdom", "en", "GB", 9, ""),
        ("testValidRepublicOfIreland", "en", "IE", 9, ""),
        ("testValidJapan", "ja", "", 9, "[MT][A-Z][0-9]{7}"),
        ("testValidSpain", "es", "", 0, "[A-z0-9]{2,3}[0-9]{6}"),
        ("testValidBulgaria", "bg", "", 9, ""),
        ("testValidFinland", "fi", "FI", 9, ""),
        ("testValidBrazil", "pt", "BR", 8, "[A-Z]{2}[0-9]{6}"),
    ];
    for (name, language, country, size, regex) in cases {
        let f = faker_in(language, country);
        for _ in 0..10 {
            let valid = f.passport().valid().unwrap();
            if *size > 0 {
                assert_eq!(valid.chars().count(), *size, "{name}: {valid:?}");
            }
            if !regex.is_empty() {
                assert_matches(regex, &valid);
            }
            if *name == "testValidDutch" {
                assert!(!valid.contains('O'), "{name}: {valid:?}");
            }
        }
    }
}

/// `LocalityTest.allSupportedLocales`: the locales are the top-level locale files, lower-cased.
#[test]
fn locality_all_supported_locales() {
    let mut files: Vec<String> = datafaker::resources::FILES
        .iter()
        .filter(|(path, _)| !path.contains('/'))
        .map(|(path, _)| path.to_lowercase().replace(".yml", ""))
        .collect();
    let mut locales: Vec<String> = Faker::new().locality().all_supported_locales().iter().map(|l| l.to_string()).collect();
    files.sort();
    locales.sort();
    assert_eq!(locales, files, "Somebody forgot to add the new locale to Locality.LOCALES");
}

/// `LocalityTest.displayName`.
#[test]
fn locality_display_name() {
    assert!(!Faker::new().locality().display_name().unwrap().is_empty());
}

/// `LocalityTest.localeStringRandom`.
#[test]
fn locality_locale_string_random() {
    let locality_faker = Faker::new();
    let locality = locality_faker.locality();
    let a = locality.locale_string_with_random(&datafaker::java::Random::new(5)).unwrap();
    let b = locality.locale_string_with_random(&datafaker::java::Random::new(5)).unwrap();
    assert_eq!(a, b);
}

/// `LocalityTest.localeStringWithRandom`.
#[test]
fn locality_locale_string_with_random() {
    let f = Faker::new();
    for _ in 0..10 {
        let locale = f.locality().locale_string_with_random(&datafaker::java::Random::from_entropy()).unwrap();
        assert!(f.locality().all_supported_locales().contains(&locale.as_str()));
    }
}

/// `LocalityTest.localeStringWithoutReplacement`: two rounds through all locales, each a subset of them.
#[test]
fn locality_locale_string_without_replacement() {
    locale_string_without_replacement_rounds(&datafaker::java::Random::from_entropy());
}

/// `LocalityTest.localeStringWithoutReplacementWithCustomRandomGenerator`: the same over a `SplittableRandom`.
#[test]
fn locality_locale_string_without_replacement_with_custom_random_generator() {
    locale_string_without_replacement_rounds(&datafaker::java::SplittableRandom::from_entropy());
}

fn locale_string_without_replacement_rounds(random: &dyn datafaker::java::RandomGenerator) {
    let f = Faker::new();
    let all = f.locality().all_supported_locales();
    for _ in 0..2 {
        let returned: std::collections::HashSet<String> =
            (0..all.len()).map(|_| f.locality().locale_string_without_replacement_with(random).unwrap()).collect();
        assert!(returned.iter().all(|l| all.contains(&l.as_str())));
    }
}

/// `LocalityTest.localeString`.
#[test]
fn locality_locale_string() {
    let f = Faker::new();
    assert!(f.locality().all_supported_locales().contains(&f.locality().locale_string().unwrap().as_str()));
}

/// `LocalityTest.localeWithoutReplacement`.
#[test]
fn locality_locale_without_replacement() {
    assert!(!Faker::new().locality().locale_string_without_replacement().unwrap().is_empty());
}

const CAPITALIZED_WORD_REGEX: &str = "[A-Z][a-z]+";

fn promotion_code_regex() -> String {
    format!("{CAPITALIZED_WORD_REGEX}(-{CAPITALIZED_WORD_REGEX})*")
}

/// `CommerceTest.productNameConsistsOf_adjective_material_and_productType`.
#[test]
fn commerce_product_name_consists_of_adjective_material_and_product_type() {
    let name = common::test_locale_faker().commerce().product_name().unwrap();
    let expected = [
        "Silver Linings Playbook",
        "Silver Linings Train",
        "Silver Iron Playbook",
        "Silver Iron Train",
        "Silent Linings Playbook",
        "Silent Linings Train",
        "Silent Iron Playbook",
        "Silent Iron Train",
    ];
    assert!(expected.contains(&name.as_str()), "{name}");
}

/// `CommerceTest.testPrice`.
#[test]
fn commerce_test_price() {
    let price = Faker::new().commerce().price();
    assert_matches(r"\d{1,3}\.\d{2}", &price);
    let value: f32 = price.parse().unwrap();
    assert!((0.0..=100.0).contains(&value));
}

/// `CommerceTest.testPriceMinMax`.
#[test]
fn commerce_test_price_min_max() {
    let price = Faker::new().commerce().price_between(100.0, 1000.0);
    assert_matches(r"\d{3,4}\.\d{2}", &price);
    let value: f32 = price.parse().unwrap();
    assert!((100.0..=1000.0).contains(&value));
}

/// `CommerceTest.testPromotionCodeDigits`.
#[test]
fn commerce_test_promotion_code_digits() {
    let regex = promotion_code_regex();
    assert_matches(&format!(r"{regex}{regex}\d{{3}}"), &Faker::new().commerce().promotion_code_with(3).unwrap());
}

/// `CommerceLocaleTest`: material, brand and vendor in every international faker.
#[test]
fn commerce_locale_provider_list() {
    for f in common::international_fakers() {
        let c = f.commerce();
        common::provider_list(&f, "commerce.product_name.material", &c.material().unwrap(), r"[\p{L}\p{M}\p{Pi}\p{Pf}]{2,}");
        common::provider_list(&f, "commerce.brand", &c.brand().unwrap(), r"\w{2,}");
        common::provider_list(&f, "commerce.vendor", &c.vendor().unwrap(), r"[\w\s]{6,}");
    }
}

const PHRASE_PATTERN: &str = r"(\w+[ /-]?){1,9}";

/// `CompanyTest.testCatchPhrase`.
#[test]
fn company_test_catch_phrase() {
    assert_matches(PHRASE_PATTERN, &Faker::new().company().catch_phrase().unwrap());
}

/// `CompanyTest.testBs`.
#[test]
fn company_test_bs() {
    assert_matches(PHRASE_PATTERN, &Faker::new().company().bs().unwrap());
}

/// `CompanyTest.testUrl`.
#[test]
fn company_test_url() {
    let f = Faker::new();
    for _ in 0..10 {
        assert_matches(r"(([a-z0-9]|[a-z0-9][a-z0-9\-]*[a-z0-9])\.)*([a-z0-9]|[a-z0-9][a-z0-9\-]*[a-z0-9])", &f.company().url().unwrap());
    }
}

/// `CountryTest.testFlag`.
#[test]
fn country_test_flag() {
    let f = Faker::new();
    for _ in 0..10 {
        assert_matches(r"^https://flags.fmcdn\.net/data/flags/w580/[a-zA-Z0-9_]+\.png$", &f.country().flag().unwrap());
    }
}

/// `BarcodeTest.isBarcodeValid`: weights 3, 1, 3, … from the first digit.
fn is_barcode_valid(barcode: i64) -> bool {
    let sum: u32 = barcode.to_string().chars().enumerate().map(|(i, c)| c.to_digit(10).unwrap() * if (i + 1) % 2 == 0 { 1 } else { 3 }).sum();
    sum.is_multiple_of(10)
}

/// `BarcodeTest`: the length tests and checksum tests.
#[test]
fn barcode_lengths_and_check_sums() {
    let f = Faker::new();
    let b = f.barcode();
    assert_matches("[0-9]{13}", &b.ean13().unwrap().to_string());
    assert_matches("[0-9]{13}", &b.gtin13().unwrap().to_string());
    assert_matches("[0-9]{8}", &b.ean8().unwrap().to_string());
    assert_matches("[0-9]{8}", &b.gtin8().unwrap().to_string());
    assert_matches("[0-9]{14}", &b.gtin14().unwrap().to_string());
    assert_matches("[0-9]{12}", &b.gtin12().unwrap().to_string());
    assert!(is_barcode_valid(b.gtin12().unwrap()));
    assert!(is_barcode_valid(b.gtin14().unwrap()));
    assert!(is_barcode_valid(b.ean8().unwrap()));
    // `testEan13CheckSum`: weights 1, 3, 1, ….
    let ean13 = b.ean13().unwrap().to_string();
    let sum: u32 = ean13.chars().enumerate().map(|(i, c)| c.to_digit(10).unwrap() * if (i + 1) % 2 == 0 { 3 } else { 1 }).sum();
    assert_eq!(sum % 10, 0, "{ean13}");
}

/// commons-validator's `ISBNValidator.getInstance(false).isValidISBN10`: ten digits (the last may be `X`), or the
/// formatted form with three separators; `Σ (i+1)·dᵢ ≡ 0 (mod 11)`.
fn is_valid_isbn10(isbn: &str) -> bool {
    if !common::matches(r"\d{9}[0-9X]|\d{1,5}[- ]\d{1,7}[- ]\d{1,6}[- ][0-9X]", isbn) {
        return false;
    }
    let digits: Vec<u32> = isbn.chars().filter(|c| *c != '-' && *c != ' ').map(|c| if c == 'X' { 10 } else { c.to_digit(10).unwrap() }).collect();
    digits.len() == 10 && digits.iter().enumerate().map(|(i, d)| (i as u32 + 1) * d).sum::<u32>() % 11 == 0
}

/// `EAN13CheckDigit.isValid`: weights 1, 3, 1, … from the left, sum divisible by 10.
fn is_valid_ean13(code: &str) -> bool {
    code.len() == 13 && code.chars().enumerate().map(|(i, c)| c.to_digit(10).unwrap() * if i % 2 == 0 { 1 } else { 3 }).sum::<u32>() % 10 == 0
}

/// `ISBNValidator.isValidISBN13`: `978`/`979`, thirteen digits or the formatted form, an EAN-13 check digit.
fn is_valid_isbn13(isbn: &str) -> bool {
    common::matches(r"(978|979)(\d{10}|[- ]\d{1,5}[- ]\d{1,7}[- ]\d{1,6}[- ][0-9])", isbn)
        && is_valid_ean13(&isbn.replace(['-', ' '], ""))
}

/// `CodeTest.isbn10DefaultIsNoSeparator`, `isbn13DefaultIsNoSeparator`, `testIsbn10`, `testIsbn13` (100 times
/// each; commons-validator's checks are written out above).
#[test]
fn code_isbn10_and_isbn13() {
    let f = Faker::new();
    let code = f.code();
    for _ in 0..100 {
        let isbn10 = code.isbn10().unwrap();
        assert!(is_valid_isbn10(&isbn10) && !isbn10.contains('-'), "{isbn10}");
        let isbn13 = code.isbn13().unwrap();
        assert!(is_valid_isbn13(&isbn13) && !isbn13.contains('-'), "{isbn13}");
        let (no_sep, sep) = (code.isbn10_with(false).unwrap(), code.isbn10_with(true).unwrap());
        assert_eq!(no_sep.len(), 10);
        assert!(is_valid_isbn10(&no_sep), "{no_sep}");
        assert_eq!(sep.len(), 13);
        assert!(is_valid_isbn10(&sep), "{sep}");
        let (no_sep, sep) = (code.isbn13_with(false).unwrap(), code.isbn13_with(true).unwrap());
        assert_eq!(no_sep.len(), 13);
        assert!(is_valid_isbn13(&no_sep), "{no_sep}");
        assert_eq!(sep.len(), 17);
        assert!(is_valid_isbn13(&sep), "{sep}");
    }
}

/// `CodeTest.testOverrides`: `code.isbn_gs1`, `isbn_group` and `isbn_registrant` from `test.yml`.
#[test]
fn code_test_overrides() {
    let f = common::test_locale_faker();
    for _ in 0..100 {
        assert_matches(r"9971-\d-\d{4}-(\d|X)", &f.code().isbn10_with(true).unwrap());
        assert_matches(r"(333|444)-9971-\d-\d{4}-\d", &f.code().isbn13_with(true).unwrap());
    }
}

/// `CodeTest.imei`.
#[test]
fn code_imei() {
    let imei = Faker::new().code().imei().unwrap();
    assert_matches(r"[\d.:\-\s]+", &imei);
    assert!(common::is_luhn_valid(&imei), "{imei}");
}

/// `CodeTest.ean13`, `gtin13`.
#[test]
fn code_ean13_gtin13() {
    let f = Faker::new();
    for code in [f.code().ean13().unwrap(), f.code().gtin13().unwrap()] {
        assert_matches(r"\d{13}", &code);
        assert!(is_valid_ean13(&code), "{code}");
    }
}

/// `CodeTest.isbnGs1`, `isbnGroup`, `isbnRegistrant` (100 times).
#[test]
fn code_isbn_parts() {
    let f = Faker::new();
    assert_matches("978|979", &f.code().isbn_gs1().unwrap());
    assert_matches("[01]", &f.code().isbn_group().unwrap());
    for _ in 0..100 {
        assert_matches("[0-9]{1,7}-[0-9]{1,6}", &f.code().isbn_registrant().unwrap());
    }
}

/// iban4j's `IbanUtil.validate` as far as the tests reach it: country code, two check digits, an upper-case
/// alphanumeric BBAN, and `(BBAN + country + check) mod 97 = 1`.
fn assert_iban_valid(iban: &str) {
    assert_matches("[A-Z]{2}[0-9]{2}[0-9A-Z]+", iban);
    let rearranged = format!("{}{}", &iban[4..], &iban[..4]);
    let mut rem = 0u32;
    for c in rearranged.chars() {
        for d in c.to_digit(36).unwrap().to_string().chars() {
            rem = (rem * 10 + d.to_digit(10).unwrap()) % 97;
        }
    }
    assert_eq!(rem, 1, "{iban}");
}

/// `FinanceTest.creditCard` (100 times) and `creditCardWithType`.
#[test]
fn finance_credit_card() {
    let f = Faker::new();
    for _ in 0..100 {
        let card = f.finance().credit_card().unwrap();
        assert!(common::is_luhn_valid(&card.replace('-', "")), "{card}");
    }
    for t in datafaker::providers::CreditCardType::ALL {
        let card = f.finance().credit_card_of(t).unwrap();
        assert!(common::is_luhn_valid(&card.replace('-', "")), "{t:?}: {card}");
    }
}

/// `FinanceTest.bic` (100 times; iban4j's `BicUtil.validate` as a regex with an upper-case country).
#[test]
fn finance_bic() {
    let f = Faker::new();
    for _ in 0..100 {
        assert_matches("[A-Z]{4}[A-Z]{2}[0-9A-Z]{2}([0-9A-Z]{3})?", &f.finance().bic().unwrap());
    }
}

/// `FinanceTest.iban` (100 times), `ibanWithCountryCode`, `ibanCountryCodes`, `ibanWithAllCountryCodes`,
/// `costaRicaIbanMustBeValid`. iban4j's `IbanRegistry` lengths are not available here; the lengths come from the
/// BBAN patterns themselves, so they are checked through validity only (and `DE` against its regex).
#[test]
fn finance_iban() {
    let f = Faker::new();
    let countries = datafaker::providers::Finance::iban_supported_countries();
    assert!(countries.len() > 100);
    for _ in 0..100 {
        let iban = f.finance().iban().unwrap();
        assert_iban_valid(&iban);
        assert!(countries.contains(&&iban[..2]));
    }
    assert_matches(r"DE\d{20}", &f.finance().iban_for("DE").unwrap());
    for country in countries {
        let iban = f.finance().iban_for(country).unwrap().to_uppercase();
        assert_iban_valid(&iban);
        assert!(iban.starts_with(country));
    }
    assert_iban_valid(&f.finance().iban_for("CR").unwrap());
}

/// `FinanceTest.visaCard`, `discoverCard`, `unionpayCard` (100 times each).
#[test]
fn finance_card_prefixes() {
    use datafaker::providers::CreditCardType;
    let f = Faker::new();
    for _ in 0..100 {
        let visa = f.finance().credit_card_of(CreditCardType::Visa).unwrap().replace('-', "");
        assert!(visa.starts_with('4') && visa.len() == 16, "{visa}");
        let discover = f.finance().credit_card_of(CreditCardType::Discover).unwrap().replace('-', "");
        assert!(discover.starts_with('6') && discover.len() == 16, "{discover}");
        let unionpay = f.finance().credit_card_of(CreditCardType::Unionpay).unwrap().replace('-', "");
        assert_eq!(unionpay.len(), 16);
        assert!(["62", "64", "65", "81"].iter().any(|p| unionpay.starts_with(p)), "{unionpay}");
    }
}

/// `FinanceTest.usRoutingNumber` (100 times).
#[test]
fn finance_us_routing_number() {
    let f = Faker::new();
    for _ in 0..100 {
        let rtn = f.finance().us_routing_number().unwrap();
        assert_matches(r"\d{9}", &rtn);
        let d: Vec<u32> = rtn.chars().map(|c| c.to_digit(10).unwrap()).collect();
        let check: u32 = (0..3).map(|i| d[i * 3] * 3 + d[i * 3 + 1] * 7 + d[i * 3 + 2]).sum();
        assert_eq!(check % 10, 0, "{rtn}");
    }
}

fn has_cyrillic(s: &str) -> bool {
    s.chars().any(|c| ('а'..='я').contains(&c) || ('А'..='Я').contains(&c))
}

/// `AddressTest.testLatinStreetName` (its `@ValueSource` is unused upstream: always the English faker),
/// `testCyrillicStreetName` and `cyrillicStreetName`.
#[test]
fn address_street_name_scripts() {
    for _ in ["en", "id", "ca", "cs"] {
        let name = Faker::new().address().street_name().unwrap();
        assert!(!name.is_empty() && !has_cyrillic(&name), "{name}");
    }
    for locale in ["be", "bg", "by", "mk", "ru", "ru_MD", "uk", "bg", "ru", "uk", "by"] {
        let name = Faker::with_locale(Locale::language_only(locale)).address().street_name().unwrap();
        assert!(has_cyrillic(&name), "{locale}: {name}");
    }
}

/// `AddressTest.testStreetAddressStartsWithNumber`, `testStreetAddressNumberIsANumberGreaterZero`,
/// `testStreetAddressIncludeSecondary`, `testFullAddress`.
#[test]
fn address_street_address() {
    let f = Faker::new();
    assert_matches("[0-9]+ .+", &f.address().street_address().unwrap());
    assert_matches("[1-9][0-9]*", &f.address().street_address_number().unwrap());
    assert!(!f.address().street_address_with(true).unwrap().is_empty());
    assert!(!f.address().full_address().unwrap().is_empty());
}

/// `AddressTest.usingOnlyCountryCodeWithoutLanguage`.
#[test]
fn address_using_only_country_code_without_language() {
    let xx = |country: &str| Faker::with_locale(Locale::new("xx", country));
    for country in ["AL", "AM", "BG", "BY"] {
        assert_eq!(xx(country).address().country_code().unwrap(), country);
    }
    assert_matches(r"\d{6}", &xx("BY").address().postcode().unwrap());
    assert_matches(r"\d{5}", &xx("EE").address().postcode().unwrap());
}

/// `AddressTest.testLatitude`, `testLongitude`, `testLocaleLatitude`, `testLocaleLongitude` (10 times each).
#[test]
fn address_latitude_longitude() {
    let f = Faker::new();
    for _ in 0..10 {
        let lat: f64 = f.address().latitude().parse().unwrap();
        assert!((-90.0..=90.0).contains(&lat));
        let lon: f64 = f.address().longitude().parse().unwrap();
        assert!((-180.0..=180.0).contains(&lon));
        assert_matches(r"-?\d{1,3}\.\d+", &f.address().latitude());
        assert_matches(r"-?\d{1,3}\.\d+", &f.address().longitude());
    }
}

/// `AddressTest.testCityWithLocaleFranceAndSeed`.
#[test]
fn address_city_with_locale_france_and_seed() {
    let seed = datafaker::java::Random::from_entropy().next_long();
    let first = Faker::with_locale_and_seed(Locale::new("fr", "FR"), seed);
    let second = Faker::with_locale_and_seed(Locale::new("fr", "FR"), seed);
    for _ in 0..100 {
        assert_eq!(first.address().city().unwrap(), second.address().city().unwrap());
    }
}

/// `AddressTest.fullAddress_estonia`, `eirCode_ireland`.
#[test]
fn address_estonia_and_ireland() {
    assert!(!faker_in("et", "EE").address().full_address().unwrap().is_empty());
    assert!(!faker_in("en", "IE").address().eircode().unwrap().is_empty());
}

/// `AddressTest.testZipCodeByState`, `testHungarianZipCodeByState`, `testCountyByZipCode`,
/// `testCountyForWrongZipCode` (its `@NullSource` case has no Rust form: `"null"` stands in for it, as upstream's
/// message prints it), `testZipIsFiveChars`, `testZipPlus4IsTenChars`, `testZipPlus4IsNineDigits`.
#[test]
fn address_us_zip_codes() {
    let us = faker_in("en", "US");
    let a = us.address();
    assert_matches("[0-9]{5}", &a.zip_code_by_state(&a.state_abbr().unwrap()).unwrap());
    let hu = faker_in("hu", "");
    assert_matches("[0-9]{4}", &hu.address().zip_code_by_state(&hu.address().state_abbr().unwrap()).unwrap());
    assert!(!a.county_by_zip_code("47732").unwrap().is_empty());
    for zip in ["null", "1", "asd", "qwe", "wrong"] {
        let e = a.county_by_zip_code(zip).unwrap_err();
        assert_eq!((e.simple_class(), e.message()), ("RuntimeException", format!("County is not configured for postcode {zip}").as_str()));
    }
    assert_eq!(a.zip_code().unwrap().len(), 5);
    assert_eq!(a.zip_code_plus4().unwrap().len(), 10);
    let plus4 = a.zip_code_plus4().unwrap();
    let (five, four) = plus4.split_once('-').unwrap();
    assert_matches("[0-9]{5}", five);
    assert_matches("[0-9]{4}", four);
}

/// `AddressTest.testArgentineZipCodes`: 4,025 distinct codes of `es-AR` (upstream draws until it has them).
#[test]
fn address_argentine_zip_codes() {
    let f = Faker::with_locale(Locale::for_language_tag("es-AR"));
    let mut codes = std::collections::HashSet::new();
    while codes.len() < 4025 {
        codes.insert(f.address().zip_code().unwrap());
    }
    for code in codes {
        assert_matches("^[ABDEFGHJKLMNPQRSTUVWXYZ][0-9]{4}$", &code);
    }
}

/// `AddressTest.testMailbox` (10 times).
#[test]
fn address_mailbox() {
    let f = Faker::new();
    for _ in 0..10 {
        assert_matches("PO Box [0-9]{2,4}", &f.address().mail_box().unwrap());
    }
}

/// `AddressTest.testLatLonEnUs`, `testLatLonNl`, `testLonLatEnUs`, `testLonLatNl` (10 times), `testLonLatRU`,
/// `testLatLonRU`, with each locale's decimal separator.
#[test]
fn address_lat_lon_formats() {
    let lat_lon = |sep: &str, delimiter: &str| format!(r"-?\d{{1,2}}\{sep}\d{{5,10}}{delimiter}-?\d{{1,3}}\{sep}\d{{5,10}}+");
    let lon_lat = |sep: &str, delimiter: &str| format!(r"-?\d{{1,3}}\{sep}\d{{5,10}}+{delimiter}-?\d{{1,2}}\{sep}\d{{5,10}}");
    let (us, nl, ru) = (faker_in("en", "US"), faker_in("nl", "NL"), faker_in("ru", "RU"));
    for _ in 0..10 {
        assert_matches(&lat_lon(".", ","), &us.address().lat_lon());
        assert_matches(&lat_lon(",", ","), &nl.address().lat_lon());
        assert_matches(&lon_lat(".", ","), &us.address().lon_lat());
        assert_matches(&lon_lat(",", ","), &nl.address().lon_lat());
    }
    assert_matches(&lon_lat(",", ";"), &ru.address().lon_lat_with(";"));
    assert_matches(&lat_lon(",", ";"), &ru.address().lat_lon_with(";"));
}

/// `AddressTest.dutchAddress`, `belgianSAddress`, `belgianZipcode`, `australiaAddress`, `irishAddress` (10 times
/// each), `testCityCnSuffix`.
#[test]
fn address_national_formats() {
    let (nl, be, au, ie, cn) = (faker_in("nl", "NL"), faker_in("nl", "BE"), faker_in("en", "AU"), faker_in("en", "IE"), faker_in("zh", "CN"));
    for _ in 0..10 {
        assert_matches("[A-Z]{2}", &nl.address().state_abbr().unwrap());
        assert_matches("[A-Z].+, [0-9]{4} [A-Z]{2}, [A-Z].+", &nl.address().full_address().unwrap());
        assert_matches("[A-Z]{3}", &be.address().state_abbr().unwrap());
        assert_matches("[A-Z].+, [0-9]{4}, [A-Z].+", &be.address().full_address().unwrap());
        let zip: i32 = be.address().zip_code().unwrap().parse().unwrap();
        assert!((1000..=9992).contains(&zip));
        assert_matches("(Unit|[0-9]).+, [A-Z].+, [A-Z]{2,3} [0-9]{4}", &au.address().full_address().unwrap());
        assert_matches(
            r"(?:.+, [A-Z].+, Co\. [A-Z].+)|(?:.+, Dublin D[A-Z0-9]{2,3})|(?:.+, [A-Z].+, (?:Ireland|Republic of Ireland))",
            &ie.address().full_address().unwrap(),
        );
        assert_matches(r"[一-龥]{1,7}(?:省|自治区)", &cn.address().city_suffix().unwrap());
    }
}

const FLIGHT_IATA_CODE: &str = r"[A-Z0-9]{2}\d{1,4}";
const FLIGHT_ICAO_CODE: &str = "[A-Z]{3}[0-9]+";

/// `AviationTest.flight_ICAO`, `flight_IATA` (its `(String) null` case is `flight_by_name` with any other name:
/// upstream's `equalsIgnoreCase` is false for null too), `flight_default`. `flight_nullCodeNotAllowed` has no Rust
/// form.
#[test]
fn aviation_flights() {
    use datafaker::providers::AviationCodeType::{Iata, Icao};
    let f = Faker::new();
    let a = f.aviation();
    assert_matches(FLIGHT_ICAO_CODE, &a.flight_of(Icao).unwrap());
    for name in ["ICAO", "icao", "Icao", "IcaO"] {
        assert_matches(FLIGHT_ICAO_CODE, &a.flight_by_name(name).unwrap());
    }
    assert_matches(FLIGHT_IATA_CODE, &a.flight_of(Iata).unwrap());
    for name in ["IATA", "iata", "test", "null"] {
        assert_matches(FLIGHT_IATA_CODE, &a.flight_by_name(name).unwrap());
    }
    assert_matches(FLIGHT_IATA_CODE, &a.flight().unwrap());
}

/// `AviationTest.airportCode`, `airportICAOCode`, `airportIATACode`, and the two list cases with a code type.
#[test]
fn aviation_airports() {
    use datafaker::providers::AviationCodeType::{Iata, Icao};
    let f = Faker::new();
    let a = f.aviation();
    let airport = a.airport().unwrap();
    assert_eq!(airport.len(), 4);
    assert_matches("[A-Z0-9]{4}", &airport);
    assert_matches("[A-Z]{4}", &a.airport_of(Icao).unwrap());
    assert_matches("[A-Z]{3}", &a.airport_of(Iata).unwrap());
    common::provider_list(&f, "aviation.airport_iata", &a.airport_of(Iata).unwrap(), "[A-Z]{3}");
    common::provider_list(&f, "aviation.airport_icao", &a.airport_of(Icao).unwrap(), "[A-Z]{4}");
}

/// `CompassTest`: the three `@EnumSource(CompassPoint.class)` tests.
#[test]
fn compass_with_compass_point() {
    for point in datafaker::providers::CompassPoint::ALL {
        let f = Faker::new();
        assert!(!f.compass().compass_point(point).word().unwrap().is_empty());
        assert!(!f.compass().compass_point(point).abbreviation().unwrap().is_empty());
        assert!(!f.compass().compass_point(point).azimuth().unwrap().is_empty());
    }
}

/// Upstream's provider cache: a point chosen once stays on the faker's `Compass` (expressions included).
#[test]
fn compass_point_stays_on_the_faker() {
    use datafaker::providers::CompassPoint;
    let f = Faker::new();
    f.compass().compass_point(CompassPoint::Cardinal);
    for _ in 0..20 {
        assert!(["N", "E", "S", "W"].contains(&f.compass().abbreviation().unwrap().as_str()));
        assert!(["N", "E", "S", "W"].contains(&f.expression("#{Compass.abbreviation}").unwrap().as_str()));
    }
    assert!(Faker::new().compass().abbreviation().is_ok());
}

const WORDS_MATCH: &str = r"^[a-zA-Z\d_/ -]*$";

/// `VehicleTest.testVin`, `testModelWithParams`, `testCarOptions`, `testCarOptionsMinMax`,
/// `testStandardSpecsMinMax`, `testStandardSpecs`, `testLicensePlateWithParam`, `testLicensePlateWithParam_Canada`
/// (10 times each).
#[test]
fn vehicle_tests() {
    let f = Faker::new();
    let v = f.vehicle();
    let canada = faker_in("en", "CA");
    let vin_regex = r"([A-HJ-NPR-Z0-9]){3}[A-HJ-NPR-Z0-9]{5}[A-HJ-NPR-Z0-9]{1}[A-HJ-NPR-Z0-9]{1}[A-HJ-NPR-Z0-0]{1}[A-HJ-NPR-Z0-9]{1}\d{5}";
    for _ in 0..10 {
        assert_matches(vin_regex, &v.vin().unwrap());
        assert_matches(r"\P{Cc}+", &v.model_of("Toyota").unwrap());
        assert!((5..=10).contains(&v.car_options().unwrap().len()));
        let options = v.car_options_between(11, 12).unwrap();
        assert!((11..=12).contains(&options.len()));
        assert!((13..=14).contains(&v.standard_specs_between(13, 14).unwrap().len()));
        assert!((5..=10).contains(&v.standard_specs().unwrap().len()));
        assert_matches(WORDS_MATCH, &v.license_plate_of("GA").unwrap().unwrap());
        assert_matches(WORDS_MATCH, &v.license_plate_of("AL").unwrap().unwrap());
        assert_matches(WORDS_MATCH, &canada.vehicle().license_plate_of("MB").unwrap().unwrap());
        assert_matches(WORDS_MATCH, &canada.vehicle().license_plate_of("ON").unwrap().unwrap());
    }
    assert_eq!(v.license_plate_of("").unwrap(), None);
}

/// `VehicleLocaleTest`: transmissions and fuel types in every international faker, and the Ukrainian and Irish
/// plates (10 times each).
#[test]
fn vehicle_locale_tests() {
    for f in common::international_fakers() {
        common::provider_list(&f, "vehicle.transmissions", &f.vehicle().transmission().unwrap(), r"[\w\s]{3,15}");
        common::provider_list(&f, "vehicle.fuel_types", &f.vehicle().fuel_type().unwrap(), r"[\w\d\-\/\s]{3,30}");
    }
    let (ua, ie) = (faker_in("uk", "UA"), faker_in("en", "IE"));
    for _ in 0..10 {
        assert_matches(
            r"^(AA|KA|AB|KB|AC|KC|AE|KE|AH|KH|AI|KI|AK|KK|AM|KM|AO|KO|AP|KP|AT|KT|AX|KX|BA|HA|BB|HB|BC|HC|BE|HE|BH|HH|BI|HI|BK|HK|BM|HM|BO|HO|BT|HT|BX|HX|CA|IA|CB|IB|CE|IE|CH|IH)\d{4}[A-Z]{2}$",
            &ua.vehicle().license_plate().unwrap(),
        );
        assert_matches("[0-9]{2}[1|2]-[A-Z]{1,2}-[0-9]{1,6}", &ie.vehicle().license_plate().unwrap());
    }
}

/// `AwsTest.testRegion`; `AzureTest.testRegion` (its long alternation is read from the test source).
#[test]
fn aws_and_azure_regions() {
    let f = Faker::new();
    assert_matches(r"^[a-z]{2}-(south|east|north|west|northeast|central|southeast)-\d$", &f.aws().region().unwrap());
    let azure = common::java_source_enum("providers/base/AzureTest.java", r#"(region)\)\.matches\("(\([^"]+\))"\)"#);
    assert_eq!(azure.len(), 1);
    assert_matches(&azure[0].1, &f.azure().region().unwrap());
}

/// `DomainTest`: every method's dot-separated parts are non-empty (`testValidDomainNotNull` 10 times).
#[test]
fn domain_tests() {
    let f = Faker::new();
    let d = f.domain();
    for domain in [
        d.first_level_domain("example").unwrap(),
        d.second_level_domain("example").unwrap(),
        d.full_domain("example").unwrap(),
        d.valid_domain("example").unwrap(),
    ] {
        assert!(domain.split('.').all(|part| !part.is_empty()), "{domain}");
    }
    for _ in 0..10 {
        assert!(!d.valid_domain("example").unwrap().is_empty());
    }
}

/// `FileTest.testExtension` (10 times), `testFileNameSpecifyExtension`, `testFileNameSpecifyDir`,
/// `testFileNameSpecifySeparator`, `testFileNameSpecifyName`.
#[test]
fn file_tests() {
    let f = Faker::new();
    let file = f.file();
    for _ in 0..10 {
        assert_matches(
            "(flac|mp3|wav|bmp|gif|jpeg|jpg|png|tiff|css|csv|html|js|json|txt|mp4|avi|mov|webm|doc|docx|xls|xlsx|ppt|pptx|odt|ods|odp|pages|numbers|key|pdf)",
            &file.extension().unwrap(),
        );
    }
    assert_matches(r"([a-z\-_]+)([\\/])([a-z\-_]+)\.txt", &file.file_name_with(None, None, Some("txt"), None).unwrap());
    assert_matches(r"my_dir([\\/])([a-z\-_]+)\.([a-z0-9]+)", &file.file_name_with(Some("my_dir"), None, None, None).unwrap());
    assert_matches(r"([a-z\-_]+)\\([a-z\-_]+)\.([a-z0-9]+)", &file.file_name_with(None, None, None, Some("\\")).unwrap());
    assert_matches(r"([a-z\-_]+)([\\/])da_name\.([a-z0-9]+)", &file.file_name_with(None, Some("da_name"), None, None).unwrap());
}

/// `HttpTest`: the codes (`statusCode` 20 times, the categories 10 times), `statusCodeWithReason`,
/// `userAgentForAllBrowsers`, `responseBodyForContentType` (`@CsvSource`),
/// `responseBodyUnknownContentTypeFallsBackToJson`, and the list cases with a content type.
#[test]
fn http_tests() {
    use datafaker::providers::Browser;
    let f = Faker::new();
    let http = f.http();
    for _ in 0..20 {
        assert!((100..=599).contains(&http.status_code().unwrap()));
    }
    for _ in 0..10 {
        assert!((100..=199).contains(&http.informational().unwrap()));
        assert!((200..=299).contains(&http.successful().unwrap()));
        assert!((300..=399).contains(&http.redirect().unwrap()));
        assert!((400..=499).contains(&http.client_error().unwrap()));
        assert!((500..=599).contains(&http.server_error().unwrap()));
        let entry = http.status_code_with_reason().unwrap();
        assert_matches(r"\d{3} .+", &entry);
        assert!((100..=599).contains(&entry[..3].parse::<i32>().unwrap()));
    }
    for browser in Browser::ALL {
        let agent = http.user_agent_of(browser).unwrap();
        assert!(!agent.trim().is_empty() && agent.starts_with("Mozilla/"), "{browser:?}: {agent}");
    }
    let cases = [
        ("application/json", "{"),
        ("application/ld+json", "{"),
        ("application/vnd.api+json", "{"),
        ("application/problem+json", "{"),
        ("application/xml", "<?xml"),
        ("text/xml", "<?xml"),
        ("text/html", "<!DOCTYPE html>"),
        ("text/html; charset=utf-8", "<!DOCTYPE html>"),
        ("text/plain", ""),
        ("text/csv", ""),
        ("application/javascript", ""),
        ("text/javascript", ""),
        ("text/css", ""),
        ("application/graphql", "{"),
        ("text/markdown", ""),
    ];
    for (content_type, prefix) in cases {
        let body = http.response_body_for(content_type).unwrap();
        assert!(!body.trim().is_empty() && body.starts_with(prefix), "{content_type}: {body}");
    }
    assert!(http.response_body_for("application/unknown").unwrap().starts_with('{'));
    for (content_type, key) in [
        ("application/json", "json"),
        ("application/xml", "xml"),
        ("text/html", "html"),
        ("text/plain", "plain"),
        ("text/csv", "csv"),
        ("application/javascript", "javascript"),
        ("text/css", "css"),
        ("application/graphql", "graphql"),
        ("text/markdown", "markdown"),
    ] {
        common::provider_list(&f, &format!("http.response_body.{key}"), &http.response_body_for(content_type).unwrap(), "");
    }
}

/// `PlanetTest.shouldHaveNinePlanets` and the nine `shouldReturnCorrect…Metadata` tests, whose calls and expected
/// values are read from the test source and run through `Planet`'s method table.
#[test]
fn planet_tests() {
    use datafaker::JValue;
    use datafaker::dispatch::Target;
    use datafaker::providers::{PlanetName, ProviderKind};
    let f = Faker::new();
    assert_eq!(PlanetName::ALL.len(), 9);
    assert_eq!(f.fake_values_service().fetch_object("planet.name").unwrap().as_list().unwrap().len(), 9);
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/upstream/datafaker/src/test/java/net/datafaker/providers/base/PlanetTest.java")).unwrap();
    let re = regex::Regex::new(r#"planet\.(\w+)\(([A-Z]+)\);\s*assertThat\(\w+\)\.isEqualTo\("([^"]*)"\)"#).unwrap();
    let mut count = 0;
    for c in re.captures_iter(&src) {
        let (method, planet, expected) = (&c[1], &c[2], &c[3]);
        let m = Target::Provider(ProviderKind::Planet).methods().iter().find(|m| m.name == method && m.params.len() == 1).unwrap();
        let constant: &'static str = Box::leak(planet.to_string().into_boxed_str());
        let got = (m.invoke)(&f, &[JValue::Enum(constant)]).unwrap();
        assert_eq!(got, JValue::str(expected), "{method}({planet})");
        count += 1;
    }
    assert_eq!(count, 45);
}

/// `SipTest`: ports, response codes, the SDP body, the body bytes and the name address.
#[test]
fn sip_tests() {
    let f = Faker::new();
    let sip = f.sip();
    assert!((1000..=10000).contains(&sip.messaging_port().unwrap()));
    let rtp = sip.rtp_port().unwrap();
    assert!(rtp >= 2 && rtp % 2 == 0);
    assert!((100..=200).contains(&sip.provisional_response_code().unwrap()));
    assert!((200..=300).contains(&sip.success_response_code().unwrap()));
    assert!((300..=400).contains(&sip.redirect_response_code().unwrap()));
    assert!((400..=500).contains(&sip.client_error_response_code().unwrap()));
    assert!((500..=600).contains(&sip.server_error_response_code().unwrap()));
    assert!((600..=700).contains(&sip.global_error_response_code().unwrap()));
    let body = sip.body_string().unwrap();
    let lines: Vec<&str> = body.split('\n').collect();
    assert_eq!(lines.len(), 7);
    assert_eq!(lines[0], "v=0");
    let second: Vec<&str> = lines[1].split(' ').collect();
    assert!(second[0].starts_with("o="));
    assert_matches("^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$", second[1]);
    assert_matches(r"[a-z]+\.\w{2,4}", second[second.len() - 1]);
    assert_eq!(lines[2], "s=-");
    let fourth: Vec<&str> = lines[3].split(' ').collect();
    assert_eq!(fourth[0], "c=IN");
    assert_matches(r"^\d{1,3}.\d{1,3}.\d{1,3}.\d{1,3}$", fourth[fourth.len() - 1]);
    assert_eq!(lines[4], "t=0 0");
    let sixth: Vec<&str> = lines[5].split(' ').collect();
    assert_eq!(sixth[0], "m=audio");
    let port: i32 = sixth[1].parse().unwrap();
    assert!(port >= 2 && port % 2 == 0);
    assert_eq!(lines[6], "a=rtpmap:0 PCMU/8000");
    assert!(!sip.body_bytes().unwrap().is_empty());
    let address = sip.name_address().unwrap();
    let (user, host) = address.split_once('@').unwrap();
    assert_matches(r"\w+", user.split(':').nth(1).unwrap());
    assert_matches(r"^\d{1,3}.\d{1,3}.\d{1,3}.\d{1,3}$", host.split(':').next().unwrap());
}

fn unique_faker() -> Faker {
    let f = common::test_locale_faker();
    f.add_path(&Locale::new("test", "EE"), concat!(env!("CARGO_MANIFEST_DIR"), "/tests/resources/test_EE.yml")).unwrap();
    f
}

const UNIQUE_VALUES: [&str; 5] = ["firstValue", "secondValue", "thirdValue", "fourthValue", "fifthValue"];

/// `UniqueTest`. `fetchFromYaml_shouldReturnValuesInRandomOrderUsingRandomService` verifies the
/// `nextInt(0, 4)` … `nextInt(0, 0)` calls with Mockito; here a seeded faker's draws are compared with those calls
/// made on a generator of the same seed.
#[test]
fn unique_tests() {
    let seeded = |seed| {
        let locale = Locale::language_only("test");
        let f = Faker::with_locale_and_seed(locale.clone(), seed);
        f.add_path(&locale, concat!(env!("CARGO_MANIFEST_DIR"), "/tests/resources/test.yml")).unwrap();
        f
    };
    let f = seeded(5);
    let results: Vec<String> = (0..5).map(|_| f.unique().fetch_from_yaml("unique.values").unwrap()).collect();
    let set: std::collections::HashSet<&str> = results.iter().map(String::as_str).collect();
    assert_eq!(set, UNIQUE_VALUES.into_iter().collect());
    let random = datafaker::RandomService::with_seed(5);
    let mut values: Vec<&str> = UNIQUE_VALUES.to_vec();
    let expected: Vec<&str> = (0..5).rev().map(|n| values.swap_remove(random.next_int_between(0, n).unwrap() as usize)).collect();
    assert_eq!(results, expected);

    let f = unique_faker();
    assert_eq!(f.unique().fetch_from_yaml("unique.single-value").unwrap(), "theOnlyValue");
    assert_eq!(
        f.unique().fetch_from_yaml("unique.single-value").unwrap_err().message(),
        "All possible values have been generated for key unique.single-value under locale test"
    );
    for key in ["unique.first-same-locale-value", "unique.second-same-locale-value"] {
        assert_eq!(f.unique().fetch_from_yaml(key).unwrap(), "theSameValue");
    }
    f.do_with_locale(Locale::language_only("test_EE"), |f| {
        for key in ["unique.first-same-locale-value", "unique.second-same-locale-value"] {
            assert_eq!(f.unique().fetch_from_yaml(key).unwrap(), "theSameValue");
        }
    });
    for key in ["unique.nonexistent-values", "unique", "unique.list-of-lists"] {
        assert_eq!(f.unique().fetch_from_yaml(key).unwrap_err().message(), format!("No values found for key {key}"));
    }
    let f = unique_faker();
    for _ in 0..UNIQUE_VALUES.len() {
        f.unique().fetch_from_yaml("unique.values").unwrap();
    }
    assert!(UNIQUE_VALUES.contains(&f.resolve("unique.values").unwrap().as_str()));
    assert_eq!(f.unique().fetch_from_yaml("unique.valid-integer").unwrap(), "123");
    assert_eq!(f.unique().fetch_from_yaml("unique.valid-decimal").unwrap(), "12.34");
    assert_eq!(f.unique().fetch_from_yaml("unique.valid-boolean").unwrap(), "true");
}

const BRAZILIAN: &str = r"\d{3}\.\d{3}\.\d{3}-\d{2}";

/// `CPFTest` (100 times each), and `idnumbers/pt/br/IdNumberGeneratorPtBrUtilTest.samples`.
#[test]
fn cpf_tests() {
    use datafaker::idnumbers::pt_br::is_cpf_valid;
    let f = Faker::new();
    for _ in 0..100 {
        assert!(is_cpf_valid(&f.cpf().valid().unwrap()));
        assert!(!is_cpf_valid(&f.cpf().invalid().unwrap()));
        assert_matches(BRAZILIAN, &f.cpf().valid().unwrap());
        assert_matches(BRAZILIAN, &f.cpf().valid_with(true).unwrap());
        assert_matches(BRAZILIAN, &f.cpf().invalid().unwrap());
        assert_matches(BRAZILIAN, &f.cpf().invalid_with(true).unwrap());
    }
    for sample in ["529.982.247-25", "168.995.350-09", "862.883.667-57", "746.971.314-01"] {
        assert!(is_cpf_valid(sample), "{sample}");
    }
}

/// `CNPJTest` (`isValidCNPJ`, `isInvalidCNPJ` and the invalid multi-branch case 10 times each).
#[test]
fn cnpj_tests() {
    use datafaker::idnumbers::pt_br::is_cnpj_valid;
    let f = Faker::with_locale(Locale::language_only("pt_BR"));
    let cnpj = f.cnpj();
    for _ in 0..10 {
        assert!(is_cnpj_valid(&cnpj.valid().unwrap()));
        assert!(!is_cnpj_valid(&cnpj.invalid().unwrap()));
    }
    let mut value = cnpj.valid_branch(true, true).unwrap();
    for _ in 0..5 {
        if &value[11..15] != "0001" {
            break;
        }
        value = cnpj.valid_branch(true, true).unwrap();
    }
    assert!(value[11..15].parse::<i32>().unwrap() > 1);
    assert!(is_cnpj_valid(&value));
    for _ in 0..10 {
        let mut value = cnpj.invalid_branch(true, true).unwrap();
        let mut i = 0;
        while &value[11..15] == "0001" && i < 5 || &value[11..15] == "0000" {
            value = cnpj.invalid_branch(true, true).unwrap();
            i += 1;
        }
        assert!(value[11..15].parse::<i32>().unwrap() > 1, "{value}");
        assert!(!is_cnpj_valid(&value));
    }
    is_cnpj_valid("57.615.644/1633-29");
    let format = r"(^\d{2}\x2E\d{3}\x2E\d{3}\x2F\d{4}\x2D\d{2}$)";
    assert_matches(format, &cnpj.valid().unwrap());
    assert_matches(format, &cnpj.valid_with(true).unwrap());
    assert_matches(format, &cnpj.invalid().unwrap());
    assert_matches(format, &cnpj.invalid_with(true).unwrap());
}

/// `DurationTest`: each `atMost…` lies between zero and its maximum.
#[test]
fn duration_tests() {
    use datafaker::java::Duration;
    let f = Faker::new();
    let d = f.duration();
    let check = |got: Duration, max: Duration| assert!(Duration::ZERO <= got && got <= max, "{got}");
    check(d.at_most_seconds(55).unwrap(), Duration::of_seconds(55));
    check(d.at_most_minutes(45).unwrap(), Duration::of_minutes(45).unwrap());
    check(d.at_most_hours(35).unwrap(), Duration::of_hours(35).unwrap());
    check(d.at_most_days(40).unwrap(), Duration::of_days(40).unwrap());
}

/// `InternetPasswordTest`: `password(8, 16, true, true, true)` always has a digit and a special character
/// (`testPassword1000` 1,000 times, `passwordSpecial` and `passwordMix` 10 times).
#[test]
fn internet_password_tests() {
    let f = Faker::new();
    let special = |p: &str| p.chars().any(|c| !c.is_ascii_alphanumeric());
    let digit = |p: &str| p.chars().any(|c| c.is_ascii_digit());
    for _ in 0..1020 {
        let password = f.internet().password_with(8, 16, true, true, true).unwrap();
        assert!(special(&password) && digit(&password), "{password}");
    }
}

/// `RandomFakerTest`: re-seeding the faker's `Random` repeats `numerify`, `letterify`, `name().name()` and
/// `internet().emailAddress()`.
#[test]
fn random_faker_tests() {
    let f = Faker::with_locale_and_random(Locale::english(), datafaker::RandomService::new(datafaker::java::Random::from_entropy()));
    let reset = || f.random().random().as_random().expect("a java.util.Random").set_seed(10);
    let twice = |g: &dyn Fn() -> String| {
        reset();
        let first = g();
        reset();
        assert_eq!(first, g());
    };
    twice(&|| f.numerify("###"));
    twice(&|| f.letterify("???"));
    twice(&|| f.name().name().unwrap());
    twice(&|| f.internet().email_address().unwrap());
}
