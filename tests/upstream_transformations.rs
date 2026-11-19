//! `JavaObjectTransformerTest`, `JavaObjectTransformerConstructorTest`, `JsonTransformerTest` (Datafaker 2.7.0,
//! `transformations/`), `FakeAnnotationTest` (`annotations/`, with its `dto` classes) and the Kotlin
//! `SchemaExampleTest`. Java's user classes are Rust types implementing `JavaObject`: what reflection finds on the
//! class (its fields, or the constructor it is built with) is written out per type (Rust has no reflection over a class, so the type says how it is built).

use std::collections::HashSet;

use datafaker::annotations::FakeForSchema;
use datafaker::java::{BigDecimal, Instant, LocalDate};
use datafaker::transformations::{JavaObject, JavaObjectTransformer, JsonTransformer, Schema, Transformer, Value, field};
use datafaker::{Error, Faker, Locale, RandomService, Result};

fn mismatch() -> Error {
    Error::illegal_argument("argument type mismatch")
}

fn string<T>(value: Value<'_, T>) -> Result<Option<String>> {
    match value {
        Value::Str(s) => Ok(Some(s)),
        Value::Null => Ok(None),
        _ => Err(mismatch()),
    }
}

fn object<T, O: Clone + 'static>(value: Value<'_, T>) -> Result<Option<O>> {
    match value {
        Value::Object(o) => o.downcast_ref::<O>().cloned().map(Some).ok_or_else(mismatch),
        Value::Null => Ok(None),
        _ => Err(mismatch()),
    }
}

fn int<T>(value: Value<'_, T>) -> Result<i32> {
    match value {
        Value::Int(i) => Ok(i),
        _ => Err(mismatch()),
    }
}

// JavaObjectTransformerTest

/// `Person`: a class with the default constructor, so its fields are set by name.
#[derive(Default)]
struct Person {
    first_name: Option<String>,
    last_name: Option<String>,
    birth_date: Option<LocalDate>,
    registration_date: Option<Instant>,
    id: i32,
}

impl JavaObject for Person {
    fn new_instance() -> Option<Self> {
        Some(Person::default())
    }

    fn field_names() -> &'static [&'static str] {
        &["firstName", "lastName", "birthDate", "registrationDate", "id"]
    }

    fn set_field(&mut self, name: &str, value: Value<'_, Self>) -> Result<()> {
        match name {
            "firstName" => self.first_name = string(value)?,
            "lastName" => self.last_name = string(value)?,
            "birthDate" => self.birth_date = object(value)?,
            "registrationDate" => self.registration_date = object(value)?,
            "id" => self.id = int(value)?,
            _ => return Err(mismatch()),
        }
        Ok(())
    }
}

/// `record Client(String firstName, String lastName, String phoneNumber, Instant registrationDate, int id)`.
struct Client {
    first_name: Option<String>,
    last_name: Option<String>,
    phone_number: Option<String>,
    registration_date: Option<Instant>,
    #[allow(dead_code)]
    id: i32,
}

impl JavaObject for Client {
    fn construct(values: Vec<Value<'_, Self>>) -> Result<Self> {
        let [first_name, last_name, phone_number, registration_date, id]: [Value<'_, Self>; 5] = values.try_into().map_err(|_| mismatch())?;
        Ok(Client {
            first_name: string(first_name)?,
            last_name: string(last_name)?,
            phone_number: string(phone_number)?,
            registration_date: object(registration_date)?,
            id: int(id)?,
        })
    }
}

fn person_schema(faker: &Faker, with_id: bool) -> Schema<'_, Person> {
    let mut fields = vec![
        field("firstName", move || faker.name().first_name()),
        field("lastName", move || faker.name().last_name()),
        field("birthDate", move || faker.time_and_date().birthday()),
        field("registrationDate", move || faker.time_and_date().past()),
    ];
    if with_id {
        fields.push(field("id", move || faker.number().positive()));
    }
    Schema::of(fields)
}

fn assert_person(person: &Person) {
    assert!(person.birth_date.is_some() && person.last_name.is_some() && person.first_name.is_some() && person.registration_date.is_some());
}

#[test]
fn java_object_test() {
    let faker = Faker::new();
    let transformer = JavaObjectTransformer::<Person>::new();
    let schema = person_schema(&faker, true);
    let mut persons = Vec::new();
    for _ in 0..10 {
        let person = transformer.apply(&schema).unwrap();
        assert_person(&person);
        persons.push(person);
    }
    assert_eq!(persons.len(), 10);
}

#[test]
fn java_record_test() {
    let faker = Faker::new();
    let transformer = JavaObjectTransformer::<Client>::new();
    let schema: Schema<Client> = Schema::of(vec![
        field("firstName", || faker.name().first_name()),
        field("lastName", || faker.name().last_name()),
        field("phoneNumber", || faker.phone_number().phone_number_international()),
        field("registrationDate", || faker.time_and_date().past()),
        field("id", || faker.number().positive()),
    ]);
    let mut clients = Vec::new();
    for _ in 0..10 {
        let client = transformer.apply(&schema).unwrap();
        assert!(client.first_name.is_some() && client.last_name.is_some() && client.phone_number.is_some() && client.registration_date.is_some());
        clients.push(client);
    }
    assert_eq!(clients.len(), 10);
}

#[test]
fn java_streaming_test() {
    let faker = Faker::new();
    let transformer = JavaObjectTransformer::<Person>::new().from();
    let schema = person_schema(&faker, false);
    let persons: Vec<Person> = transformer.generate_stream(&schema, 10).unwrap().collect::<Result<_>>().unwrap();
    persons.iter().for_each(assert_person);
    assert_eq!(persons.len(), 10);
}

#[test]
fn java_empty_stream_test() {
    let faker = Faker::new();
    let transformer = JavaObjectTransformer::<Person>::new();
    assert_eq!(transformer.generate_stream(&person_schema(&faker, false), 10).unwrap().count(), 0);
}

#[test]
fn java_collection_test() {
    let faker = Faker::new();
    let transformer = JavaObjectTransformer::<Person>::new().from();
    let persons = transformer.generate(&person_schema(&faker, false), 10).unwrap();
    persons.iter().for_each(assert_person);
    assert_eq!(persons.len(), 10);
}

// JavaObjectTransformerConstructorTest

/// `Person`: a public parameterless constructor and a public `Person(int)` that must not be called.
#[derive(Default)]
struct NoArgPerson {
    id: Option<i32>,
}

impl JavaObject for NoArgPerson {
    fn new_instance() -> Option<Self> {
        Some(NoArgPerson::default())
    }

    fn field_names() -> &'static [&'static str] {
        &["id"]
    }

    fn construct(_: Vec<Value<'_, Self>>) -> Result<Self> {
        Err(Error::runtime("This should not be called"))
    }

    fn set_field(&mut self, _: &str, value: Value<'_, Self>) -> Result<()> {
        self.id = Some(int(value)?);
        Ok(())
    }
}

/// `ProtectedPerson` and `PrivatePerson`: the parameterless constructor is not public (and throws), so the public
/// `(int id)` one builds them.
struct ConstructedPerson {
    id: Option<i32>,
}

impl JavaObject for ConstructedPerson {
    fn construct(values: Vec<Value<'_, Self>>) -> Result<Self> {
        let [id]: [Value<'_, Self>; 1] = values.try_into().map_err(|_| mismatch())?;
        Ok(ConstructedPerson { id: Some(int(id)?) })
    }
}

#[test]
fn java_no_arg_constructor_test() {
    let faker = Faker::new();
    let schema: Schema<NoArgPerson> = Schema::of(vec![field("id", || faker.number().positive())]);
    for person in JavaObjectTransformer::<NoArgPerson>::new().from().generate(&schema, 10).unwrap() {
        assert!(person.id.is_some());
    }
}

#[test]
fn java_no_arg_constructor_protected_test() {
    let faker = Faker::new();
    let schema: Schema<ConstructedPerson> = Schema::of(vec![field("id", || faker.number().positive())]);
    for person in JavaObjectTransformer::<ConstructedPerson>::new().from().generate(&schema, 10).unwrap() {
        assert!(person.id.is_some());
    }
}

#[test]
fn java_no_arg_constructor_private_test() {
    let faker = Faker::new();
    let schema: Schema<ConstructedPerson> = Schema::of(vec![field("id", || faker.number().positive())]);
    for person in JavaObjectTransformer::<ConstructedPerson>::new().from().generate(&schema, 10).unwrap() {
        assert!(person.id.is_some());
    }
}

// JsonTransformerTest

#[test]
fn issue1556_double() {
    let faker = Faker::new();
    let a_double = faker.number().random_double(3, 1, 2);
    let schema: Schema = Schema::of(vec![field("Dbl", move || a_double)]);
    let json = JsonTransformer::builder().build().generate(&schema, 1).unwrap();
    assert_eq!(json, format!("{{\"Dbl\": {}}}", datafaker::java::double_to_string(a_double)));
}

#[test]
fn issue1556_big_decimal() {
    let faker = Faker::new();
    let a_big_dec = BigDecimal::value_of_double(faker.number().random_double(7, 1, 2)).unwrap();
    let text = a_big_dec.to_string();
    let schema: Schema = Schema::of(vec![field("BigDecimal", move || a_big_dec.clone())]);
    let json = JsonTransformer::builder().build().generate(&schema, 1).unwrap();
    assert_eq!(json, format!("{{\"BigDecimal\": {text}}}"));
}

// FakeAnnotationTest

fn default_schema<T>() -> Schema<'static, T> {
    let faker = Faker::with_locale_and_random(Locale::for_language_tag("fr-en"), RandomService::with_seed(1));
    Schema::of(vec![field("name", move || faker.name().full_name())])
}

fn custom_schema<T>() -> Schema<'static, T> {
    let faker = Faker::with_locale_and_random(Locale::for_language_tag("de-en"), RandomService::with_seed(1));
    Schema::of(vec![field("name", move || faker.superhero().name())])
}

fn complex_schema<T>() -> Schema<'static, T> {
    let faker = std::sync::Arc::new(Faker::with_locale_and_random(Locale::for_language_tag("de-DE"), RandomService::with_seed(1)));
    let (f1, f2, f3) = (faker.clone(), faker.clone(), faker);
    Schema::of(vec![
        field("name", move || f1.superhero().name()),
        field("address", move || f2.address().full_address()),
        field("color", move || f3.color().name()),
    ])
}

/// A class with one `name` field (`dto.Person`, `DefaultPerson`, `SimplePerson`), set by name.
macro_rules! name_class {
    ($name:ident) => {
        #[derive(Default, PartialEq, Eq, Hash)]
        struct $name {
            name: Option<String>,
        }

        impl JavaObject for $name {
            fn new_instance() -> Option<Self> {
                Some($name::default())
            }

            fn field_names() -> &'static [&'static str] {
                &["name"]
            }

            fn set_field(&mut self, _: &str, value: Value<'_, Self>) -> Result<()> {
                self.name = string(value)?;
                Ok(())
            }
        }
    };
}

/// A record with one `name` component (`dto.PersonJavaRecord`, `DefaultPersonJavaRecord`,
/// `SimplePersonJavaRecord`).
macro_rules! name_record {
    ($name:ident) => {
        struct $name {
            name: Option<String>,
        }

        impl JavaObject for $name {
            fn construct(values: Vec<Value<'_, Self>>) -> Result<Self> {
                let [name]: [Value<'_, Self>; 1] = values.try_into().map_err(|_| mismatch())?;
                Ok($name { name: string(name)? })
            }
        }
    };
}

name_class!(DtoPerson);
name_class!(DefaultPerson);
name_class!(SimplePerson);
name_record!(PersonJavaRecord);
name_record!(DefaultPersonJavaRecord);
name_record!(SimplePersonJavaRecord);

/// `@FakeForSchema("net.datafaker.annotations.FakeAnnotationTest#defaultSchema")`.
impl FakeForSchema for DtoPerson {
    fn default_schema() -> Schema<'static, Self> {
        default_schema()
    }
}

impl FakeForSchema for PersonJavaRecord {
    fn default_schema() -> Schema<'static, Self> {
        default_schema()
    }
}

/// `@FakeForSchema("defaultSchema")`: a method of the enclosing class.
impl FakeForSchema for DefaultPerson {
    fn default_schema() -> Schema<'static, Self> {
        default_schema()
    }
}

impl FakeForSchema for DefaultPersonJavaRecord {
    fn default_schema() -> Schema<'static, Self> {
        default_schema()
    }
}

/// `dto.ComplexPersonJavaRecord(String name, String address, String color)`.
struct ComplexPersonJavaRecord {
    name: Option<String>,
    address: Option<String>,
    color: Option<String>,
}

impl JavaObject for ComplexPersonJavaRecord {
    fn construct(values: Vec<Value<'_, Self>>) -> Result<Self> {
        let [name, address, color]: [Value<'_, Self>; 3] = values.try_into().map_err(|_| mismatch())?;
        Ok(ComplexPersonJavaRecord { name: string(name)?, address: string(address)?, color: string(color)? })
    }
}

#[test]
fn should_generate_entity_with_default_schema() {
    let person = Faker::populate::<DtoPerson>().unwrap();
    assert_eq!(person.name.as_deref(), Some("Dr Alexis Noël"));
}

#[test]
fn should_generate_entity_from_java_record_with_default_schema() {
    let person = Faker::populate::<PersonJavaRecord>().unwrap();
    assert_eq!(person.name.as_deref(), Some("Dr Alexis Noël"));
}

#[test]
fn should_generate_entity_with_custom_schema() {
    let person = Faker::populate_with::<DtoPerson>(&custom_schema()).unwrap();
    assert_eq!(person.name.as_deref(), Some("Aztar Ivy"));
}

#[test]
fn should_generate_entity_from_java_record_with_custom_schema() {
    let person = Faker::populate_with::<PersonJavaRecord>(&custom_schema()).unwrap();
    assert_eq!(person.name.as_deref(), Some("Aztar Ivy"));
}

#[test]
fn should_generate_entity_with_custom_schema_when_class_template_without_annotation() {
    let person = Faker::populate_with::<SimplePerson>(&custom_schema()).unwrap();
    assert_eq!(person.name.as_deref(), Some("Aztar Ivy"));
}

#[test]
fn should_generate_entity_from_java_record_with_custom_schema_when_class_template_without_annotation() {
    let person = Faker::populate_with::<SimplePersonJavaRecord>(&custom_schema()).unwrap();
    assert_eq!(person.name.as_deref(), Some("Aztar Ivy"));
}

#[test]
fn should_generate_entity_from_java_record_with_complex_schema_when_class_template_without_annotation() {
    let person = Faker::populate_with::<ComplexPersonJavaRecord>(&complex_schema()).unwrap();
    assert_eq!(person.name.as_deref(), Some("Aztar Ivy"));
    assert_eq!(person.address.as_deref(), Some("Am Buttermarkt 43b, Furkanheim, BE 71304"));
    assert_eq!(person.color.as_deref(), Some("aprikose"));
}

#[test]
fn should_generate_entity_with_default_schema_and_in_default_schema_in_current_class() {
    assert!(Faker::populate::<DefaultPerson>().unwrap().name.is_some());
}

#[test]
fn should_generate_entity_with_default_schema_and_in_default_schema_in_current_class2() {
    let expected_size = 5;
    let schema = custom_schema::<DefaultPerson>();
    let list = datafaker::sequence::FakeCollection::builder()
        .supplier(|| Faker::populate_with::<DefaultPerson>(&schema).unwrap())
        .len(expected_size)
        .generate()
        .unwrap();
    assert_eq!(list.into_iter().map(Option::unwrap).collect::<HashSet<_>>().len(), expected_size as usize);
}

#[test]
fn should_generate_entity_from_java_record_with_default_schema_and_in_default_schema_in_current_class() {
    assert!(Faker::populate::<DefaultPersonJavaRecord>().unwrap().name.is_some());
}

// SchemaExampleTest (Kotlin)

/// `data class Person(var firstName: String, var lastName: String, var birthDate: LocalDate, var id: Int)`: no
/// parameterless constructor, so the primary one builds it.
struct KotlinPerson {
    first_name: String,
    last_name: String,
    birth_date: LocalDate,
    id: i32,
}

impl JavaObject for KotlinPerson {
    fn construct(values: Vec<Value<'_, Self>>) -> Result<Self> {
        let [first_name, last_name, birth_date, id]: [Value<'_, Self>; 4] = values.try_into().map_err(|_| mismatch())?;
        Ok(KotlinPerson {
            first_name: string(first_name)?.ok_or_else(mismatch)?,
            last_name: string(last_name)?.ok_or_else(mismatch)?,
            birth_date: object(birth_date)?.ok_or_else(mismatch)?,
            id: int(id)?,
        })
    }
}

/// AssertJ's `isMixedCase`: neither all upper case nor all lower case.
fn is_mixed_case(s: &str) -> bool {
    s != s.to_uppercase() && s != s.to_lowercase()
}

#[test]
fn should_support_kotlin_data_classes() {
    let faker = Faker::new();
    let schema: Schema<KotlinPerson> = Schema::of(vec![
        field("firstName", || faker.name().first_name()),
        field("lastName", || faker.name().last_name()),
        field("birthDate", || faker.time_and_date().birthday()),
        field("id", || faker.number().positive()),
    ]);
    let person = JavaObjectTransformer::<KotlinPerson>::new().apply(&schema).unwrap();
    assert!(person.id > 0);
    assert!(person.birth_date < LocalDate::now());
    assert!(is_mixed_case(&person.first_name));
    assert!(is_mixed_case(&person.last_name));
}

/// The transformer's own failures, against what upstream throws: a record built from values of the wrong type and
/// a class with a schema field it does not declare.
#[test]
fn java_object_failures() {
    let schema: Schema<ComplexPersonJavaRecord> = Schema::of(vec![field("name", || 1)]);
    let e = JavaObjectTransformer::<ComplexPersonJavaRecord>::new().apply(&schema).err().unwrap();
    assert_eq!(e.simple_class(), "RuntimeException");
    assert!(e.message().starts_with("Failed to instantiate "));
    assert_eq!(e.cause().unwrap().message(), "argument type mismatch");
    let schema: Schema<DtoPerson> = Schema::of(vec![field("nickname", || "x")]);
    let e = JavaObjectTransformer::<DtoPerson>::new().apply(&schema).err().unwrap();
    assert_eq!(e.simple_class(), "NullPointerException");
}
