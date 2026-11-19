//! `CsvTest`, `JsonTest`, `SqlTest`, `TomlTest`, `XmlTest` and `YamlTest` (Datafaker 2.7.0, `formats/`). A
//! parameterized test runs its cases in a loop. Unseeded fakers as upstream.

mod common;

use datafaker::providers::Name;
use datafaker::transformations::sql::Case;
use datafaker::transformations::{
    CsvTransformer, Field, JsonTransformer, LINE_SEPARATOR, PrimitiveArray, Schema, SqlDialect, SqlTransformer, TomlTransformer,
    Transformer, Value, XmlTransformer, YamlTransformer, composite_field, field, field_of,
};
use std::sync::Arc;

use datafaker::{Faker, Locale, Result};

fn count(s: &str, needle: &str) -> usize {
    s.matches(needle).count()
}

fn lines(s: &str) -> usize {
    count(s, LINE_SEPARATOR)
}

fn password(faker: &Faker, n: i32) -> Result<String> {
    faker.internet().password_with(n, n, false, false, true)
}

// CsvTest

fn separators(csv: &str, separator: &str) -> (usize, usize) {
    // `regionMatches` at every index, the line separator first: overlapping separators count once each.
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

#[test]
fn csv_test_new() {
    let faker = Faker::new();
    let separator = "@@@";
    let limit = 20;
    let schema: Schema = Schema::of(vec![
        field("first_name", || faker.name().first_name()),
        field("last_name", || faker.name().last_name()),
        field("address", || faker.address().street_address()),
    ]);
    let csv = CsvTransformer::builder().header(true).separator(separator).build().generate(&schema, limit).unwrap();
    let (n_lines, n_seps) = separators(&csv, separator);
    assert_eq!(n_lines, limit as usize);
    assert_eq!(n_seps, (limit as usize + 1) * 2);
}

#[test]
fn csv_test_with_quotes_new() {
    let faker = Faker::new();
    let separator = "$$$";
    let limit = 20;
    let schema: Schema = Schema::of(vec![
        field("first_name", || faker.expression("#{Name.first_name}")),
        field("last_name", || faker.expression("#{Name.last_name}")),
    ]);
    let csv = CsvTransformer::builder().header(true).separator(separator).build().generate(&schema, limit).unwrap();
    let (n_lines, n_seps) = separators(&csv, separator);
    assert_eq!(n_lines, limit as usize);
    assert_eq!(n_seps, (limit as usize + 1) * (schema.fields().len() - 1));
}

#[test]
fn test_csv_with_comma_new() {
    let schema: Schema = Schema::of(vec![field("values", || "1,2,3"), field("title", || "The \"fabulous\" artist")]);
    let csv = CsvTransformer::builder().header(true).separator(",").build().generate(&schema, 1).unwrap();
    assert_eq!(csv, format!("\"values\",\"title\"{LINE_SEPARATOR}\"1,2,3\",\"The \"\"fabulous\"\" artist\""));
}

#[test]
fn test_csv_with_different_objects() {
    let faker = Faker::with_seed(10);
    let schema: Schema = Schema::of(vec![
        field("Number", || faker.number().random_digit()),
        field("Bool", || faker.bool().bool()),
        field("String", || faker.name().first_name()),
        field("Text", || "The, \"fabulous\" artist'"),
    ]);
    let csv = CsvTransformer::builder().header(true).separator(",").build().generate(&schema, 4).unwrap();
    let expected = [
        "\"Number\",\"Bool\",\"String\",\"Text\"",
        "3,false,\"Flor\",\"The, \"\"fabulous\"\" artist'\"",
        "6,true,\"Stephnie\",\"The, \"\"fabulous\"\" artist'\"",
        "1,false,\"Edythe\",\"The, \"\"fabulous\"\" artist'\"",
        "1,true,\"Dwight\",\"The, \"\"fabulous\"\" artist'\"",
    ]
    .join(LINE_SEPARATOR);
    assert_eq!(csv, expected);
}

fn number_password_schema(faker: &Faker) -> Schema<'_, i32> {
    Schema::of(vec![field_of("Number", |i: &i32| *i), field_of("Password", move |i: &i32| password(faker, *i))])
}

#[test]
fn test_csv_with_different_objects_function() {
    let faker = Faker::with_seed(10);
    let schema = number_password_schema(&faker);
    let transformer = CsvTransformer::builder().header(true).separator(",").build();
    let sequence = faker.collection().supplier(|| faker.number().random_digit()).len(5).build().unwrap();
    let csv = transformer.generate_from(&sequence, &schema).unwrap();
    let expected =
        ["\"Number\",\"Password\"", "3,\"nf3\"", "6,\"4b0v69\"", "7,\"00827v2\"", "1,\"5\"", "3,\"p6x\""].join(LINE_SEPARATOR);
    assert_eq!(csv, expected);
}

#[test]
fn test_csv_with_different_objects_function_stream() {
    let faker = Faker::with_seed(10);
    let schema = number_password_schema(&faker);
    let transformer = CsvTransformer::builder().header(true).separator(",").build();
    let sequence = faker.stream().supplier(|| faker.number().random_digit()).len(3).build().unwrap();
    let csv = transformer.generate_from(&sequence, &schema).unwrap();
    let expected = ["\"Number\",\"Password\"", "3,\"0p4\"", "8,\"714487nf\"", "5,\"0v691\""].join(LINE_SEPARATOR);
    assert_eq!(csv, expected);
}

const INFINITE: &str = "The sequence should be finite of size: FakeStream{minLength=-1, maxLength=-1, nullRate=0.0}";

#[test]
fn test_csv_with_infinite_sequence() {
    let faker = Faker::with_seed(10);
    let schema: Schema<i32> = Schema::of(vec![
        field_of("Number", |i: &i32| faker.number().digits(*i as usize)),
        field_of("String", |prefix: &i32| Ok::<_, datafaker::Error>(format!("{prefix}: {}", faker.name().first_name()?))),
    ]);
    let transformer = CsvTransformer::builder().header(true).separator(",").build();
    let sequence = faker.stream().supplier(|| faker.number().random_digit()).build().unwrap();
    let e = transformer.generate_from(&sequence, &schema).unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", INFINITE));
}

fn name_schema<'a>(last: &str) -> Schema<'a, Name<'a>> {
    Schema::of(vec![field_of("firstName", |n: &Name| n.first_name()), field_of(last.to_string(), |n: &Name| n.last_name())])
}

#[test]
fn test_limit_for_csv() {
    for limit in [0, 2, 3, 10, 20, 100] {
        let faker = Faker::new();
        let sequence = faker.collection().supplier(|| faker.name()).max_len(limit + 1).build().unwrap();
        let csv = CsvTransformer::builder().separator(" : ").header(false).build().generate_from(&sequence, &name_schema("lastname")).unwrap();
        assert_eq!(lines(&csv), limit as usize);
    }
}

#[test]
fn test_limit_for_csv_new() {
    for limit in [0, 2, 3, 10, 20, 100] {
        let faker = Faker::new();
        let sequence = faker.collection().supplier(|| faker.name()).max_len(limit + 1).build().unwrap();
        let csv = CsvTransformer::builder().header(false).separator(",").build().generate_from(&sequence, &name_schema("lastname")).unwrap();
        assert_eq!(lines(&csv), limit as usize);
    }
}

#[test]
fn test_limit_for_collection() {
    for limit in [0, 2, 3, 10, 20] {
        let faker = Faker::new();
        let sequence = faker.collection().supplier(|| faker.name()).max_len(limit).build().unwrap();
        let csv = CsvTransformer::builder().header(false).build().generate_from(&sequence, &name_schema("lastName")).unwrap();
        let n = lines(&csv);
        assert_eq!(if n == 0 { 0 } else { n + 1 }, limit as usize);
    }
}

#[test]
fn test_limit_for_collection_new() {
    for limit in [0, 2, 3, 10, 20, 100] {
        let faker = Faker::new();
        let sequence = faker.collection().supplier(|| faker.name()).max_len(limit + 1).build().unwrap();
        let csv = CsvTransformer::builder().header(false).separator(" : ").build().generate_from(&sequence, &name_schema("lastname")).unwrap();
        assert_eq!(lines(&csv), limit as usize);
    }
}

#[test]
fn test_infinite_csv() {
    let faker = Faker::new();
    let sequence = faker.stream().supplier(|| faker.name()).build().unwrap();
    let e = CsvTransformer::builder().separator(" : ").header(false).build().generate_from(&sequence, &name_schema("lastName")).unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", INFINITE));
}

#[test]
fn test_infinite_csv_with_limit() {
    let limit = 10;
    let faker = Faker::new();
    let sequence = faker.stream().supplier(|| faker.name()).len(limit).build().unwrap();
    let csv = CsvTransformer::builder().header(false).separator(" : ").build().generate_from(&sequence, &name_schema("lastName")).unwrap();
    assert_eq!(lines(&csv) + 1, limit as usize);
}

#[test]
fn supplier_should_be_defined_in_case_of_null_input() {
    let e = CsvTransformer::builder().header(false).separator(" : ").build().generate(&name_schema("lastname"), 1).unwrap_err();
    assert_eq!(e.simple_class(), "IllegalArgumentException");
    assert!(e.message().contains("Input could be null only if suppliers are defined"));
}

// JsonTest

fn text_bool_schema(faker: &Faker) -> Schema<'_> {
    Schema::of(vec![field("Text", || faker.name().first_name()), field("Bool", || faker.bool().bool())])
}

#[test]
fn test_json_stream() {
    let faker = Faker::with_seed(10);
    let schema = text_bool_schema(&faker);
    let transformer = JsonTransformer::builder().build();
    let output = transformer.generate_stream(&schema, 10).unwrap().collect::<Result<Vec<_>>>().unwrap().join(LINE_SEPARATOR);
    let rows = [
        ("Willis", false),
        ("Carlena", true),
        ("Stephnie", true),
        ("Rutha", true),
        ("Armand", true),
        ("Margot", false),
        ("Patrick", false),
        ("Alphonse", false),
        ("Louisa", true),
        ("Caryn", false),
    ];
    let body = rows.iter().map(|(t, b)| format!("{{\"Text\": \"{t}\", \"Bool\": {b}}}")).collect::<Vec<_>>().join(&format!(",{LINE_SEPARATOR}"));
    assert_eq!(output, format!("[{LINE_SEPARATOR}{body}{LINE_SEPARATOR}]"));
}

#[test]
fn test_generate_from_schema_with_limit() {
    let faker = Faker::with_seed(10);
    let json = JsonTransformer::builder().build().generate(&text_bool_schema(&faker), 2).unwrap();
    let expected = format!(
        "[{0}{{\"Text\": \"Willis\", \"Bool\": false}},{0}{{\"Text\": \"Carlena\", \"Bool\": true}}{0}]",
        LINE_SEPARATOR
    );
    assert_eq!(json, expected);
}

fn json_rows(rows: &[(i32, &str)], comma: bool) -> String {
    let sep = if comma { format!(",{LINE_SEPARATOR}") } else { LINE_SEPARATOR.to_string() };
    let body = rows.iter().map(|(n, p)| format!("{{\"Number\": {n}, \"Password\": \"{p}\"}}")).collect::<Vec<_>>().join(&sep);
    format!("[{LINE_SEPARATOR}{body}{LINE_SEPARATOR}]")
}

const COLLECTION_ROWS: [(i32, &str); 5] = [(3, "nf3"), (6, "4b0v69"), (7, "00827v2"), (1, "5"), (3, "p6x")];

#[test]
fn test_generate_from_fake_sequence_collection_without_comma() {
    let faker = Faker::with_seed(10);
    let schema = number_password_schema(&faker);
    let sequence = faker.collection().supplier(|| faker.number().random_digit()).len(5).build().unwrap();
    let json = JsonTransformer::builder().with_comma_between_objects(false).build().generate_from(&sequence, &schema).unwrap();
    assert_eq!(json, json_rows(&COLLECTION_ROWS, false));
}

#[test]
fn test_generate_from_fake_sequence_collection() {
    let faker = Faker::with_seed(10);
    let schema = number_password_schema(&faker);
    let sequence = faker.collection().supplier(|| faker.number().random_digit()).len(5).build().unwrap();
    let json = JsonTransformer::builder().build().generate_from(&sequence, &schema).unwrap();
    assert_eq!(json, json_rows(&COLLECTION_ROWS, true));
}

#[test]
fn test_generate_from_fake_sequence_stream() {
    let faker = Faker::with_seed(10);
    let schema = number_password_schema(&faker);
    let sequence = faker.stream().supplier(|| faker.number().random_digit()).len(2).build().unwrap();
    let json = JsonTransformer::builder().with_comma_between_objects(false).build().generate_from(&sequence, &schema).unwrap();
    assert_eq!(json, json_rows(&[(3, "0p4"), (8, "714487nf")], false));
}

#[test]
fn test_generate_from_infinite_fake_sequence() {
    let faker = Faker::with_seed(10);
    let schema = number_password_schema(&faker);
    let sequence = faker.stream().supplier(|| faker.number().random_digit()).build().unwrap();
    let e = JsonTransformer::builder().build().generate_from(&sequence, &schema).unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", INFINITE));
}

fn json_test_schemas() -> Vec<(Schema<'static>, &'static str)> {
    vec![
        (Schema::of(vec![]), "{}"),
        (Schema::of(vec![composite_field("key", vec![field("key", || "value")])]), "{\"key\": {\"key\": \"value\"}}"),
        (Schema::of(vec![field("key", || "value")]), "{\"key\": \"value\"}"),
        (Schema::of(vec![field("number", || 123)]), "{\"number\": 123}"),
        (Schema::of(vec![field("number", || 123.0)]), "{\"number\": 123.0}"),
        (Schema::of(vec![field("number", || 123.123)]), "{\"number\": 123.123}"),
        (Schema::of(vec![field("boolean", || true)]), "{\"boolean\": true}"),
        (Schema::of(vec![field("nullValue", || ())]), "{\"nullValue\": null}"),
        (Schema::of(vec![field("array", || Value::array([None, Some("test"), Some("123")]))]), "{\"array\": [null, \"test\", \"123\"]}"),
        (Schema::of(vec![field("array", || Value::array([123, 456, 789]))]), "{\"array\": [123, 456, 789]}"),
        (
            Schema::of(vec![field("array", || Value::array([Value::Str("test".into()), Value::Int(456), Value::Bool(true)]))]),
            "{\"array\": [\"test\", 456, true]}",
        ),
        (Schema::of(vec![field("emptyarray", || Value::array(Vec::<i64>::new()))]), "{\"emptyarray\": []}"),
        (Schema::of(vec![field("emptyarray", Vec::<String>::new)]), "{\"emptyarray\": []}"),
        (Schema::of(vec![field("es\"ca\"ped", || "va\"lu\"e")]), "{\"es\\\"ca\\\"ped\": \"va\\\"lu\\\"e\"}"),
        (
            Schema::of(vec![field("key", || "value"), composite_field("nested", vec![field("nestedkey", || "nestedvalue")])]),
            "{\"key\": \"value\", \"nested\": {\"nestedkey\": \"nestedvalue\"}}",
        ),
    ]
}

#[test]
fn simple_json_test_for_json_transformer() {
    for (schema, expected) in json_test_schemas() {
        assert_eq!(JsonTransformer::builder().build().generate(&schema, 1).unwrap(), expected);
    }
}

#[test]
fn output_array_json_test_for_json_transformer() {
    for (schema, expected) in json_test_schemas() {
        let json = JsonTransformer::builder().build().generate(&schema, 2).unwrap().replace(LINE_SEPARATOR, "");
        assert_eq!(json, format!("[{expected},{expected}]"));
    }
}

#[test]
fn output_without_comma_for_json_transformer() {
    for (schema, expected) in json_test_schemas() {
        let json = JsonTransformer::builder().with_comma_between_objects(false).build().generate(&schema, 2).unwrap().replace(LINE_SEPARATOR, "");
        assert_eq!(json, format!("[{expected}{expected}]"));
    }
}

#[test]
fn json_with_different_field_formats_in_one_object_test() {
    let faker = Faker::with_seed(10);
    let limit = 2;
    let sequence = faker.collection().min_len(limit).max_len(limit).supplier(|| faker.name()).build().unwrap();
    let schema: Schema<Name> = Schema::of(vec![
        field("text", || faker.name().first_name()),
        field("array", || faker.collection().supplier(|| faker.phone_number().phone_number()).max_len(3).generate()),
    ]);
    let json = JsonTransformer::builder().build().generate_from(&sequence, &schema).unwrap();
    assert_eq!(count(&json, "},"), limit as usize - 1);
}

fn country_city() -> Vec<Field<'static>> {
    vec![
        composite_field(None::<&str>, vec![field("country", || "Denmark"), field("city", || "Port Angel")]),
        composite_field(None::<&str>, vec![field("two", || "Denmark"), field("one", || "Port Angel")]),
    ]
}

#[test]
fn json_object_collection_test() {
    let schema: Schema = Schema::of(vec![field("text", || "Mrs. Brian Braun"), field("objectCollection", country_city)]);
    let json = JsonTransformer::builder().build().generate(&schema, 1).unwrap();
    assert_eq!(
        json,
        "{\"text\": \"Mrs. Brian Braun\", \"objectCollection\": [{\"country\": \"Denmark\", \"city\": \"Port Angel\"}, {\"two\": \"Denmark\", \"one\": \"Port Angel\"}]}"
    );
}

#[test]
fn json_collection_of_collections_test() {
    let schema: Schema = Schema::of(vec![field("text", || "Mrs. Brian Braun"), field("objectCollection", || vec![vec![country_city()]])]);
    let json = JsonTransformer::builder().build().generate(&schema, 1).unwrap();
    assert_eq!(
        json,
        "{\"text\": \"Mrs. Brian Braun\", \"objectCollection\": [[[{\"country\": \"Denmark\", \"city\": \"Port Angel\"}, {\"two\": \"Denmark\", \"one\": \"Port Angel\"}]]]}"
    );
}

// SqlTest

fn credentials_schema(faker: &Faker) -> Schema<'_, i32> {
    Schema::of(vec![
        field_of("Number", |i: &i32| *i),
        field_of("Password", move |i: &i32| faker.credentials().password_with(*i, *i, false, false, true)),
    ])
}

#[test]
fn generate_from_fake_sequence_separated() {
    let faker = Faker::with_seed(10);
    let schema = credentials_schema(&faker);
    let sequence = faker.collection().supplier(|| faker.number().random_digit()).len(5).build().unwrap();
    let sql = SqlTransformer::builder().build().unwrap().generate_from(&sequence, &schema).unwrap();
    let expected = COLLECTION_ROWS
        .iter()
        .map(|(n, p)| format!("INSERT INTO \"MyTable\" (\"Number\", \"Password\") VALUES ({n}, '{p}');"))
        .collect::<Vec<_>>()
        .join(LINE_SEPARATOR);
    assert_eq!(sql, expected);
}

#[test]
fn generate_from_fake_sequence_batch() {
    let faker = Faker::with_seed(10);
    let schema = credentials_schema(&faker);
    let sequence = faker.collection().supplier(|| faker.number().random_digit()).len(5).build().unwrap();
    let sql = SqlTransformer::builder().batch().build().unwrap().generate_from(&sequence, &schema).unwrap();
    let expected = [
        "INSERT INTO \"MyTable\" (\"Number\", \"Password\")",
        "VALUES (3, 'nf3'),",
        "       (6, '4b0v69'),",
        "       (7, '00827v2'),",
        "       (1, '5'),",
        "       (3, 'p6x');",
    ]
    .join(LINE_SEPARATOR);
    assert_eq!(sql, expected);
}

#[test]
fn generate_from_empty_schema() {
    let faker = Faker::new();
    let schema: Schema<i32> = Schema::of(vec![]);
    let sequence = faker.stream().supplier(|| faker.number().random_digit()).build().unwrap();
    assert!(SqlTransformer::builder().build().unwrap().generate_from(&sequence, &schema).unwrap().is_empty());
}

#[test]
fn generate_from_infinite_fake_sequence_batch() {
    let faker = Faker::with_seed(10);
    let schema = credentials_schema(&faker);
    let sequence = faker.stream().supplier(|| faker.number().random_digit()).build().unwrap();
    let e = SqlTransformer::builder().batch().build().unwrap().generate_from(&sequence, &schema).unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", INFINITE));
}

#[test]
fn test_generate_from_schema_with_limit_separated_statements() {
    let faker = Faker::with_seed(10);
    let sql = SqlTransformer::builder().build().unwrap().generate(&text_bool_schema(&faker), 2).unwrap();
    let expected = [
        "INSERT INTO \"MyTable\" (\"Text\", \"Bool\") VALUES ('Willis', false);",
        "INSERT INTO \"MyTable\" (\"Text\", \"Bool\") VALUES ('Carlena', true);",
    ]
    .join(LINE_SEPARATOR);
    assert_eq!(sql, expected);
}

#[test]
fn test_generate_from_schema_with_limit_batch_mode_statements() {
    let faker = Faker::with_seed(10);
    let sql = SqlTransformer::builder().batch().build().unwrap().generate(&text_bool_schema(&faker), 2).unwrap();
    let expected = ["INSERT INTO \"MyTable\" (\"Text\", \"Bool\")", "VALUES ('Willis', false),", "       ('Carlena', true);"].join(LINE_SEPARATOR);
    assert_eq!(sql, expected);
}

#[test]
fn test_force_quoted_with_sql_identifiers() {
    let faker = Faker::with_seed(10);
    let schema: Schema = Schema::of(vec![field("TEXT", || faker.name().first_name()), field("BOOL", || faker.bool().bool())]);
    let transformer = SqlTransformer::builder().table_name("MY_TABLE").force_use_sql_quote_identifier().batch().build().unwrap();
    let sql = transformer.generate(&schema, 2).unwrap();
    let expected = ["INSERT INTO \"MY_TABLE\" (\"TEXT\", \"BOOL\")", "VALUES ('Willis', false),", "       ('Carlena', true);"].join(LINE_SEPARATOR);
    assert_eq!(sql, expected);
}

type SqlCase = (Schema<'static>, Option<&'static str>, &'static str);

fn sql_cases(expected: [&'static str; 9], schema_names: [Option<&'static str>; 9]) -> Vec<SqlCase> {
    let schemas: Vec<Schema<'static>> = vec![
        Schema::of(vec![]),
        Schema::of(vec![field("key", || "value")]),
        Schema::of(vec![field("number", || 123)]),
        Schema::of(vec![field("number", || 123.0)]),
        Schema::of(vec![field("number", || 123.123)]),
        Schema::of(vec![field("boolean", || true)]),
        Schema::of(vec![field("nullValue", || ())]),
        Schema::of(vec![field("nullValue", || ())]),
        Schema::of(vec![field("nullValue", || ())]),
    ];
    schemas.into_iter().zip(schema_names).zip(expected).map(|((s, n), e)| (s, n, e)).collect()
}

#[test]
fn simple_sql_test_for_sql_transformer() {
    let cases = sql_cases(
        [
            "",
            "INSERT INTO MY_TABLE (`key`) VALUES ('value');",
            "INSERT INTO MY_TABLE (`number`) VALUES (123);",
            "INSERT INTO MY_TABLE (`number`) VALUES (123.0);",
            "INSERT INTO MY_TABLE (`number`) VALUES (123.123);",
            "INSERT INTO MY_TABLE (`boolean`) VALUES (true);",
            "INSERT INTO MY_TABLE (`nullValue`) VALUES (null);",
            "INSERT INTO `My.SCHEMA`.MY_TABLE (`nullValue`) VALUES (null);",
            "INSERT INTO MY_SCHEMA.MY_TABLE (`nullValue`) VALUES (null);",
        ],
        [None, Some(""), Some(""), None, None, Some(""), None, Some("My.SCHEMA"), Some("MY_SCHEMA")],
    );
    for (schema, schema_name, expected) in cases {
        let transformer = SqlTransformer::builder().sql_quote_identifier("`").schema_name(schema_name).table_name("MY_TABLE").build().unwrap();
        assert_eq!(transformer.generate(&schema, 1).unwrap(), expected);
    }
}

fn dialect_cases(dialect: SqlDialect, open: &str, close: &str, table: &str, schema_names: [Option<&'static str>; 9]) {
    let expected: Vec<String> = [
        None,
        Some(("key", "'value'")),
        Some(("number", "123")),
        Some(("number", "123.0")),
        Some(("number", "123.123")),
        Some(("boolean", "true")),
        Some(("nullValue", "null")),
        Some(("nullValue", "null")),
        Some(("nullValue", "null")),
    ]
    .iter()
    .zip(schema_names)
    .map(|(c, schema_name)| match c {
        None => String::new(),
        Some((column, value)) => {
            let prefix = schema_name.filter(|s| !s.is_empty()).map(|s| format!("{open}{s}{close}.")).unwrap_or_default();
            format!("INSERT INTO {prefix}{open}{table}{close} ({open}{column}{close}) VALUES ({value});")
        }
    })
    .collect();
    let leaked: Vec<&'static str> = expected.into_iter().map(|s| &*Box::leak(s.into_boxed_str())).collect();
    for (schema, schema_name, expected) in sql_cases(leaked.try_into().unwrap(), schema_names) {
        let transformer = SqlTransformer::builder().schema_name(schema_name).dialect(dialect).build().unwrap();
        assert_eq!(transformer.generate(&schema, 1).unwrap(), expected, "{dialect:?}");
    }
}

#[test]
fn simple_sql_test_for_sql_transformer_oracle() {
    dialect_cases(SqlDialect::Oracle, "\"", "\"", "MyTable", [None, Some(""), None, None, Some(""), None, None, None, Some("MySchema")]);
}

#[test]
fn simple_sql_test_for_sql_transformer_postgres() {
    dialect_cases(SqlDialect::Postgres, "\"", "\"", "MyTable", [None, Some(""), None, None, Some(""), None, None, None, Some("MySchema")]);
}

#[test]
fn simple_sql_test_for_sql_transformer_mssql() {
    dialect_cases(SqlDialect::Mssql, "[", "]", "MyTable", [None, Some(""), None, None, Some(""), Some(""), None, None, Some("MySchema")]);
}

#[test]
fn simple_sql_test_for_sql_transformer_my_sql() {
    dialect_cases(SqlDialect::Mysql, "", "", "MyTable", [None, Some(""), Some(""), None, None, None, None, None, Some("MySchema")]);
}

fn first_last(faker: &Faker) -> Schema<'_> {
    Schema::of(vec![field("firstName", || faker.name().first_name()), field("lastName", || faker.name().last_name())])
}

#[test]
fn batch_sql_test_for_sql_transformer_postgres() {
    let faker = Faker::new();
    let output = SqlTransformer::builder().batch().dialect(SqlDialect::Postgres).build().unwrap().generate(&first_last(&faker), 5).unwrap();
    assert_eq!(output.split(LINE_SEPARATOR).count(), 5 + 1);
}

#[test]
fn sql_keyword_case_check() {
    let faker = Faker::new();
    let transformer = SqlTransformer::builder().keyword_case(Case::Lowercase).dialect(SqlDialect::Postgres).build().unwrap();
    let sql = transformer.generate(&first_last(&faker), 1).unwrap();
    assert!(sql.contains("insert into ") && !sql.contains("INSERT INTO ") && sql.contains("values ") && !sql.contains("VALUES"));
}

#[test]
fn sql_keyword_capital_case_check() {
    let faker = Faker::new();
    let transformer = SqlTransformer::builder().keyword_case(Case::Capital).dialect(SqlDialect::Postgres).build().unwrap();
    let sql = transformer.generate(&first_last(&faker), 1).unwrap();
    assert!(sql.contains("Insert Into ") && !sql.contains("INSERT INTO ") && !sql.contains("insert into"));
    assert!(sql.contains("Values ") && !sql.contains("VALUES") && !sql.contains("values"));
}

/// `String.split(regex)` drops trailing empty strings.
fn java_split_count(s: &str, sep: &str) -> usize {
    let mut parts: Vec<&str> = s.split(sep).collect();
    while parts.len() > 1 && parts.last() == Some(&"") {
        parts.pop();
    }
    parts.len()
}

#[test]
fn batch_sql_test_for_sql_transformer_oracle() {
    let faker = Faker::new();
    let schema = first_last(&faker);
    let output = SqlTransformer::builder().batch().dialect(SqlDialect::Oracle).build().unwrap().generate(&schema, 5).unwrap();
    assert_eq!(java_split_count(&output, LINE_SEPARATOR), 5 + 2);
    assert!(output.contains("INSERT ALL") && output.contains("INTO") && !output.contains("INSERT INTO") && output.contains("SELECT 1 FROM dual;"));
    let transformer = SqlTransformer::builder().batch().keyword_case(Case::Lowercase).dialect(SqlDialect::Oracle).build().unwrap();
    let output = transformer.generate(&schema, 5).unwrap();
    assert_eq!(java_split_count(&output, LINE_SEPARATOR), 5 + 2);
    assert!(output.contains("insert all") && output.contains("into") && !output.contains("insert into") && output.contains("select 1 from dual;"));
}

#[test]
fn batch_size_sql_test_for_sql_transformer_oracle() {
    let faker = Faker::new();
    let output = SqlTransformer::builder().batch_of(2).dialect(SqlDialect::Oracle).build().unwrap().generate(&first_last(&faker), 5).unwrap();
    assert_eq!(java_split_count(&output, "INSERT ALL"), 4);
    assert_eq!(java_split_count(&output, "SELECT 1 FROM dual"), 4);
}

#[test]
fn batch_size_sql_test_for_sql_transformer_postgres() {
    let faker = Faker::new();
    let output = SqlTransformer::builder().batch_of(2).dialect(SqlDialect::Postgres).build().unwrap().generate(&first_last(&faker), 5).unwrap();
    assert_eq!(java_split_count(&output, "INSERT INTO"), 4);
    assert_eq!(java_split_count(&output, "VALUES "), 4);
}

fn ints(v: &[i32]) -> Value<'static> {
    Value::Primitive(PrimitiveArray::Int(v.to_vec()))
}

#[test]
fn array_and_multiset_sql_test_for_sql_transformer_calcite() {
    let cases: Vec<(Schema<'static>, Option<&str>, &str)> = vec![
        (Schema::of(vec![]), None, ""),
        (Schema::of(vec![field("ints", || ints(&[1, 2, 3]))]), Some(""), "INSERT INTO \"MyTable\" (\"ints\") VALUES (ARRAY[1, 2, 3]);"),
        (Schema::of(vec![field("longs", || PrimitiveArray::Long(vec![23, 45]))]), None, "INSERT INTO \"MyTable\" (\"longs\") VALUES (ARRAY[23, 45]);"),
        (Schema::of(vec![field("bytes", || PrimitiveArray::Byte(vec![1, 0]))]), None, "INSERT INTO \"MyTable\" (\"bytes\") VALUES (ARRAY[1, 0]);"),
        (Schema::of(vec![field("shorts", || PrimitiveArray::Short(vec![1, 0, 3]))]), None, "INSERT INTO \"MyTable\" (\"shorts\") VALUES (ARRAY[1, 0, 3]);"),
        (
            Schema::of(vec![field("booleans", || PrimitiveArray::Boolean(vec![true, false]))]),
            None,
            "INSERT INTO \"MyTable\" (\"booleans\") VALUES (ARRAY[true, false]);",
        ),
        (
            Schema::of(vec![field("floats", || PrimitiveArray::Float(vec![1.0, 0.0, 3.0]))]),
            None,
            "INSERT INTO \"MyTable\" (\"floats\") VALUES (ARRAY[1.0, 0.0, 3.0]);",
        ),
        (
            Schema::of(vec![field("doubles", || PrimitiveArray::Double(vec![1.0, 5.0, 3.0]))]),
            None,
            "INSERT INTO \"MyTable\" (\"doubles\") VALUES (ARRAY[1.0, 5.0, 3.0]);",
        ),
        (Schema::of(vec![field("names", || Value::array(["hello", "world"]))]), None, "INSERT INTO \"MyTable\" (\"names\") VALUES (ARRAY['hello', 'world']);"),
        (Schema::of(vec![field("names_list", || vec!["hello", "hello"])]), Some(""), "INSERT INTO \"MyTable\" (\"names_list\") VALUES (MULTISET['hello', 'hello']);"),
        (Schema::of(vec![field("names_multiset", || vec!["hello"])]), Some(""), "INSERT INTO \"MyTable\" (\"names_multiset\") VALUES (MULTISET['hello']);"),
        (
            Schema::of(vec![field("ints_ints", || Value::array([ints(&[1]), Value::Null, ints(&[3, 4, 5])]))]),
            Some(""),
            "INSERT INTO \"MyTable\" (\"ints_ints\") VALUES (ARRAY[ARRAY[1], NULL, ARRAY[3, 4, 5]]);",
        ),
        (
            Schema::of(vec![field("ints_ints", || Value::array([ints(&[1]), ints(&[2]), ints(&[3, 4, 5])]))]),
            Some(""),
            "INSERT INTO \"MyTable\" (\"ints_ints\") VALUES (ARRAY[ARRAY[1], ARRAY[2], ARRAY[3, 4, 5]]);",
        ),
        (
            Schema::of(vec![field("multiset", || vec![vec![vec!["value"]]])]),
            Some(""),
            "INSERT INTO \"MyTable\" (\"multiset\") VALUES (MULTISET[MULTISET[MULTISET['value']]]);",
        ),
        (
            Schema::of(vec![field("multiset_array", || vec![ints(&[1, 2])])]),
            Some(""),
            "INSERT INTO \"MyTable\" (\"multiset_array\") VALUES (MULTISET[ARRAY[1, 2]]);",
        ),
        (
            Schema::of(vec![field("array_multiset", || Value::array([vec!["value"]]))]),
            Some(""),
            "INSERT INTO \"MyTable\" (\"array_multiset\") VALUES (ARRAY[MULTISET['value']]);",
        ),
        (Schema::of(vec![composite_field("row", vec![field("name", || "2")])]), None, "INSERT INTO \"MyTable\" (\"row\") VALUES (ROW('2'));"),
        (
            Schema::of(vec![composite_field("row_row", vec![field("name1", || "1"), composite_field("row", vec![field("name", || "2")])])]),
            None,
            "INSERT INTO \"MyTable\" (\"row_row\") VALUES (ROW('1', ROW('2')));",
        ),
        (
            Schema::of(vec![composite_field(
                "row_array",
                vec![field("name1", || "1"), composite_field("row", vec![field("name", || ints(&[1, 2, 3]))])],
            )]),
            None,
            "INSERT INTO \"MyTable\" (\"row_array\") VALUES (ROW('1', ROW(ARRAY[1, 2, 3])));",
        ),
    ];
    for (schema, schema_name, expected) in cases {
        let transformer = SqlTransformer::builder().schema_name(schema_name).dialect(SqlDialect::Calcite).build().unwrap();
        assert_eq!(transformer.generate(&schema, 1).unwrap(), expected);
    }
}

#[test]
fn batch_test_for_sql_transformer_spark_sql() {
    let (batch_size, records) = (5, 20);
    let faker = Faker::new();
    let transformer = SqlTransformer::builder().dialect(SqlDialect::Sparksql).batch_of(batch_size).build().unwrap();
    let schema: Schema = Schema::of(vec![field("name", || faker.cat().name()), field("breed", || faker.cat().breed())]);
    let generation = transformer.generate(&schema, records).unwrap();
    assert_eq!(java_split_count(&generation, "INSERT INTO"), (records / batch_size) as usize + 1);
    assert_eq!(java_split_count(&generation, "VALUES"), (records / batch_size) as usize + 1);
    assert_eq!(java_split_count(&generation, ";"), (records / batch_size) as usize);
}

fn spark_sql_cases() -> Vec<(Schema<'static>, &'static str)> {
    let small_map = || Value::map([("k1", "v1")]);
    vec![
        (Schema::of(vec![]), ""),
        (Schema::of(vec![field("bytes", || PrimitiveArray::Byte(vec![1, 0]))]), "INSERT INTO `MyTable` (`bytes`) VALUES (ARRAY(1, 0));"),
        (Schema::of(vec![field("booleans", || PrimitiveArray::Boolean(vec![true, false]))]), "INSERT INTO `MyTable` (`booleans`) VALUES (ARRAY(true, false));"),
        (Schema::of(vec![field("ints", || ints(&[1, 2, 3]))]), "INSERT INTO `MyTable` (`ints`) VALUES (ARRAY(1, 2, 3));"),
        (Schema::of(vec![field("longs", || PrimitiveArray::Long(vec![23, 45]))]), "INSERT INTO `MyTable` (`longs`) VALUES (ARRAY(23, 45));"),
        (Schema::of(vec![field("empty_map", || Value::map(Vec::<(String, String)>::new()))]), "INSERT INTO `MyTable` (`empty_map`) VALUES (MAP());"),
        (
            Schema::of(vec![field("maps", move || Value::map([("k1", small_map()), ("k2", small_map())]))]),
            "INSERT INTO `MyTable` (`maps`) VALUES (MAP('k1', MAP('k1', 'v1'), 'k2', MAP('k1', 'v1')));",
        ),
        (
            Schema::of(vec![composite_field(
                "struct_array",
                vec![field("name1", || "1"), composite_field("struct", vec![field("name", || ints(&[1, 2, 3]))])],
            )]),
            "INSERT INTO `MyTable` (`struct_array`) VALUES (NAMED_STRUCT('name1', '1', 'struct', NAMED_STRUCT('name', ARRAY(1, 2, 3))));",
        ),
        (
            Schema::of(vec![composite_field("struct_struct", vec![field("name1", || "1"), composite_field("struct", vec![field("name", || "2")])])]),
            "INSERT INTO `MyTable` (`struct_struct`) VALUES (NAMED_STRUCT('name1', '1', 'struct', NAMED_STRUCT('name', '2')));",
        ),
    ]
}

#[test]
fn simple_sql_test_for_sql_transformer_spark_sql() {
    for (schema, expected) in spark_sql_cases() {
        let transformer = SqlTransformer::builder().schema_name(None::<&str>).dialect(SqlDialect::Sparksql).build().unwrap();
        assert_eq!(transformer.generate(&schema, 1).unwrap(), expected);
    }
}

#[test]
fn simple_stream_test_for_sql_transformer_spark_sql() {
    for (schema, expected) in spark_sql_cases() {
        let transformer = SqlTransformer::builder().schema_name(None::<&str>).dialect(SqlDialect::Sparksql).build().unwrap();
        let sql = transformer.generate_stream(&schema, 1).unwrap().collect::<Result<Vec<_>>>().unwrap().join(LINE_SEPARATOR);
        assert_eq!(sql, expected);
    }
}

#[test]
fn test_sql_batch() {
    let faker = Faker::with_seed(10);
    let schema: Schema = Schema::of(vec![field("Number", || faker.number().digit()), field("Password", || faker.internet().uuidv3())]);
    let transformer = SqlTransformer::builder().batch_of(3).build().unwrap();
    let sql = transformer.generate_stream(&schema, 4).unwrap().collect::<Result<Vec<_>>>().unwrap().join(LINE_SEPARATOR);
    let expected = [
        "INSERT INTO \"MyTable\" (\"Number\", \"Password\")",
        "VALUES ('6', '09fd4007-40ba-39df-8cb1-65926bf7b8a9'),",
        "       ('8', '96c19757-1f18-3051-9acb-f56f0b5555ae'),",
        "       ('2', '8a4a0365-cd39-33c1-a52a-279b1076cf2d');",
        "INSERT INTO \"MyTable\" (\"Number\", \"Password\")",
        "VALUES ('6', 'e807efdd-b6db-319d-8342-a044274d3417');",
    ]
    .join(LINE_SEPARATOR);
    assert_eq!(sql, expected);
}

/// `new Locale("test")` reads `test.yml` from upstream's test resources, added here with `addPath`.
#[test]
fn test_word_truncated() {
    let locale = Locale::language_only("test");
    let faker = Faker::with_locale_and_seed(locale.clone(), 10);
    faker.add_path(&locale, concat!(env!("CARGO_MANIFEST_DIR"), "/tests/resources/test.yml")).unwrap();
    let schema: Schema = Schema::of(vec![field("Address", || faker.address().country()), field("Book", || faker.book().title())]);
    let transformer = SqlTransformer::builder().batch_of(4).build().unwrap();
    let sql = transformer.generate_stream(&schema, 3).unwrap().collect::<Result<Vec<_>>>().unwrap().join(LINE_SEPARATOR);
    let expected = [
        "INSERT INTO \"MyTable\" (\"Address\", \"Book\")",
        "VALUES ('Cote d\\'Ivoire', 'All the King\\'s Men'),",
        "       ('Cote d\\'Ivoire', 'Blood\\'s a Rover'),",
        "       ('Cote d\\'Ivoire', 'Blood\\'s a Rover');",
    ]
    .join(LINE_SEPARATOR);
    assert_eq!(sql, expected);
}

// TomlTest

fn toml_and_yaml_cases() -> Vec<Schema<'static>> {
    vec![
        Schema::of(vec![]),
        Schema::of(vec![field("key", || "value")]),
        Schema::of(vec![field("number", || 123)]),
        Schema::of(vec![field("number", || Value::BigDecimal("123.0".into()))]),
        Schema::of(vec![field("number", || Value::BigDecimal("123.123".into()))]),
        Schema::of(vec![field("boolean", || true)]),
        Schema::of(vec![field("nullValue", || ())]),
        Schema::of(vec![field("array", || Value::array([None, Some("test"), Some("123")]))]),
        Schema::of(vec![field("array", || Value::array([123, 456, 789]))]),
        Schema::of(vec![field("array", || Value::array([Value::Str("test".into()), Value::Int(456), Value::Bool(true)]))]),
        Schema::of(vec![field("emptyarray", || Value::array(Vec::<i64>::new()))]),
        Schema::of(vec![field("emptyarray", Vec::<String>::new)]),
        Schema::of(vec![field("key", || "value"), field("nested", || Schema::of(vec![field("nestedkey", || "nestedvalue")]))]),
        Schema::of(vec![
            field("key", || "value"),
            field("nested", || {
                Schema::of(vec![field("nestedkey", || "nestedvalue"), field("nested2", || Schema::of(vec![field("nestedkey2", || "nestedvalue2")]))])
            }),
        ]),
    ]
}

#[test]
fn simple_toml_test() {
    let sep = LINE_SEPARATOR;
    let mut cases: Vec<(Schema<'static>, String)> = toml_and_yaml_cases()
        .into_iter()
        .zip([
            String::new(),
            "key = \"value\"".into(),
            "number = 123".into(),
            "number = 123.0".into(),
            "number = 123.123".into(),
            "boolean = true".into(),
            "nullValue = null".into(),
            "array = [ null, \"test\", \"123\" ]".into(),
            "array = [ 123, 456, 789 ]".into(),
            "array = [ \"test\", 456, true ]".into(),
            "emptyarray = []".into(),
            "emptyarray = []".into(),
            format!("key = \"value\"{sep}[nested]{sep}nestedkey = \"nestedvalue\""),
            format!("key = \"value\"{sep}[nested]{sep}nestedkey = \"nestedvalue\"{sep}[nested.nested2]{sep}nestedkey2 = \"nestedvalue2\""),
        ])
        .collect();
    cases.push((Schema::of(vec![field("parent", || Schema::of(vec![field("numbers", || Value::array([1, 2]))]))]), format!("[parent]{sep}numbers = [ 1, 2 ]")));
    cases.push((
        Schema::of(vec![field("items", || vec![Schema::of(vec![field("id", || 1)]), Schema::of(vec![field("id", || 2)])])]),
        format!("[[items]]{sep}id = 1{sep}[[items]]{sep}id = 2"),
    ));
    for (schema, expected) in cases {
        assert_eq!(TomlTransformer::new().generate(&schema, 1).unwrap(), expected);
    }
}

#[test]
fn toml_generate_from_fake_sequence() {
    for limit in [1, 4, 8] {
        let faker = Faker::new();
        let schema: Schema<Name> = Schema::of(vec![field_of("firstName", |n: &Name| n.first_name())]);
        let sequence = faker.collection().supplier(|| faker.name()).max_len(limit).build().unwrap();
        let toml = TomlTransformer::new().generate_from(&sequence, &schema).unwrap();
        assert_eq!(lines(&toml), limit as usize * schema.fields().len() - 1);
    }
}

#[test]
fn toml_generate_from_fake_sequence_with_collection() {
    for limit in [1, 4, 8] {
        let faker = Faker::new();
        let schema: Schema<Name> = Schema::of(vec![field_of("firstNames", move |n: &Name| (1..=limit).map(|_| n.first_name()).collect::<Result<Vec<_>>>())]);
        let sequence = faker.collection().supplier(|| faker.name()).max_len(1).build().unwrap();
        let toml = TomlTransformer::new().generate_from(&sequence, &schema).unwrap();
        common::assert_matches(&format!("firstNames = \\[ \"(.+\",?){{{limit}}} ]"), &toml);
    }
}

// XmlTest

fn xml_cases() -> Vec<Schema<'static>> {
    let attributes = || vec![field("attribute1", || "value1"), field("attribute2", || "value2")];
    vec![
        Schema::of(vec![field("root", Vec::<Field>::new)]),
        Schema::of(vec![field("root", || "value")]),
        Schema::of(vec![composite_field("root", attributes())]),
        Schema::of(vec![composite_field("root", [attributes(), vec![field(None::<&str>, || "value")]].concat())]),
        Schema::of(vec![composite_field("root", [attributes(), vec![field(None::<&str>, || vec![field("child", || "value")])]].concat())]),
        Schema::of(vec![field("root", || "<> value\"")]),
    ]
}

#[test]
fn xml_schema_test() {
    let expected = [
        "<root/>",
        "<root>value</root>",
        "<root attribute1=\"value1\" attribute2=\"value2\"/>",
        "<root attribute1=\"value1\" attribute2=\"value2\">value</root>",
        "<root attribute1=\"value1\" attribute2=\"value2\"><child>value</child></root>",
        "<root>&lt;&gt; value&quot;</root>",
    ];
    for (schema, expected) in xml_cases().into_iter().zip(expected) {
        assert_eq!(XmlTransformer::builder().build().generate(&schema, 1).unwrap(), expected);
    }
}

#[test]
fn xml_pretty_schema_test() {
    let pretty_child = format!("<root attribute1=\"value1\" attribute2=\"value2\">{0}    <child>value</child>{0}</root>", LINE_SEPARATOR);
    let expected = [
        "<root/>",
        "<root>value</root>",
        "<root attribute1=\"value1\" attribute2=\"value2\"/>",
        "<root attribute1=\"value1\" attribute2=\"value2\">value</root>",
        pretty_child.as_str(),
        "<root>&lt;&gt; value&quot;</root>",
    ];
    for (schema, expected) in xml_cases().into_iter().zip(expected) {
        assert_eq!(XmlTransformer::builder().pretty(true).build().generate(&schema, 1).unwrap(), expected);
    }
}

fn xml_lines(xml: &str) -> usize {
    1 + lines(xml)
}

#[test]
fn xml_generate_from_fake_sequence() {
    let faker = Faker::new();
    let schema: Schema<Name> =
        Schema::of(vec![field("root", || vec![field_of("firstName", |n: &Name| n.first_name()), field_of("lastName", |n: &Name| n.last_name())])]);
    let sequence = faker.collection().supplier(|| faker.name()).max_len(1).build().unwrap();
    let xml = XmlTransformer::builder().pretty(true).build().generate_from(&sequence, &schema).unwrap();
    assert_eq!(xml_lines(&xml), 4);
}

fn collect<T>(stream: &datafaker::sequence::FakeStream<'_, T>) -> Result<Vec<Option<T>>> {
    stream.get()?.collect()
}

#[test]
fn generate_xml_with_three_nested_levels() {
    let faker = Faker::new();
    let faker = &faker;
    let address = Arc::new(
        faker
            .stream()
            .supplier(move || {
                field("address", move || {
                    vec![
                        field("country", move || faker.address().country()),
                        field("city", move || faker.address().city()),
                        field("streetAddress", move || faker.address().street_address()),
                    ]
                })
            })
            .max_len(3)
            .build()
            .unwrap(),
    );
    let persons = Arc::new(
        faker
            .stream()
            .supplier(move || {
                let address = address.clone();
                field("person", move || {
                    let address = address.clone();
                    vec![
                        field("firstname", move || faker.name().first_name()),
                        field("lastname", move || faker.name().last_name()),
                        field("addresses", move || collect(&address)),
                    ]
                })
            })
            .max_len(3)
            .build()
            .unwrap(),
    );
    let schema: Schema = Schema::of(vec![field("persons", move || collect(&persons))]);
    let xml = XmlTransformer::builder().pretty(true).build().generate(&schema, 1).unwrap();
    assert!(!xml.is_empty());
    assert_eq!(xml_lines(&xml), 65);
}

#[test]
fn generate_xml_with_three_nested_levels_and_attributes() {
    let faker = Faker::new();
    let faker = &faker;
    let address = Arc::new(
        faker
            .stream()
            .supplier(move || {
                composite_field(
                    "address",
                    vec![
                        field("country", move || faker.address().country()),
                        field("city", move || faker.address().city()),
                        field("streetAddress", move || faker.address().street_address()),
                    ],
                )
            })
            .max_len(3)
            .build()
            .unwrap(),
    );
    let persons = Arc::new(
        faker
            .stream()
            .supplier(move || {
                let address = address.clone();
                composite_field(
                    "person",
                    vec![
                        field("firstname", move || faker.name().first_name()),
                        field("lastname", move || faker.name().last_name()),
                        field(None::<&str>, move || {
                            let address = address.clone();
                            vec![field("addresses", move || collect(&address))]
                        }),
                    ],
                )
            })
            .max_len(3)
            .build()
            .unwrap(),
    );
    let schema: Schema = Schema::of(vec![field("persons", move || collect(&persons))]);
    let xml = XmlTransformer::builder().pretty(true).build().generate(&schema, 1).unwrap();
    assert!(!xml.is_empty());
    assert_eq!(xml_lines(&xml), 23);
}

// YamlTest

#[test]
fn simple_yaml_test() {
    let sep = LINE_SEPARATOR;
    let expected = [
        String::new(),
        format!("key: value{sep}"),
        format!("number: 123{sep}"),
        format!("number: 123.0{sep}"),
        format!("number: 123.123{sep}"),
        format!("boolean: true{sep}"),
        format!("nullValue: null{sep}"),
        format!("array:{sep}  - null{sep}  - test{sep}  - 123{sep}"),
        format!("array:{sep}  - 123{sep}  - 456{sep}  - 789{sep}"),
        format!("array:{sep}  - test{sep}  - 456{sep}  - true{sep}"),
        format!("emptyarray:{sep}"),
        format!("emptyarray:{sep}"),
        format!("key: value{sep}nested:{sep}  nestedkey: nestedvalue{sep}"),
        format!("key: value{sep}nested:{sep}  nestedkey: nestedvalue{sep}  nested2:{sep}    nestedkey2: nestedvalue2{sep}"),
    ];
    for (schema, expected) in toml_and_yaml_cases().into_iter().zip(expected) {
        assert_eq!(YamlTransformer::new().generate(&schema, 1).unwrap(), expected);
    }
}

#[test]
fn yaml_generate_from_fake_sequence() {
    for limit in [1, 4, 8] {
        let faker = Faker::new();
        let schema: Schema<Name> = Schema::of(vec![field_of("firstName", |n: &Name| n.first_name())]);
        let sequence = faker.collection().supplier(|| faker.name()).max_len(limit).build().unwrap();
        let yaml = YamlTransformer::new().generate_from(&sequence, &schema).unwrap();
        assert_eq!(lines(&yaml), limit as usize * (schema.fields().len() + 1) - 1);
    }
}

#[test]
fn yaml_generate_from_fake_sequence_with_collection() {
    for limit in [1, 4, 8] {
        let faker = Faker::new();
        let schema: Schema<Name> = Schema::of(vec![field_of("firstNames", move |n: &Name| (1..=limit).map(|_| n.first_name()).collect::<Result<Vec<_>>>())]);
        let sequence = faker.collection().supplier(|| faker.name()).max_len(1).build().unwrap();
        let yaml = YamlTransformer::new().generate_from(&sequence, &schema).unwrap();
        assert_eq!(lines(&yaml), limit as usize + 1);
    }
}
