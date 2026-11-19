//! Port of Datafaker 2.7.0's `CustomFakerTest`: a custom provider and a faker "subclass" that declares it.
//!
//! Upstream's `MyCustomFaker extends BaseFaker` declares `insect()` and `insectFromFile()`; here a faker declares
//! the same accessors with `Faker::declare_provider`, and `getProvider(Class, Insect::new)` is `Faker::get_provider`.
//! The user classes implement `JavaClass` with their methods as a table, where upstream finds them by reflection.
//! `addPath(locale, null)` has no Rust form (a path is never null); `addNullExistingPath` checks an empty path.
//! Repeated tests run their body ten times.

use std::sync::Arc;

use datafaker::dispatch::{JValue, JavaClass, ObjectMethod};
use datafaker::java::Random;
use datafaker::yaml::{Map, Value};
use datafaker::{Faker, Locale, Result, WeightedRandomSelector};

/// `CustomFakerTest.Insect`.
struct Insect {
    selector: WeightedRandomSelector,
}

const INSECT_NAMES: [&str; 4] = ["Ant", "Beetle", "Butterfly", "Wasp"];

impl Insect {
    fn new(_faker: &Faker) -> Insect {
        Insect { selector: WeightedRandomSelector::new(Some(Arc::new(Random::from_entropy()))) }
    }

    fn next_insect_name(&self, faker: &Faker) -> Result<String> {
        Ok(INSECT_NAMES[faker.random().next_int_bounded(INSECT_NAMES.len() as i32)? as usize].to_string())
    }

    fn weighted_insect_name(&self) -> Result<String> {
        let item = |value: &str, weight: f64| {
            let mut m = Map::new();
            m.insert("value".into(), Value::Str(value.into()));
            m.insert("weight".into(), Value::Float(weight));
            Some(m)
        };
        let items = [item("Driver ant", 6.0), item("Fire ant", 3.0), item("Harvester ant", 1.0)];
        Ok(self.selector.select(Some(&items))?.to_java_string())
    }
}

impl JavaClass for Insect {
    const NAME: &'static str = "net.datafaker.providers.base.CustomFakerTest$Insect";

    const METHODS: &'static [ObjectMethod<Self>] = &[
        ObjectMethod::object("nextInsectName", &[], |this, f, _| this.next_insect_name(f).map(JValue::Str)),
        ObjectMethod::object("weightedInsectName", &[], |this, _, _| this.weighted_insect_name().map(JValue::Str)),
    ];
}

/// `CustomFakerTest.InsectFromFile`: its constructor adds the two files to English.
struct InsectFromFile;

const KEY: &str = "insectsfromfile";

impl InsectFromFile {
    fn new(faker: &Faker) -> InsectFromFile {
        faker.add_path(&Locale::english(), "tests/resources/ants.yml").unwrap();
        faker.add_path(&Locale::english(), "tests/resources/bees.yml").unwrap();
        InsectFromFile
    }

    fn ant(self: &Arc<Self>, faker: &Faker) -> Result<String> {
        faker.fake_values_service().resolve_on(&format!("{KEY}.ants"), self)
    }

    fn bee(self: &Arc<Self>, faker: &Faker) -> Result<String> {
        faker.fake_values_service().resolve_on(&format!("{KEY}.bees"), self)
    }
}

impl JavaClass for InsectFromFile {
    const NAME: &'static str = "net.datafaker.providers.base.CustomFakerTest$InsectFromFile";

    const METHODS: &'static [ObjectMethod<Self>] = &[];
}

/// `new MyCustomFaker()`.
fn my_custom_faker() -> Faker {
    let faker = Faker::new();
    faker.declare_provider("insect", Insect::new);
    faker.declare_provider("insectFromFile", InsectFromFile::new);
    faker
}

/// `myFaker.insect()`.
fn insect(faker: &Faker) -> Arc<Insect> {
    faker.get_provider(Insect::new).unwrap()
}

fn insect_from_file(faker: &Faker) -> Arc<InsectFromFile> {
    faker.get_provider(InsectFromFile::new).unwrap()
}

fn assert_matches(regex: &str, s: &str) {
    assert!(regex::Regex::new(&format!("^(?:{regex})$")).unwrap().is_match(s), "{s:?} does not match {regex}");
}

#[test]
fn add_null_existing_path() {
    assert_eq!(Faker::new().add_path(&Locale::english(), "").unwrap_err().simple_class(), "IllegalArgumentException");
}

#[test]
fn add_non_existing_path() {
    assert_eq!(Faker::new().add_path(&Locale::english(), "non-existing-file").unwrap_err().simple_class(), "IllegalArgumentException");
}

#[test]
fn insect_test() {
    for _ in 0..10 {
        let my_faker = my_custom_faker();
        assert_matches("[A-Za-z ]+", &insect(&my_faker).next_insect_name(&my_faker).unwrap());
    }
}

#[test]
fn insect_test_expression() {
    for _ in 0..10 {
        let my_faker = my_custom_faker();
        assert_matches("[A-Za-z ]+", &my_faker.expression("#{Insect.nextInsectName}").unwrap());
    }
}

#[test]
fn insect_ant_test_expression_from_file() {
    for _ in 0..10 {
        let my_faker = my_custom_faker();
        assert_matches("[A-Za-z ]+", &insect_from_file(&my_faker).ant(&my_faker).unwrap());
    }
}

#[test]
fn insect_bee_test_expression_from_file() {
    for _ in 0..10 {
        let my_faker = my_custom_faker();
        assert!(insect_from_file(&my_faker).bee(&my_faker).unwrap().ends_with("bee"));
    }
}

/// `BaseFaker.getProvider(InsectFromFile.class, InsectFromFile::new, faker)` (deprecated): the same as the
/// faker's own `getProvider`.
#[test]
fn insect_bee_test_expression_from_file_without_extra_faker() {
    let faker = Faker::new();
    assert!(faker.get_provider(InsectFromFile::new).unwrap().bee(&faker).unwrap().ends_with("bee"));
}

#[test]
fn insect_test_without_extra_faker() {
    let faker = Faker::new();
    assert_matches("[A-Za-z ]+", &faker.get_provider(Insect::new).unwrap().next_insect_name(&faker).unwrap());
}

#[test]
fn test_multiple_faker_contexts_per_one_class_name() {
    let faker1 = Faker::with_locale(Locale::english());
    let faker2 = Faker::with_locale(Locale::language_only("de"));
    let insect1 = faker1.get_provider(Insect::new).unwrap();
    let insect2 = faker2.get_provider(Insect::new).unwrap();
    assert!(!Arc::ptr_eq(&insect1, &insect2));
    // Each faker keeps its one instance.
    assert!(Arc::ptr_eq(&insect1, &faker1.get_provider(Insect::new).unwrap()));
}

#[test]
fn weighted_insect_name_test() {
    let count = 300;
    let my_faker = my_custom_faker();
    let mut counts = std::collections::HashMap::from([("Driver ant".to_string(), 0), ("Fire ant".to_string(), 0), ("Harvester ant".to_string(), 0)]);
    for _ in 0..count {
        let selected = insect(&my_faker).weighted_insect_name().unwrap();
        *counts.get_mut(&selected).expect("one of the three") += 1;
    }
    assert_eq!(counts.len(), 3);
    assert_eq!(counts.values().sum::<i32>(), count, "Insects: {counts:?}");
    assert!(counts["Driver ant"] > counts["Fire ant"], "Insects: {counts:?}");
    assert!(counts["Fire ant"] > counts["Harvester ant"], "Insects: {counts:?}");
}

/// Not upstream: the declared accessor's name reaches the provider as `#{insect.next_insect_name}` does on a
/// `MyCustomFaker` (`resolveFakerObjectAndMethod`), and `getProvider("Insect")` returns the cached instance.
#[test]
fn declared_accessor_and_provider_by_name() {
    let my_faker = my_custom_faker();
    assert_matches("[A-Za-z ]+", &my_faker.expression("#{insect.next_insect_name}").unwrap());
    let by_name = my_faker.custom_provider("Insect").unwrap().unwrap().downcast::<Insect>().ok().unwrap();
    assert!(Arc::ptr_eq(&by_name, &insect(&my_faker)));
    assert!(my_faker.custom_provider("Beetle").unwrap().is_none());
}

/// Not upstream: the white list sees a custom provider's binary name when the faker would create it.
#[test]
fn white_list_applies_to_custom_providers() {
    let faker = Faker::with_white_list(Locale::english(), datafaker::RandomService::default(), |c| !c.ends_with("$Insect"));
    faker.declare_provider("insect", Insect::new);
    let message = "Provider 'net.datafaker.providers.base.CustomFakerTest$Insect' is not in white list";
    assert_eq!(faker.get_provider(Insect::new).err().unwrap().message(), message);
    assert_eq!(faker.expression("#{Insect.nextInsectName}").unwrap_err().root_cause().message(), message);
    assert!(faker.get_provider(InsectFromFile::new).is_ok());
}
