//! `FakerTest`, `Issue1178Test` and `Issue759Test` (Datafaker 2.7.0, `net.datafaker`).
//!
//! Adapted where a test reaches a provider of a later tranche: `issue883Test` resolves the YAML key
//! `ancient.god` where upstream calls `Ancient.god()`. `doWithLocaleExceptionTest` and the exception half of
//! `differentSeeds` check that an error returned inside `doWith` comes out of it (Java wraps a checked exception in a
//! `RuntimeException`; Rust closures return their error). `whitelistTest`'s predicates see the provider class's
//! binary name (`Faker::with_white_list`). `Issue759Test` runs its five threads of 20,000 iterations once per repetition, as upstream,
//! with the 12-second limit checked after the threads join.

mod common;

use std::collections::HashSet;

use common::{assert_matches, matches};
use datafaker::dispatch::Target;
use datafaker::providers::ProviderKind;
use datafaker::{Error, Faker, Locale};

fn faker() -> Faker {
    Faker::new()
}

#[test]
fn examplify_uppercase_letters() {
    assert_matches("[A-Z]{3}", &faker().examplify("ABC"));
}

#[test]
fn examplify_lowercase_letters() {
    assert_matches("[a-z]{3}", &faker().examplify("abc"));
}

#[test]
fn examplify_numbers() {
    assert_matches("[0-9]{6}", &faker().examplify("489321"));
}

#[test]
fn examplify_mixed() {
    assert_matches("[a-z]{3}[0-9]{3}[A-Z]{3}[0-9][a-z]{3}", &faker().examplify("abc123ABC1zzz"));
}

#[test]
fn examplify_with_spaces_and_special_characters() {
    assert_matches("[A-Z][a-z]{2} [a-z]{6} [0-9]!", &faker().examplify("The number 4!"));
}

#[test]
fn bothify_should_generate_letters_and_numbers() {
    assert_matches("\\w{4}\\d{2}@gmail.com", &faker().bothify("????##@gmail.com"));
}

#[test]
fn letterify_should_generate_letters() {
    assert_matches("\\w{4}", &faker().bothify("????"));
}

#[test]
fn letterify_should_generate_upper_case_letters() {
    assert_matches("[A-Z]{4}", &faker().bothify_with("????", true));
}

#[test]
fn letterify_should_leave_non_special_characters_alone() {
    assert_matches("ABC\\w{4}DEF", &faker().bothify("ABC????DEF"));
}

#[test]
fn numerify_test() {
    let cases = [
        ("Ø###", "[1-9]\\d{3}"),
        ("ØØ", "[1-9]{2}"),
        ("####123", "\\d{4}123"),
        ("####", "\\d{4}"),
        ("#############", "\\d{13}"),
        ("################", "\\d{16}"),
        ("####################", "\\d{20}"),
    ];
    for (input, expected) in cases {
        assert_matches(expected, &faker().numerify(input));
    }
}

#[test]
fn templatify() {
    let f = faker();
    let utf16_len = |s: String| s.encode_utf16().count();
    assert_eq!(utf16_len(f.templatify("12??34", '?', &["тест", "test", "测试测试"]).unwrap()), 12);
    let service = f.fake_values_service();
    assert_eq!(utf16_len(service.templatify("12??34", &[('1', &["тест", "test", "测试测试"])]).unwrap()), 9);
    assert_eq!(utf16_len(service.templatify("12??34", &[('1', &[""])]).unwrap()), 5);
}

/// Issue 1091.
#[test]
fn test_regexify() {
    let inputs = [
        "\\d",
        "\\w",
        "[aeiou]{2,3}",
        "[a-z]{2,3}",
        "a{2,3}",
        "a{2}",
        "a*b+c?",
        "[a-z1-9]",
        "[a-z]{2,3}[0-9]{2,3}",
        "(a|b){2,3}",
        "[a-z]",
        "(aeiou)",
        "(a|b)",
        "\\.\\*\\?\\+",
        "^arn:.+:.+:.*:([0-9]{12}):(.+)$",
        "\\s[\\s]\\d[\\d]\\w[\\w]",
    ];
    for input in inputs {
        let value = Faker::new().regexify(input).unwrap();
        assert!(matches(input, &value), "{input} → {value:?}");
    }
}

fn assert_runtime_failure(r: datafaker::Result<String>) {
    let e = r.unwrap_err();
    assert!(["RuntimeException", "IllegalArgumentException"].contains(&e.simple_class()), "{e:?}");
}

#[test]
fn bad_expression_too_many_args() {
    assert_runtime_failure(faker().expression("#{regexify 'a','a'}"));
}

#[test]
fn bad_expression_too_few_args() {
    assert_runtime_failure(faker().expression("#{regexify}"));
}

#[test]
fn bad_expression_could_not_coerce() {
    assert_runtime_failure(faker().expression("#{number.number_between 'x','10'}"));
}

#[test]
fn expression() {
    let f = faker();
    let e = |x: &str| f.expression(x).unwrap();
    assert_matches("([abcd])", &e("#{options.option 'a','b','c','d'}"));
    assert_eq!(e("#{options.option 'single'}"), "single");
    assert_matches("(12|345|89|54321)", &e("#{options.option '12','345','89','54321'}"));
    assert_matches("([ab]){2,3}", &e("#{regexify '(a|b){2,3}'}"));
    assert_matches("\\.\\*\\?\\+", &e("#{regexify '\\.\\*\\?\\+'}"));
    assert_matches("[A-Z]{4}", &e("#{bothify '????','true'}"));
    assert_matches("[a-z]{4}", &e("#{bothify '????','false'}"));
    assert_matches("[A-Z]{4}", &e("#{letterify '????','true'}"));
    assert_matches("([12qr]){4}", &e("#{templatify '????','?','1','2','q','r'}"));
    assert_matches("[a-zA-Z']+ [a-zA-Z']+ [a-zA-Z']+", &e("#{Name.first_name} #{Name.first_name} #{Name.last_name}"));
    assert_matches("[1-9]\\d*", &e("#{number.number_between '1','10'}"));
    assert_matches("[1-9](\\.[0-9]+)?", &e("#{number.number_between '1','10.0'}"));
    assert_matches("[1-9](\\.[0-9]+)?", &e("#{number.number_between '1.0','10'}"));
    assert_matches("[a-z\\s]+", &e("#{color.name}"));
    assert_matches("[0-9]{2}/[0-9]{2}/[0-9]{4} [0-9]{2}:[0-9]{2}:[0-9]{2}", &e("#{date.past '15','SECONDS','dd/MM/yyyy hh:mm:ss'}"));
    assert_matches("[0-9]{2} [0-9]{3} [0-9]{2}:[0-9]{2}:[0-9]{2}", &e("#{date.birthday 'yy DDD hh:mm:ss'}"));
}

#[test]
fn json_expression_test() {
    let f = faker();
    let json = f
        .expression(
            "#{json 'person','#{json ''first_name'',''#{Name.first_name}'',''last_name'',''#{Name.last_name}''}','address','#{json ''country'',''#{Address.country}'',''city'',''#{Address.city}''}'}",
        )
        .unwrap();
    assert!(json.contains("\"address\": {\"country\":"), "{json}");
    let jsona = f
        .expression(concat!(
            "#{jsona '-1','person',",
            "'#{json ''first_name'',''#{Name.first_name}'',''last_name'',''#{Name.last_name}''}',",
            " '2','addesses',",
            "'#{json ''address'',",
            "''#{json ''''country'''',''''#{Address.country}'''',''''city'''',''''#{Address.city}''''}''}'}"
        ))
        .unwrap();
    assert!(jsona.contains("\"addesses\": [{\"address\": {\"country\": "), "{jsona}");
}

#[test]
fn test_limit_for_csv_expression() {
    for limit in [0, 2, 3, 10, 20, 100] {
        let f = faker();
        let full = f
            .expression(&format!("#{{csv ';','\"','false','{limit}','first_name','#{{Name.first_name}}','last_name','#{{Name.last_name}}'}}"))
            .unwrap();
        let short = f.expression(&format!("#{{csv '{limit}','first_name','#{{Name.first_name}}','last_name','#{{Name.last_name}}'}}")).unwrap();
        // Upstream starts both counts from whether the full text is empty.
        let start = usize::from(!full.is_empty());
        assert_eq!(start + full.matches('\n').count(), limit, "full, limit {limit}");
        assert_eq!(start + short.matches('\n').count(), limit + 1, "short, limit {limit}"); // + header
    }
}

#[test]
fn number_between_repeated() {
    let f = faker();
    for _ in 0..100 {
        assert_matches("[1-9](\\.[0-9]+)?", &f.expression("#{number.number_between '1','10'}").unwrap());
    }
}

#[test]
fn regexify_should_generate_same_value_for_faker_with_same_seed() {
    let first = Faker::with_seed(1).regexify("\\d").unwrap();
    let second = Faker::with_seed(1).regexify("\\d").unwrap();
    assert_eq!(first, second);
}

#[test]
fn resolve_should_return_value_that_exists() {
    assert!(!faker().resolve("address.city_prefix").unwrap().is_empty());
}

#[test]
fn resolve_should_throw_exception_when_property_doesnt_exist() {
    assert_eq!(faker().resolve("address.nothing").unwrap_err().simple_class(), "RuntimeException");
}

/// Issue 87: the same expression twice gives different values.
#[test]
fn datafaker87() {
    for expression in ["#{regexify '[a-z]{5}[A-Z]{5}'}", "#{Address.city}"] {
        let f = faker();
        let n = 10;
        let same = (0..n).filter(|_| f.expression(expression).unwrap() == f.expression(expression).unwrap()).count();
        assert!(same < n, "{expression}");
    }
}

#[test]
fn faker_instance_can_be_acquired_via_utility_methods() {
    Faker::new();
    Faker::with_locale(Locale::new("en", "CA"));
    Faker::with_seed(1);
    Faker::with_locale_and_seed(Locale::new("zh", "CN"), 2);
}

#[test]
fn different_locales_test() {
    let local_faker = Faker::new();
    let first_name = |f: &Faker| f.name().first_name().unwrap();
    local_faker.do_with_locale(Locale::new("ru", "RU"), first_name);
    local_faker.do_with_locale(Locale::language_only("de"), first_name);
    local_faker.do_with_locale(Locale::new("zh", "CN"), first_name);
    for _ in 0..10 {
        assert_matches("[а-яА-ЯЁё ]+", &local_faker.do_with_locale(Locale::new("ru", "RU"), first_name));
    }
}

#[test]
fn issue883_test() {
    for i in 0..10_000 {
        let f = Faker::new();
        f.resolve("ancient.god").unwrap();
        if i % 1_000 == 0 {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

#[test]
fn do_with_locale_exception_test() {
    let local_faker = Faker::new();
    let r: datafaker::Result<String> = local_faker.do_with_locale(Locale::new("ja", "JP"), |_| Err(Error::runtime("おっと")));
    assert_eq!(r.unwrap_err().message(), "おっと");
    assert_eq!(local_faker.locale(), Locale::english());
}

#[test]
fn different_seeds() {
    let local_faker = Faker::new();
    let first_name = |f: &Faker| f.name().first_name().unwrap();
    assert_eq!(local_faker.do_with_seed(123, first_name), local_faker.do_with_seed(123, first_name));
    let at987 = local_faker.do_with_seed(987, first_name);
    assert_ne!(at987, local_faker.do_with_seed(123, first_name));
    assert_eq!(at987, local_faker.do_with_seed(987, first_name));
    let r: datafaker::Result<String> = local_faker.do_with_seed(123, |_| Err(Error::runtime("Oops1")));
    assert!(r.is_err());
    let canada = || Locale::new("en", "CA");
    assert_eq!(local_faker.do_with_locale_and_seed(canada(), 123, first_name), local_faker.do_with_locale_and_seed(canada(), 123, first_name));
    let at987 = local_faker.do_with_locale_and_seed(canada(), 987, first_name);
    assert_ne!(at987, local_faker.do_with_locale_and_seed(canada(), 123, first_name));
    assert_eq!(at987, local_faker.do_with_locale_and_seed(canada(), 987, first_name));
    let r: datafaker::Result<String> = local_faker.do_with_locale_and_seed(Locale::english(), 123, |_| Err(Error::runtime("Oops2")));
    assert!(r.is_err());
}

/// The base providers' methods annotated `@Deterministic` upstream.
const DETERMINISTIC: &[(&str, &str)] =
    &[("Locality", "allSupportedLocales"), ("Address", "streetPrefix"), ("Finance", "ibanSupportedCountries"), ("Drone", "shutterSpeedUnits")];

/// Every public no-argument method of every ported provider gives more than one value in 100 calls, or exactly one
/// when it is `@Deterministic` upstream.
#[test]
fn test_deterministic_and_non_deterministic_providers_return_values() {
    let f = faker();
    for kind in ProviderKind::ALL {
        for m in Target::Provider(*kind).methods().iter().filter(|m| m.params.is_empty()) {
            let deterministic = DETERMINISTIC.contains(&(kind.simple_name(), m.name));
            let mut set = HashSet::new();
            for _ in 0..100 {
                if set.len() > 1 {
                    break;
                }
                set.insert((m.invoke)(&f, &[]).unwrap().to_java_string(&f));
            }
            if deterministic {
                assert_eq!(set.len(), 1, "Class: {}, method: {} should have the same return value", kind.simple_name(), m.name);
            } else {
                assert!(set.len() > 1, "Class: {}, method: {} should generate different return values", kind.simple_name(), m.name);
            }
        }
    }
}

// Issue1178Test

#[test]
fn test_expression_english_fails() {
    assert!(!Faker::with_locale(Locale::english()).expression("#{name.first_name}").unwrap().trim().is_empty());
}

#[test]
fn test_expression_fails() {
    assert!(!Faker::new().expression("#{name.first_name}").unwrap().trim().is_empty());
}

#[test]
fn test_expression_us_fails() {
    assert!(!Faker::with_locale(Locale::new("en", "US")).expression("#{name.first_name}").unwrap().trim().is_empty());
}

#[test]
fn test_expression_au_works() {
    assert!(!Faker::with_locale(Locale::new("en", "AU")).expression("#{name.first_name}").unwrap().trim().is_empty());
}

#[test]
fn test_expression_nl_works() {
    assert!(!Faker::with_locale(Locale::new("nl", "nl")).expression("#{name.first_name}").unwrap().trim().is_empty());
}

#[test]
fn test_expression_nl_with_middle_works() {
    assert!(!Faker::with_locale(Locale::new("nl", "nl")).expression("#{name.name_with_middle}").unwrap().trim().is_empty());
}

#[test]
fn test_expression_nl_with_city() {
    assert!(!Faker::new().expression("#{address.city}").unwrap().trim().is_empty());
}


#[test]
fn test_expression_nl_with_full_address() {
    let faker = Faker::with_locale(Locale::new("nl", "nl"));
    assert!(!faker.expression("#{address.full_address}").unwrap().trim().is_empty());
}

/// `Issue759Test.fakeSomeData`: `None` when it went as expected (a county, or "County is not configured").
fn fake_some_data(faker: &Faker) -> Option<String> {
    let result = (|| {
        let state = faker.address().state_abbr()?;
        let zip_code = faker.address().zip_code_by_state(&state)?;
        let county = faker.address().county_by_zip_code(&zip_code)?;
        Ok::<_, Error>((zip_code, county))
    })();
    match result {
        Ok((zip_code, county)) => (county == zip_code).then(|| format!("county {county} equals the zip code")),
        Err(e) if e.message().starts_with("County is not configured ") => None,
        Err(e) => Some(format!("{}: {}", e.simple_class(), e.message())),
    }
}

/// `Issue759Test.issue759Test` (`@RepeatedTest(10)`): five threads share one faker.
#[test]
fn issue759_test() {
    for _ in 0..10 {
        let faker = Faker::new();
        let start = std::time::Instant::now();
        let errors: Vec<String> = std::thread::scope(|scope| {
            let threads: Vec<_> = (0..5).map(|_| scope.spawn(|| (0..20000).filter_map(|_| fake_some_data(&faker)).collect::<Vec<_>>())).collect();
            threads.into_iter().flat_map(|t| t.join().expect("thread")).collect()
        });
        assert!(start.elapsed().as_secs() < 12, "Test did not complete within 12 second");
        assert!(errors.is_empty(), "{errors:?}");
    }
}

/// Issue 716: an expression cached without arguments does not answer one with arguments.
#[test]
fn should_not_apply_caching_to_methods_with_parameters() {
    let f = faker();
    assert_matches("[A-z0-9]{2}\\d{1,4}", &f.expression("#{Aviation.flight}").unwrap());
    assert_matches("[A-z]{3}\\d{1,4}", &f.expression("#{Aviation.flight 'ICAO'}").unwrap());
}

#[test]
fn whitelist_test() {
    let faker_without_white_list = Faker::new();
    let with = |p: fn(&str) -> bool| Faker::with_white_list(Locale::english(), datafaker::RandomService::default(), p);
    let faker_with_empty_white_list = with(|_| false);
    let faker_with_full_white_list = with(|_| true);
    let faker_with_without_name_in_white_list = with(|c| c != "net.datafaker.providers.base.Name");

    let e = faker_with_empty_white_list.expression("#{Address.country}").unwrap_err();
    assert_eq!(e.root_cause().message(), "Provider 'net.datafaker.providers.base.Address' is not in white list");

    faker_without_white_list.expression("#{Address.country}").unwrap();
    faker_with_full_white_list.expression("#{Address.country}").unwrap();
    faker_with_without_name_in_white_list.expression("#{Address.country}").unwrap();

    let e = faker_with_empty_white_list.expression("#{Name.fullName}").unwrap_err();
    assert_eq!(e.root_cause().message(), "Provider 'net.datafaker.providers.base.Name' is not in white list");
}

/// Not upstream: a provider reached through another one's YAML, or through `faker.name()` in an expression, is
/// checked too; the user's providers are checked when the faker creates them.
#[test]
fn whitelist_applies_on_every_expression_path() {
    let faker = Faker::with_white_list(Locale::english(), datafaker::RandomService::default(), |c| c != "net.datafaker.providers.base.Name");
    let e = faker.expression("#{name.full_name}").unwrap_err();
    assert_eq!(e.root_cause().message(), "Provider 'net.datafaker.providers.base.Name' is not in white list");
    assert!(faker.expression("#{Name.first_name}").is_err());
    assert!(faker.expression("#{Lorem.word}").is_ok());
}
