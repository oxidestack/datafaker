//! `FakeValuesServiceTest` (Datafaker 2.7.0, `service/FakeValuesServiceTest.java`).
//!
//! Upstream runs it on `new Faker(new Locale("test"))`, whose data is `test.yml` from the test class path. Here
//! the same file is added to the `test` locale with `addPath`, which groups it with the (empty) bundled data.
//! Adapted: the Mockito `verify(faker).regexify(…)` checks are left out (the values are checked), and the
//! `FakerContext` tests use the faker, which holds the context here. The user objects (`DummyService`, the local
//! `Property` provider) implement `JavaClass` with their methods as a table, where upstream finds them by
//! reflection. `futureDateExpression` and `pastDateExpression` parse the `Timestamp` text by hand (upstream: a
//! `DateTimeFormatter` with an optional fraction).

mod common;

use datafaker::yaml::Value;
use datafaker::{Faker, Locale, RandomService};

const TEST_YML: &str = "tests/resources/test.yml";

fn test_locale() -> Locale {
    Locale::language_only("test")
}

fn faker() -> Faker {
    let faker = Faker::with_locale(test_locale());
    faker.add_path(&test_locale(), TEST_YML).unwrap();
    faker
}

fn s(v: &str) -> Value {
    Value::Str(v.into())
}

#[test]
fn fetch_string_should_return_value() {
    let v = faker().fake_values_service().fetch_string("property.dummy").unwrap().unwrap();
    assert!(["x", "y", "z"].contains(&v.as_str()));
}

#[test]
fn fetch_should_return_value() {
    let f = faker();
    let v = f.fake_values_service().fetch("property.dummy").unwrap().unwrap();
    assert!([s("x"), s("y"), s("z")].contains(v));
}

#[test]
fn fetch_object_should_return_value() {
    let f = faker();
    assert_eq!(f.fake_values_service().fetch_object("property.dummy"), Some(&Value::List(vec![s("x"), s("y"), s("z")])));
}

#[test]
fn safe_fetch_should_return_value_in_list() {
    let v = faker().fake_values_service().safe_fetch("property.dummy", None).unwrap().unwrap();
    assert!(["x", "y", "z"].contains(&v.as_str()));
}

#[test]
fn safe_fetch_should_return_simple_list() {
    assert_eq!(faker().fake_values_service().safe_fetch("property.simple", None).unwrap().as_deref(), Some("hello"));
}

#[test]
fn safe_fetch_should_return_empty_string_when_property_doesnt_exist() {
    assert_eq!(faker().fake_values_service().safe_fetch("property.dummy2", Some("")).unwrap().as_deref(), Some(""));
}

/// The current object (`DummyService`) has none of the methods named, so the faker's methods resolve the
/// directives, as with no current object.
#[test]
fn bothify2_args() {
    let value = faker().fake_values_service().resolve("property.bothify_2", None).unwrap();
    common::assert_matches("[A-Z]{2}\\d{2}", &value);
}

#[test]
fn regexify_directive() {
    let value = faker().fake_values_service().resolve("property.regexify1", None).unwrap();
    assert!(["55", "44", "45", "54"].contains(&value.as_str()), "{value}");
}

#[test]
fn regexify_slash_format_directive() {
    let value = faker().fake_values_service().resolve("property.regexify_slash_format", None).unwrap();
    assert!(["55", "44", "45", "54"].contains(&value.as_str()), "{value}");
}

#[test]
fn regexify_directive2() {
    let value = faker().fake_values_service().resolve("property.regexify_cell", None).unwrap();
    assert!(["479", "459"].contains(&value.as_str()), "{value}");
}

fn chain(locale: Locale) -> Vec<Locale> {
    locale.chain()
}

#[test]
fn test_locale_chain() {
    let chain = chain(Locale::new("zh", "CN"));
    for l in [Locale::new("zh", "CN"), Locale::language_only("zh"), Locale::english()] {
        assert!(chain.contains(&l), "{l}");
    }
}

#[test]
fn test_locale_chain_english() {
    assert!(chain(Locale::english()).contains(&Locale::english()));
}

#[test]
fn test_locale_chain_language_only() {
    let chain = chain(Locale::language_only("zh"));
    assert!(chain.contains(&Locale::language_only("zh")) && chain.contains(&Locale::english()));
}

#[test]
fn test_locales_chain_getter() {
    let chain = faker().locale_chain();
    assert!(chain.contains(&test_locale()) && chain.contains(&Locale::english()));
}

#[test]
fn test_locales_chain_getter_ru() {
    let ru = Locale::language_only("ru");
    assert_eq!(Faker::with_locale(ru.clone()).locale_chain(), ru.chain());
}

#[test]
fn test_faker_context_set_locale() {
    let f = Faker::with_locale(Locale::language_only("en"));
    assert_eq!(f.do_with_locale(Locale::language_only("uk"), |f| f.locale()), Locale::language_only("uk"));
}

/// `new RandomService()` is the shared generator, so two of them compare equal.
#[test]
fn test_faker_context_set_random_service() {
    let f = Faker::with_locale_and_seed(Locale::new("en", "US"), 42);
    let shared = RandomService::default();
    assert!(!std::ptr::addr_eq(f.random().random(), shared.random()));
    f.do_with_random(RandomService::default(), |f| assert!(std::ptr::addr_eq(f.random().random(), shared.random())));
}

/// `FakerContext` equality: the same locale and the same random service.
#[test]
fn check_faker_context_equality() {
    let context = |f: &Faker| (f.locale(), std::ptr::from_ref(f.random().random()).cast::<()>());
    let en = Faker::with_locale(Locale::language_only("en"));
    let uk = Faker::with_locale(Locale::language_only("uk"));
    assert_ne!(context(&en), context(&uk));
    let us1 = Faker::with_locale(Locale::new("en", "US"));
    let us2 = Faker::with_locale(Locale::new("en", "US"));
    assert_eq!(context(&us1), context(&us2));
}

fn expression_should_fail_with_containing(expression: &str, message: &str) {
    let e = faker().fake_values_service().expression(expression).unwrap_err();
    assert_eq!(e.simple_class(), "RuntimeException");
    assert!(e.message().contains(message), "{}", e.message());
}

#[test]
fn expression_with_invalid_faker_object() {
    expression_should_fail_with_containing("#{ObjectNotOnFaker.methodName}", "Unable to resolve #{ObjectNotOnFaker.methodName} directive");
}

#[test]
fn expression_with_valid_faker_object_but_invalid_method() {
    expression_should_fail_with_containing("#{Name.nonExistentMethod}", "Unable to resolve #{Name.nonExistentMethod} directive");
}

#[test]
fn expression_with_valid_faker_object_valid_method_invalid_args() {
    expression_should_fail_with_containing("#{Number.number_between 'x','y'}", "Unable to resolve #{Number.number_between 'x','y'} directive");
}

#[test]
fn expression_with_four_arguments() {
    let value = faker().fake_values_service().expression("#{Internet.password '5','8','true','true'}").unwrap();
    common::assert_matches("[\\w\\d!%#$@_^&*]{5,8}", &value);
}

#[test]
fn file_expression_test_failure() {
    for filename in ["src/test/test.txt_null", "qwerty", "src"] {
        let e = faker().fake_values_service().file_expression(filename).unwrap_err();
        assert_eq!(e.simple_class(), "RuntimeException");
    }
}

fn temp_file(content: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join(format!("tmp{}-{}file", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn file_no_expression_test() {
    let path = temp_file("");
    assert_eq!(faker().fake_values_service().file_expression(&path).unwrap(), "");
}

/// Upstream's `src/test/test.txt` is `tes` and `#{Address.city}`; `Address` is a T1 provider, so the file here
/// names `Name.first_name` instead.
#[test]
fn file_expression_test() {
    let path = temp_file("tes\n#{Name.first_name}\n");
    let raw = std::fs::read_to_string(&path).unwrap().lines().collect::<Vec<_>>().join("\n");
    let resolved = faker().fake_values_service().file_expression(&path).unwrap();
    assert_ne!(raw, resolved);
    assert!(resolved.starts_with("tes\n"));
}

#[test]
fn expression_completely_unresolvable() {
    expression_should_fail_with_containing("#{x}", "Unable to resolve #{x} directive");
}

/// `expressionWithSingleEnumArg` (issue 1274): an enum constant named with its class.
#[test]
fn expression_with_single_enum_arg() {
    let master_card = Faker::new().expression("#{finance.creditCard 'CreditCardType.MASTERCARD'}").unwrap();
    assert!(common::is_luhn_valid(&master_card.replace('-', "")), "{master_card}");
}

/// `Timestamp.toString()` (`yyyy-MM-dd HH:mm:ss.f`, one to three fraction digits here) as a `LocalDateTime`.
fn parse_timestamp(text: &str) -> datafaker::java::LocalDateTime {
    use datafaker::java::{LocalDate, LocalDateTime, LocalTime};
    let (date, time) = text.split_once(' ').expect("a date and a time");
    let (hms, fraction) = time.split_once('.').expect("a fraction");
    assert!((1..=3).contains(&fraction.len()), "{text}");
    LocalDateTime::of(LocalDate::parse(date).unwrap(), LocalTime::parse(&format!("{hms}.{fraction}")).unwrap())
}

/// `futureDateExpression` (repeated 10 times).
#[test]
fn future_date_expression() {
    use datafaker::java::LocalDateTime;
    for _ in 0..10 {
        let now = LocalDateTime::now();
        let now_plus_10_days = LocalDateTime::of_instant(now.to_instant().plus(10, datafaker::java::ChronoUnit::Days).unwrap());
        let expression = faker().fake_values_service().expression("#{date.future '10','TimeUnit.DAYS'}").unwrap();
        let date = parse_timestamp(&expression);
        assert!(date > now && date < now_plus_10_days, "{expression}");
    }
}

/// `pastDateExpression` (repeated 10 times).
#[test]
fn past_date_expression() {
    use datafaker::java::{ChronoUnit, LocalDateTime};
    for _ in 0..10 {
        let now = LocalDateTime::now();
        let now_minus_5_hours = LocalDateTime::of_instant(now.to_instant().minus(5, ChronoUnit::Hours).unwrap());
        let expression = faker().fake_values_service().expression("#{date.past '4','TimeUnit.HOURS'}").unwrap();
        let date = parse_timestamp(&expression);
        assert!(date > now_minus_5_hours && date < now, "{expression}");
    }
}

/// `FakeValuesServiceTest.DummyService`: `hello()` returns the next greeting.
struct DummyService {
    counter: std::sync::atomic::AtomicUsize,
    greetings: Vec<String>,
}

impl DummyService {
    fn new(greetings: &[&str]) -> std::sync::Arc<DummyService> {
        std::sync::Arc::new(DummyService { counter: Default::default(), greetings: greetings.iter().map(|g| g.to_string()).collect() })
    }
}

impl datafaker::dispatch::JavaClass for DummyService {
    const NAME: &'static str = "net.datafaker.service.FakeValuesServiceTest$DummyService";
    const PROVIDER: bool = false;
    const METHODS: &'static [datafaker::dispatch::ObjectMethod<Self>] = &[
        datafaker::dispatch::ObjectMethod::object("firstName", &[], |_, _, _| Ok(datafaker::JValue::str("John"))),
        datafaker::dispatch::ObjectMethod::object("lastName", &[], |_, _, _| Ok(datafaker::JValue::str("Smith"))),
        datafaker::dispatch::ObjectMethod::object("hello", &[], |this, _, _| {
            let i = this.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(datafaker::JValue::str(this.greetings[i].clone()))
        }),
    ];
}

#[test]
fn resolve_key_to_property_with_a_property_without_an_object() {
    let dummy = DummyService::new(&["Yo!"]);
    assert_eq!(faker().fake_values_service().resolve_on("property.simpleResolution", &dummy).unwrap(), "Yo!");
}

/// The local `Property` provider of `resolveKeyToPropertyWithAList`: its constructor adds `test.yml` to `test`.
struct Property;

impl Property {
    fn new(faker: &Faker) -> Property {
        faker.add_path(&test_locale(), TEST_YML).unwrap();
        Property
    }

    fn resolution_with_list(self: &std::sync::Arc<Self>, faker: &Faker) -> datafaker::Result<String> {
        faker.fake_values_service().resolve_on("property.resolutionWithList", self)
    }
}

impl datafaker::dispatch::JavaClass for Property {
    const NAME: &'static str = "net.datafaker.service.FakeValuesServiceTest$1Property";
    const METHODS: &'static [datafaker::dispatch::ObjectMethod<Self>] = &[
        datafaker::dispatch::ObjectMethod::object("hello", &[], |_, _, _| Ok(datafaker::JValue::str("Yo!"))),
        datafaker::dispatch::ObjectMethod::object("hello2", &[], |_, _, _| Ok(datafaker::JValue::str("Yo2!"))),
        datafaker::dispatch::ObjectMethod::object("resolutionWithList", &[], |_, f, _| {
            f.get_provider(Property::new)?.resolution_with_list(f).map(datafaker::JValue::Str)
        }),
    ];
}

/// `PropertyFaker`'s `property()` is a declared accessor; `getProvider("Property")` finds it by the class's name.
#[test]
fn resolve_key_to_property_with_a_list() {
    let test_faker = Faker::with_locale(test_locale());
    test_faker.declare_provider("property", Property::new);
    for _ in 0..20 {
        let provider = test_faker.custom_provider("Property").unwrap().unwrap().downcast::<Property>().ok().unwrap();
        let actual = provider.resolution_with_list(&test_faker).unwrap();
        assert!(actual.starts_with("Yo"), "{actual}");
    }
}

#[test]
fn resolve_using_the_same_key_twice() {
    let dummy = DummyService::new(&["Hello", "Tere", "Labvakar"]);
    assert_eq!(faker().fake_values_service().resolve_on("property.sameResolution", &dummy).unwrap(), "Hello Tere");
}
