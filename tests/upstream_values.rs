//! `FakeValuesTest` and `FakeValuesGroupingTest` (Datafaker 2.7.0, `service/`).

use datafaker::yaml::{Map, Value};
use datafaker::{FakeValues, FakeValuesContext, FakeValuesGrouping, FakeValuesInterface, Locale, prefix_unqualified_expressions};

const PATH: &str = "address";

fn english() -> Locale {
    Locale::english()
}

fn address_values() -> &'static FakeValues {
    FakeValues::of(FakeValuesContext::of_file(&english(), "address.yml", Some(PATH)))
}

/// `Path.toUri().toURL()` of a path relative to the crate.
fn file_url(relative: &str) -> String {
    format!("file:{}", std::path::absolute(relative).unwrap().display())
}

// FakeValuesTest

#[test]
fn get_a_value_returns_a_value() {
    assert!(address_values().get(PATH).unwrap().is_some());
}

#[test]
fn get_a_value_does_not_return_a_value() {
    assert!(address_values().get("dog").unwrap().is_none());
}

#[test]
fn get_a_value_with_a_non_english_file() {
    let french = FakeValues::of(FakeValuesContext::of(&Locale::language_only("fr")));
    assert!(french.get(PATH).unwrap().is_some());
}

#[test]
fn get_a_value_for_hebrew_locale() {
    let hebrew = FakeValues::of(FakeValuesContext::of(&Locale::language_only("iw")));
    assert!(hebrew.get(PATH).unwrap().is_some());
}

#[test]
fn correct_path_for_hebrew_language() {
    let hebrew = FakeValues::of(FakeValuesContext::of(&Locale::language_only("iw")));
    assert_eq!(hebrew.paths().unwrap(), Some(vec!["he".to_string()]));
}

#[test]
fn incorrect_path_for_hebrew_language() {
    let hebrew = FakeValues::of(FakeValuesContext::of(&Locale::language_only("iw")));
    assert!(!hebrew.paths().unwrap().unwrap().contains(&"iw".to_string()));
}

#[test]
fn correct_locale() {
    let fv = FakeValues::of(FakeValuesContext::of(&Locale::language_only("uk")));
    assert_eq!(fv.locale(), &Locale::language_only("uk"));
}

#[test]
fn get_a_value_from_a_locale_that_cant_be_loaded() {
    let fake_values = FakeValues::of(FakeValuesContext::of(&Locale::language_only("nothing")));
    assert!(fake_values.get(PATH).unwrap().is_none());
}

/// Issue #1477: unqualified references in lists are rewritten when a URL is loaded.
#[test]
fn load_from_url_rewrites_unqualified_template_references_at_load_time() {
    let fake_values = FakeValues::of(FakeValuesContext::of_url(&english(), &file_url("resources/en/name.yml")));
    let Some(Value::Map(name_map)) = fake_values.get("name").unwrap() else { panic!("name provider map") };
    let Value::List(first_names) = &name_map["first_name"] else { panic!("a list") };
    let mut names: Vec<String> = first_names.iter().map(Value::to_java_string).collect();
    names.sort();
    assert_eq!(names, ["#{Name.femaleFirstName}", "#{Name.maleFirstName}"]);
}

fn strs(items: &[&str]) -> Value {
    Value::List(items.iter().map(|s| Value::Str(s.to_string())).collect())
}

// FakeValuesTest.PrefixUnqualifiedExpressions

#[test]
fn rewrites_list_items() {
    let cases: &[(&str, &[&str], &str, &[&str])] = &[
        ("rewrites simple unqualified expression in list", &["#{first_name}"], "name", &["#{Name.firstName}"]),
        ("rewrites multiple expressions inside one string", &["#{first_name} #{last_name}"], "name", &["#{Name.firstName} #{Name.lastName}"]),
        ("converts snake_case key to camelCase method name", &["#{some_long_yaml_key}"], "p", &["#{P.someLongYamlKey}"]),
        ("capitalizes provider name", &["#{x}"], "myProvider", &["#{MyProvider.x}"]),
        ("leaves already-qualified expressions untouched", &["#{Name.firstName}"], "address", &["#{Name.firstName}"]),
        ("leaves expressions with arguments untouched", &["#{numerify '#-####'}"], "code", &["#{numerify '#-####'}"]),
        ("leaves plain strings untouched", &["Aaron", "Bob", "Charlie"], "name", &["Aaron", "Bob", "Charlie"]),
        (
            "rewrites unqualified and preserves qualified in the same string",
            &["#{city_prefix} #{Name.firstName}#{city_suffix}"],
            "address",
            &["#{Address.cityPrefix} #{Name.firstName}#{Address.citySuffix}"],
        ),
        (
            "rewrites name.first_name redirect list (issue #1477)",
            &["#{female_first_name}", "#{male_first_name}"],
            "name",
            &["#{Name.femaleFirstName}", "#{Name.maleFirstName}"],
        ),
    ];
    for (description, input, provider, expected) in cases {
        let mut list = strs(input);
        prefix_unqualified_expressions(&mut list, provider);
        assert_eq!(list, strs(expected), "{description}");
    }
}

#[test]
fn recurses_into_nested_maps_and_rewrites_lists_at_any_depth() {
    let mut inner = Map::new();
    inner.insert("title".into(), strs(&["#{first_name}"]));
    let mut middle = Map::new();
    middle.insert("nested".into(), Value::Map(inner));
    let mut root = Map::new();
    root.insert("deeply".into(), Value::Map(middle));
    let mut root = Value::Map(root);
    prefix_unqualified_expressions(&mut root, "name");
    let Value::Map(root) = root else { unreachable!() };
    let Value::Map(middle) = &root["deeply"] else { unreachable!() };
    let Value::Map(inner) = &middle["nested"] else { unreachable!() };
    assert_eq!(inner["title"], strs(&["#{Name.firstName}"]));
}

#[test]
fn does_not_rewrite_scalar_strings_outside_lists() {
    let mut map = Map::new();
    map.insert("scalar".into(), Value::Str("#{first_name}".into()));
    map.insert("list".into(), strs(&["#{first_name}"]));
    let mut map = Value::Map(map);
    prefix_unqualified_expressions(&mut map, "name");
    let Value::Map(map) = map else { unreachable!() };
    assert_eq!(map["scalar"], Value::Str("#{first_name}".into()));
    assert_eq!(map["list"], strs(&["#{Name.firstName}"]));
}

#[test]
fn check_equals() {
    let canada = Locale::new("en", "CA");
    let of = |c: FakeValuesContext| Some(FakeValues::of(c));
    let tmp = file_url("tmp");
    let tmp2 = file_url("tmp2");
    let cases: Vec<(Option<&FakeValues>, Option<&FakeValues>, bool)> = vec![
        (of(FakeValuesContext::of(&canada)), of(FakeValuesContext::of(&canada)), true),
        (None, of(FakeValuesContext::of(&canada)), false),
        (of(FakeValuesContext::of(&canada)), None, false),
        (of(FakeValuesContext::of(&canada)), None, false),
        (of(FakeValuesContext::of(&english())), of(FakeValuesContext::of_file(&english(), "filepath", Some("path"))), false),
        (of(FakeValuesContext::of_file(&english(), "filepath", None)), of(FakeValuesContext::of_file(&english(), "filepath", Some("path"))), false),
        (
            of(FakeValuesContext::of_file(&english(), "filepath", Some("path"))),
            of(FakeValuesContext::of_file(&english(), "filepath", Some("path"))),
            true,
        ),
        (
            of(FakeValuesContext::of_file_url(&english(), "filepath", Some("path"), Some(&tmp))),
            of(FakeValuesContext::of_file_url(&english(), "filepath", Some("path"), Some(&tmp))),
            true,
        ),
        (
            of(FakeValuesContext::of_file_url(&english(), "filepath", Some("path"), Some(&tmp2))),
            of(FakeValuesContext::of_file_url(&english(), "filepath", Some("path"), Some(&tmp))),
            false,
        ),
    ];
    for (i, (a, b, equals)) in cases.into_iter().enumerate() {
        assert_eq!(a == b, equals, "case {i}");
    }
}

// FakeValuesGroupingTest

#[test]
fn handles_one_fake_value() {
    let grouping = FakeValuesGrouping::with(address_values()).unwrap();
    let got = grouping.get("address").unwrap();
    assert!(got.is_some());
    assert_eq!(got, address_values().get("address").unwrap());
}

#[test]
fn handles_multiple_fake_values() {
    let mut grouping = FakeValuesGrouping::with(address_values()).unwrap();
    let cat_values = FakeValues::of(FakeValuesContext::of_file(&english(), "cat.yml", Some("creature")));
    grouping.add(FakeValuesInterface::Values(cat_values)).unwrap();
    let address = grouping.get("address").unwrap();
    assert!(address.is_some());
    assert_eq!(address, address_values().get("address").unwrap());
    let creature = grouping.get("creature").unwrap();
    assert!(creature.is_some());
    assert_eq!(creature, cat_values.get("creature").unwrap());
}
