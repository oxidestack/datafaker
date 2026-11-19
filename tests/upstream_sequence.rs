//! `FakeCollectionTest` and `FakeStreamTest` (Datafaker 2.7.0, `sequence/`), with their `Data` classes. Repeated
//! tests run their body ten times; parameterized ones loop over their cases. A Java list or stream element is an
//! `Option` here (`None` for `null`).

mod common;

use common::assert_matches;
use datafaker::providers::Name;
use datafaker::sequence::{FakeCollection, FakeSequenceBuilder, Supplier};
use datafaker::transformations::{
    CsvTransformer, Field, JsonTransformer, LINE_SEPARATOR, Schema, SqlTransformer, Transformer, XmlTransformer, composite_field, field, field_of,
};
use datafaker::{Faker, RandomService, Result};
use std::sync::Arc;

/// `Data` and its three implementations: each value reads a new unseeded faker's random service.
#[derive(Clone, Copy)]
enum Data {
    BloodPressure,
    Glucose,
    Temperature,
}

impl Data {
    fn name(&self) -> &'static str {
        match self {
            Data::BloodPressure => "Mean Blood Pressure",
            Data::Glucose => "Glucose",
            Data::Temperature => "Temperature",
        }
    }

    fn value(&self) -> String {
        let random = RandomService::default();
        match self {
            Data::BloodPressure => random.next_int_between(60, 180).unwrap().to_string(),
            Data::Glucose => format!("{:.1}", random.next_double_between(3.2, 5.5)),
            Data::Temperature => random.next_int_between(30, 50).unwrap().to_string(),
        }
    }

    fn range(&self) -> &'static str {
        match self {
            Data::Glucose => "3.2-5.5",
            _ => "",
        }
    }

    fn unit(&self) -> &'static str {
        match self {
            Data::BloodPressure => "mm Hg",
            Data::Glucose => "mmol/L",
            Data::Temperature => "degrees C",
        }
    }
}

fn data_suppliers<'a, K>(builder: FakeSequenceBuilder<'a, Data, K>) -> FakeSequenceBuilder<'a, Data, K> {
    builder.supplier(|| Data::BloodPressure).supplier(|| Data::Glucose).supplier(|| Data::Temperature)
}

fn data_schema() -> Schema<'static, Data> {
    Schema::of(vec![
        field_of("name", |d: &Data| d.name()),
        field_of("value", |d: &Data| d.value()),
        field_of("range", |d: &Data| d.range()),
        field_of("unit", |d: &Data| d.unit()),
    ])
}

fn count_lines_and_separators(csv: &str, separator: &str) -> (usize, usize) {
    let (mut lines, mut seps) = (0, 0);
    let bytes = csv.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i..].starts_with(LINE_SEPARATOR.as_bytes()) {
            lines += 1;
        } else if bytes[i..].starts_with(separator.as_bytes()) {
            seps += 1;
        }
    }
    (lines, seps)
}

fn some<T>(items: Vec<Option<T>>) -> Vec<T> {
    items.into_iter().map(|i| i.expect("not null")).collect()
}

const NULL_RATES: [f64; 8] = [i64::MIN as f64, i32::MIN as f64, -1.0, -0.3, 2.0, 3.0, i32::MAX as f64, f64::MAX];

fn null_rate_message(rate: f64) -> String {
    format!("Null rate should be between 0 and 1 (received: {})", datafaker::java::double_to_string(rate))
}

const INFINITE: &str = "The sequence should be finite of size: FakeStream{minLength=-1, maxLength=-1, nullRate=0.0}";

/// Address fields for `toNestedJson`.
fn address_fields(faker: &Faker) -> Vec<Field<'_, Name<'_>>> {
    vec![
        field("country", move || faker.address().country()),
        field("city", move || faker.address().city()),
        field("zipcode", move || faker.address().zip_code()),
        field("streetAddress", move || faker.address().street_address()),
    ]
}

// FakeCollectionTest

#[test]
fn generate_collection() {
    let faker = Faker::new();
    let names = faker.collection().supplier(|| faker.name().first_name().unwrap()).supplier(|| faker.name().last_name().unwrap()).min_len(3).max_len(5).build().unwrap().get().unwrap();
    assert!((3..=5).contains(&names.len()));
    for name in some(names) {
        assert_matches("[a-zA-Z']+", &name);
    }
}

#[test]
fn generate_sequence() {
    let faker = Faker::new();
    let digits = faker.collection().supplier(|| faker.number().digit()).len_between(3, 10).generate().unwrap();
    assert!((3..=10).contains(&digits.len()));
    for digit in some(digits) {
        assert_matches("\\d", &digit);
    }
}

#[test]
fn generate_sequence_of_default_size() {
    let faker = Faker::new();
    assert_eq!(faker.collection().supplier(|| faker.number().digit()).generate().unwrap().len(), 10);
    assert_eq!(faker.collection().supplier(|| faker.number().digit()).max_len(-175).generate().unwrap().len(), 10);
}

#[test]
fn generate_empty_sequence() {
    let faker = Faker::new();
    assert!(faker.collection().supplier(|| faker.number().digit()).max_len(0).generate().unwrap().is_empty());
}

#[test]
fn collection_is_infinite_test() {
    let faker = Faker::new();
    assert!(!faker.collection().supplier(|| faker.number().digit()).build().unwrap().is_infinite());
    assert!(!faker.collection().supplier(|| faker.number().digit()).min_len(3).max_len(5).build().unwrap().is_infinite());
}

#[test]
fn generate_sequence5() {
    let faker = Faker::new();
    let digits = faker.collection().supplier(|| faker.number().digit()).len(5).generate().unwrap();
    assert_eq!(digits.len(), 5);
    for digit in some(digits) {
        assert_matches("\\d", &digit);
    }
}

#[test]
fn generate_null_collection() {
    let faker = Faker::new();
    let names = faker
        .collection()
        .supplier(|| faker.name().first_name().unwrap())
        .supplier(|| faker.name().last_name().unwrap())
        .null_rate(1.0)
        .unwrap()
        .min_len(3)
        .max_len(5)
        .build()
        .unwrap()
        .get()
        .unwrap();
    assert!((3..=5).contains(&names.len()));
    assert!(names.iter().all(Option::is_none));
}

#[test]
fn collection_illegal_null_rate() {
    let faker = Faker::new();
    for rate in NULL_RATES {
        let e = faker.collection::<String>().supplier(|| faker.name().first_name().unwrap()).null_rate(rate).map(|_| ()).unwrap_err();
        assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", null_rate_message(rate).as_str()));
    }
}

#[test]
fn generate_collection_passing_suppliers_as_list() {
    let faker = Faker::new();
    let seeded = Faker::with_seed(10);
    let suppliers: Vec<Supplier<'_, String>> = vec![Arc::new(|| faker.name().first_name().unwrap()), Arc::new(|| faker.name().last_name().unwrap())];
    let names = faker.collection_of(suppliers).faker(&seeded).len(3).generate().unwrap();
    assert_eq!(names.len(), 3);
}

#[test]
fn generate_collection_with_repeatable_faker() {
    let faker = Faker::new();
    let seeded = Faker::with_seed(10);
    let names = faker
        .collection()
        .faker(&seeded)
        .supplier(|| seeded.name().first_name().unwrap())
        .supplier(|| seeded.name().last_name().unwrap())
        .min_len(1)
        .max_len(20)
        .build()
        .unwrap()
        .get()
        .unwrap();
    assert_eq!(names.len(), 14);
    assert_eq!(names[0].as_deref(), Some("Flor"));
    assert_eq!(names[1].as_deref(), Some("Brian"));
}

/// Java's `List<Object>` of `String`s and `Integer`s: an enum of the two here.
#[test]
fn generate_collection_with_different_objects() {
    #[derive(Debug)]
    enum Obj {
        Str(#[allow(dead_code)] String),
        Int(#[allow(dead_code)] i32),
    }
    let faker = Faker::new();
    let objects = faker
        .collection()
        .supplier(|| Obj::Str(faker.name().first_name().unwrap()))
        .supplier(|| Obj::Int(faker.random().next_int_bounded(100).unwrap()))
        .max_len(5)
        .build()
        .unwrap()
        .get()
        .unwrap();
    assert_eq!(objects.len(), 5);
    assert!(objects.iter().all(|o| matches!(o, Some(Obj::Str(_) | Obj::Int(_)))));
}

#[test]
fn collection_check_wrong_arguments() {
    let faker = Faker::new();
    let e = faker.collection().supplier(|| faker.name().first_name().unwrap()).min_len(10).max_len(5).build().map(|_| ()).unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", "Max length (5) must be not less than min length (10) and not negative"));
}

#[test]
fn collection_to_csv() {
    let faker = Faker::new();
    let (separator, limit) = ("$$$", 5);
    let items = data_suppliers(faker.collection()).len(limit).generate().unwrap();
    let csv = CsvTransformer::builder().header(true).separator(separator).build().generate_from(&items, &data_schema()).unwrap();
    let (lines, seps) = count_lines_and_separators(&csv, separator);
    assert_eq!(lines, limit as usize);
    assert_eq!(seps, (limit as usize + 1) * (4 - 1));
}

#[test]
fn collection_to_json() {
    let faker = Faker::new();
    let limit = 10;
    let sequence = data_suppliers(faker.collection().min_len(limit).max_len(limit)).build().unwrap();
    let json = JsonTransformer::builder().build().generate_from(&sequence, &data_schema()).unwrap();
    assert_eq!(json.matches("},").count(), limit as usize - 1);
}

#[test]
fn collection_to_nested_json() {
    let faker = Faker::new();
    let limit = 2;
    let faker = &faker;
    let secondary_addresses: Arc<FakeCollection<'_, Field<'_, Name<'_>>>> =
        Arc::new(faker.collection().supplier(move || composite_field(None::<&str>, address_fields(faker))).max_len(1).min_len(1).build().unwrap());
    let names = faker.collection().min_len(limit).max_len(limit).supplier(|| faker.name()).build().unwrap();
    let schema = Schema::of(vec![
        composite_field("primaryAddress", address_fields(faker)),
        field("secondaryAddresses", move || secondary_addresses.get()),
        field("phones", move || faker.collection().supplier(move || faker.phone_number().phone_number()).max_len(3).build().and_then(|c| c.get())),
    ]);
    let json = JsonTransformer::builder().build().generate_from(&names, &schema).unwrap();
    assert_eq!(json.matches(&format!("}},{LINE_SEPARATOR}")).count(), limit as usize - 1);
}

#[test]
fn collection_singleton_test() {
    let faker = Faker::new();
    for _ in 0..10 {
        let sequence = data_suppliers(faker.collection().min_len(10).max_len(10)).build().unwrap();
        assert!(sequence.singleton().unwrap().is_some());
    }
}

#[test]
fn collection_test_iterator() {
    let faker = Faker::new();
    let digits = faker.collection().supplier(|| faker.number().digit()).len(10).build().unwrap();
    let mut count = 0;
    for digit in some(digits.get().unwrap()) {
        assert_matches("\\d", &digit);
        count += 1;
    }
    assert_eq!(count, 10);
}

// FakeStreamTest

#[test]
fn generate_finite_stream() {
    let faker = Faker::new();
    let names = faker.stream().supplier(|| faker.name().first_name().unwrap()).supplier(|| faker.name().last_name().unwrap()).min_len(3).max_len(5).generate().unwrap();
    assert!((3..=5).contains(&names.count()));
}

/// `spliterator().estimateSize()` is `Long.MAX_VALUE`: an unbounded size hint here.
#[test]
fn generate_infinite_stream() {
    let faker = Faker::new();
    let names = faker.stream().supplier(|| faker.name().first_name().unwrap()).generate().unwrap();
    assert_eq!(names.size_hint().1, None);
}

#[test]
fn generate_stream_of_names() {
    let faker = Faker::new();
    for name in faker.stream().supplier(|| faker.name().first_name().unwrap()).len(5).generate().unwrap() {
        assert_matches("[a-zA-Z']+", &name.unwrap().unwrap());
    }
}

#[test]
fn generate_stream_of_digits() {
    let faker = Faker::new();
    for digit in faker.stream().supplier(|| faker.number().digit()).generate().unwrap().take(1_000) {
        assert_matches("\\d", &digit.unwrap().unwrap());
    }
}

#[test]
fn generate_null_stream() {
    let faker = Faker::new();
    let names = faker
        .stream()
        .supplier(|| faker.name().first_name().unwrap())
        .supplier(|| faker.name().last_name().unwrap())
        .null_rate(1.0)
        .unwrap()
        .len(10)
        .generate()
        .unwrap();
    for name in names {
        assert!(name.unwrap().is_none());
    }
}

#[test]
fn stream_illegal_null_rate() {
    let faker = Faker::new();
    for rate in NULL_RATES {
        let e = faker.stream::<String>().supplier(|| faker.name().first_name().unwrap()).null_rate(rate).map(|_| ()).unwrap_err();
        assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", null_rate_message(rate).as_str()));
    }
}

#[test]
fn generate_stream_passing_suppliers_as_list() {
    let faker = Faker::new();
    let suppliers: Vec<Supplier<'_, String>> = vec![Arc::new(|| faker.name().first_name().unwrap()), Arc::new(|| faker.name().last_name().unwrap())];
    assert_eq!(faker.stream_of(suppliers).len(3).generate().unwrap().count(), 3);
}

#[test]
fn generate_stream_with_repeatable_faker() {
    let faker = Faker::new();
    let seeded = Faker::with_seed(10);
    let names: Vec<Option<String>> = faker
        .stream()
        .faker(&seeded)
        .supplier(|| seeded.name().first_name().unwrap())
        .supplier(|| seeded.name().last_name().unwrap())
        .min_len(1)
        .max_len(20)
        .generate()
        .unwrap()
        .collect::<Result<_>>()
        .unwrap();
    assert_eq!(names.len(), 14);
    assert_eq!(names[0].as_deref(), Some("Flor"));
    assert_eq!(names[1].as_deref(), Some("Brian"));
}

#[test]
fn generate_stream_with_different_objects() {
    let faker = Faker::new();
    let objects = faker
        .stream()
        .supplier(|| Ok::<_, i32>(faker.name().first_name().unwrap()))
        .supplier(|| Err(faker.random().next_int_bounded(100).unwrap()))
        .max_len(5)
        .generate()
        .unwrap();
    for object in objects {
        assert!(matches!(object.unwrap(), Some(Ok(_) | Err(_))));
    }
}

#[test]
fn stream_check_wrong_arguments() {
    let faker = Faker::new();
    let e = faker.stream().supplier(|| faker.name().first_name().unwrap()).min_len(10).max_len(5).generate().map(|_| ()).unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", "Max length (5) must be not less than min length (10) and not negative"));
}

#[test]
fn stream_singleton_test() {
    let faker = Faker::new();
    for _ in 0..10 {
        assert!(data_suppliers(faker.stream().min_len(10).max_len(10)).build().unwrap().singleton().unwrap().is_some());
    }
}

#[test]
fn stream_is_infinite_test() {
    let faker = Faker::new();
    assert!(faker.stream().supplier(|| faker.name().first_name().unwrap()).build().unwrap().is_infinite());
    assert!(!faker.stream().supplier(|| faker.name().first_name().unwrap()).len(5).build().unwrap().is_infinite());
}

#[test]
fn stream_to_csv() {
    let faker = Faker::new();
    let (separator, limit) = ("$$$", 5);
    let stream = data_suppliers(faker.stream().min_len(limit).max_len(limit)).build().unwrap();
    let csv = CsvTransformer::builder().header(true).separator(separator).build().generate_from(&stream, &data_schema()).unwrap();
    assert_eq!(count_lines_and_separators(&csv, separator), (5, 18));
}

#[test]
fn stream_to_csv_from_infinite_sequence() {
    let faker = Faker::new();
    let stream = data_suppliers(faker.stream()).build().unwrap();
    let e = CsvTransformer::builder().separator("$$$").build().generate_from(&stream, &data_schema()).unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", INFINITE));
}

#[test]
fn stream_to_json() {
    let faker = Faker::new();
    let limit = 10;
    let sequence = data_suppliers(faker.collection().min_len(limit).max_len(limit)).build().unwrap();
    let json = JsonTransformer::builder().build().generate_from(&sequence, &data_schema()).unwrap();
    assert_eq!(json.matches("},").count(), limit as usize - 1);
}

#[test]
fn stream_to_json_from_infinite_sequence() {
    let faker = Faker::new();
    let stream = data_suppliers(faker.stream()).build().unwrap();
    let e = JsonTransformer::builder().build().generate_from(&stream, &data_schema()).unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", INFINITE));
}

#[test]
fn stream_to_nested_json() {
    let faker = Faker::new();
    let limit = 3;
    let faker = &faker;
    let secondary_addresses: Arc<FakeCollection<'_, Field<'_, Name<'_>>>> =
        Arc::new(faker.collection().supplier(move || composite_field(None::<&str>, address_fields(faker))).max_len(1).min_len(1).build().unwrap());
    let names = faker.stream().supplier(|| faker.name()).max_len(limit).min_len(limit).build().unwrap();
    let schema = Schema::of(vec![
        composite_field("primaryAddress", address_fields(faker)),
        field("secondaryAddresses", move || secondary_addresses.get()),
        field("phones", move || faker.collection().supplier(move || faker.phone_number().phone_number()).max_len(3).build().and_then(|c| c.get())),
    ]);
    let json = JsonTransformer::builder().build().generate_from(&names, &schema).unwrap();
    assert_eq!(json.matches(&format!("}},{LINE_SEPARATOR}")).count(), limit as usize - 1);
}

#[test]
fn stream_test_iterator() {
    let faker = Faker::new();
    let digits = faker.stream().supplier(|| faker.number().digit()).len(100).build().unwrap();
    let mut count = 0;
    for digit in digits.get().unwrap() {
        assert_matches("\\d", &digit.unwrap().unwrap());
        count += 1;
    }
    assert_eq!(count, 100);
}

#[test]
fn stream_test_iterator_infinite() {
    let faker = Faker::new();
    let digits = faker.stream().supplier(|| faker.number().digit()).build().unwrap();
    assert!(digits.is_infinite());
    let mut count = 0;
    for digit in digits.get().unwrap() {
        assert_matches("\\d", &digit.unwrap().unwrap());
        count += 1;
        if count == 1_000 {
            break;
        }
    }
    assert_eq!(count, 1_000);
}

/// `testCsvFilesCreated`: `writeToOutputStream` into a buffer instead of a temporary file.
#[test]
fn test_csv_files_created() {
    fn lines_of<T: Transformer<'static, ()>>(transformer: &T) -> Vec<String> {
        let faker: &'static Faker = Box::leak(Box::new(Faker::with_seed(10)));
        let name = faker.name();
        let schema: Schema<'static> = Schema::of(vec![field("FirstName", move || name.first_name()), field("LastName", move || name.last_name())]);
        let mut out = Vec::new();
        transformer.write_to_output_stream(&mut out, &schema, 2).unwrap();
        String::from_utf8(out).unwrap().lines().map(str::to_string).collect()
    }
    fn contains_all(lines: Vec<String>, expected: &[&str]) {
        for e in expected {
            assert!(lines.iter().any(|l| l == e), "{e} not in {lines:?}");
        }
    }
    contains_all(
        lines_of(&XmlTransformer::builder().build()),
        &["<FirstName>Willis</FirstName><LastName>Huels</LastName>", "<FirstName>Carlena</FirstName><LastName>Jenkins</LastName>"],
    );
    contains_all(lines_of(&CsvTransformer::builder().build()), &["\"FirstName\";\"LastName\"", "\"Willis\";\"Huels\"", "\"Carlena\";\"Jenkins\""]);
    contains_all(
        lines_of(&SqlTransformer::builder().build().unwrap()),
        &[
            "INSERT INTO \"MyTable\" (\"FirstName\", \"LastName\") VALUES ('Willis', 'Huels');",
            "INSERT INTO \"MyTable\" (\"FirstName\", \"LastName\") VALUES ('Carlena', 'Jenkins');",
        ],
    );
    contains_all(
        lines_of(&JsonTransformer::builder().build()),
        &["[", "{\"FirstName\": \"Willis\", \"LastName\": \"Huels\"},", "{\"FirstName\": \"Carlena\", \"LastName\": \"Jenkins\"}", "]"],
    );
}

