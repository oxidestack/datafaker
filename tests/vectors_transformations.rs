//! Differential vectors of the transformers and fake sequences (T5): `tests/vectors/transformations.tsv` holds named
//! cases recorded from Datafaker 2.7.0 by `scripts/Transformations.java` (`scripts/record.sh
//! tests/vectors/transformations.tsv Transformations`); each is rebuilt here the same way on a fresh
//! `Faker::with_seed(seed)` and must give the same text, exceptions included (`ERR <simple class name>: <message>`).

use std::sync::Arc;

use datafaker::sequence::Supplier;
use datafaker::transformations::sql::Case;
use datafaker::transformations::{
    Casing, CsvTransformer, Field, FieldName, JsonTransformer, PrimitiveArray, Schema, SqlDialect, SqlTransformer, TomlTransformer, Transformer,
    Value, XmlTransformer, YamlTransformer, composite_field, field, field_of,
};
use datafaker::{Faker, Result};

fn f<'a, IN, R: datafaker::transformations::IntoValue<'a, IN>>(name: impl FieldName, s: impl Fn() -> R + Send + Sync + 'a) -> Field<'a, IN> {
    field(name, s)
}

fn none() -> Option<&'static str> {
    None
}

/// `values(faker)`: scalars, a collection and a map of every runtime type the transformers tell apart.
fn values<IN>(faker: &Faker) -> Schema<'_, IN> {
    Schema::of(vec![
        f("null", || ()),
        f("int", move || faker.number().random_digit()),
        f("long", || 5i64),
        f("short", || 3i16),
        f("byte", || 4i8),
        f("float", || 2.5f32),
        f("double", || 1.0E-5),
        f("bigInteger", || Value::BigInteger("123456789012345678901234567890".into())),
        f("bigDecimal", || Value::BigDecimal("1.50".into())),
        f("bool", move || faker.bool().bool()),
        f("char", || 'q'),
        f("string", move || Ok::<_, datafaker::Error>(format!(" a \"b\" 'c' \\ / \t\n\u{1}\u{1f} {} ", faker.name().first_name()?))),
        f("json", || "#{json 'a','b'}"),
        f("list", || Value::list([Value::Str("x".into()), Value::Int(1), Value::Bool(false)])),
        f("map", || Value::map([(Value::Str("k".into()), Value::Str("v".into())), (Value::Str("n".into()), Value::Int(2))])),
    ])
}

fn name_schema<IN>(faker: &Faker) -> Schema<'_, IN> {
    Schema::of(vec![f("first", move || faker.name().first_name()), f("digit", move || faker.number().random_digit())])
}

fn join(stream: Result<Box<dyn Iterator<Item = Result<String>> + '_>>) -> Result<String> {
    Ok(stream?.collect::<Result<Vec<_>>>()?.join("\n"))
}

fn first_name(faker: &Faker) -> String {
    faker.name().first_name().unwrap()
}

/// `List.toString()` of a list with `null`s.
fn list<T: ToString>(items: Vec<Option<T>>) -> String {
    format!("[{}]", items.iter().map(|i| i.as_ref().map_or("null".to_string(), T::to_string)).collect::<Vec<_>>().join(", "))
}

fn identity<'a>(name: &str) -> Field<'a, String> {
    field_of(name.to_string(), |s: &String| s.clone())
}

fn xml() -> XmlTransformer {
    XmlTransformer::builder().build()
}

fn run(name: &str, fk: &Faker) -> Result<String> {
    let digit = move || fk.number().random_digit();
    Ok(match name {
        // CSV
        "csv.values" => CsvTransformer::builder().build().generate(&values::<()>(fk), 2)?,
        "csv.options" => CsvTransformer::builder().separator(" | ").quote('\'').header(false).build().generate(
            &Schema::<()>::of(vec![f("it's", move || Ok::<_, datafaker::Error>(format!("it's {}", fk.name().last_name()?))), f("n", digit)]),
            3,
        )?,
        "csv.stream" => join(CsvTransformer::builder().build().generate_stream(&name_schema::<()>(fk), 3))?,
        "csv.streamNoHeader" => join(CsvTransformer::builder().header(false).build().generate_stream(&name_schema::<()>(fk), 2))?,
        "csv.nullName" => CsvTransformer::builder().build().generate(&Schema::<()>::of(vec![f(none(), || "v")]), 1)?,
        "csv.composite" => CsvTransformer::builder().build().generate(&Schema::<()>::of(vec![composite_field("c", vec![f("a", || "b")])]), 1)?,
        "csv.list" => CsvTransformer::builder()
            .build()
            .generate_from(&vec!["a".to_string(), "b'c".to_string()], &Schema::of(vec![identity("v"), f("n", move || fk.name().first_name())]))?,
        "csv.empty" => CsvTransformer::builder().build().generate_from(&Vec::<String>::new(), &name_schema(fk))?,
        "csv.limit0" => CsvTransformer::builder().build().generate(&name_schema::<()>(fk), 0)?,
        // JSON
        "json.values" => JsonTransformer::builder().build().generate(&values::<()>(fk), 2)?,
        "json.objects" => JsonTransformer::builder().build().generate(
            &Schema::<()>::of(vec![
                f("objects", || {
                    Value::array([
                        Value::Str("s".into()),
                        Value::Int(1),
                        Value::Null,
                        Value::list([2])?,
                        Value::array(["n"])?,
                        Value::Field(composite_field(none(), vec![f("in", || "x")])),
                    ])
                }),
                f("field", || vec![f("simple", || "x")]),
                composite_field(none(), vec![f("a", move || fk.name().first_name()), composite_field("b", vec![f(none(), || ())])]),
            ]),
            1,
        )?,
        "json.ints" => JsonTransformer::builder().build().generate(&Schema::<()>::of(vec![f("ints", || PrimitiveArray::Int(vec![1, 2]))]), 1)?,
        "json.doubles" => JsonTransformer::builder().build().generate(&Schema::<()>::of(vec![f("d", || PrimitiveArray::Double(vec![1.0]))]), 1)?,
        "json.stream" => join(JsonTransformer::builder().build().generate_stream(&name_schema::<()>(fk), 3))?,
        "json.stream1" => join(JsonTransformer::builder().with_comma_between_objects(false).build().generate_stream(&name_schema::<()>(fk), 1))?,
        "json.streamNegative" => join(JsonTransformer::builder().build().generate_stream(&name_schema::<()>(fk), -1))?,
        "json.limit1" => JsonTransformer::builder().build().generate(&name_schema::<()>(fk), 1)?,
        "json.limit0" => JsonTransformer::builder().build().generate(&name_schema::<()>(fk), 0)?,
        "json.emptyInput" => JsonTransformer::builder().build().generate_from(&Vec::<String>::new(), &name_schema(fk))?,
        "json.oneInput" => JsonTransformer::builder().build().generate_from(&vec!["x".to_string()], &Schema::of(vec![identity("v")]))?,
        "json.nullRows" => JsonTransformer::builder()
            .build()
            .generate_from(&fk.collection().supplier(move || first_name(fk)).null_rate(0.5)?.len(6).build()?, &Schema::<String>::of(vec![f("n", digit)]))?,
        "json.nullRowsFunction" => JsonTransformer::builder()
            .build()
            .generate_from(&fk.collection().supplier(move || first_name(fk)).null_rate(0.5)?.len(6).build()?, &Schema::of(vec![identity("n")]))?,
        "json.streamRows" => JsonTransformer::builder().with_comma_between_objects(false).build().generate_from(
            &fk.stream().supplier(move || fk.name()).len_between(1, 4).build()?,
            &Schema::of(vec![field_of("first", |n: &datafaker::providers::Name| n.first_name()), f("d", digit)]),
        )?,
        // YAML
        "yaml.values" => YamlTransformer::new().generate(&values::<()>(fk), 2)?,
        "yaml.nested" => YamlTransformer::new().generate(
            &Schema::<()>::of(vec![
                f(" key ", || "v"),
                f("key", || "dup"),
                f("nested", move || {
                    Schema::of(vec![
                        f("a", move || fk.name().first_name()),
                        f("b", || Value::list([Value::Int(1), Value::Schema(Schema::of(vec![f("x", || "y")]))])),
                    ])
                }),
                f("composite", || composite_field("c", vec![f("in", || "z")])),
                composite_field("compositeField", vec![f("in", || "z")]),
                f("objects", || Value::array([Value::Str("s".into()), Value::Null, Value::list(["t"])?])),
            ]),
            1,
        )?,
        "yaml.ints" => YamlTransformer::new().generate(&Schema::<()>::of(vec![f("ints", || PrimitiveArray::Int(vec![1]))]), 1)?,
        "yaml.nullName" => YamlTransformer::new().generate(&Schema::<()>::of(vec![f(none(), || "v")]), 1)?,
        "yaml.rows" => YamlTransformer::new()
            .generate_from(&fk.collection().supplier(move || first_name(fk)).len(3).build()?, &Schema::of(vec![identity("v"), f("d", digit)]))?,
        // XML
        "xml.mixed" => xml().generate(
            &Schema::<()>::of(vec![
                f("text", move || Ok::<_, datafaker::Error>(format!("<a & 'b'> \"{}\"", fk.name().first_name()?))),
                f(" spaced ", || ()),
                f("empty", Vec::<String>::new),
                f("number", || 5),
                composite_field(
                    "el",
                    vec![f("x", || "1 < 2"), f(none(), || vec![f("child", || "c"), composite_field("sub", vec![f("k", || "v")])])],
                ),
                composite_field("onlyText", vec![f(none(), || "t"), f(none(), || "ignored")]),
            ]),
            2,
        )?,
        "xml.pretty" => XmlTransformer::builder().pretty(true).build().generate(
            &Schema::<()>::of(vec![f("root", move || {
                vec![f("a", move || fk.name().first_name()), f("b", || vec![f("c", || Value::Str("d".into())), f("e", || Value::Null)])]
            })]),
            2,
        )?,
        "xml.attrInt" => xml().generate(&Schema::<()>::of(vec![composite_field("el", vec![f("x", || 1)])]), 1)?,
        "xml.attrNull" => xml().generate(&Schema::<()>::of(vec![composite_field("el", vec![f("x", || ())])]), 1)?,
        "xml.childString" => xml().generate(&Schema::<()>::of(vec![f("el", || vec!["x"])]), 1)?,
        "xml.nullName" => xml().generate(&Schema::<()>::of(vec![f(none(), || "v")]), 1)?,
        "xml.rows" => xml().generate_from(&fk.collection().supplier(move || first_name(fk)).len(2).build()?, &Schema::of(vec![identity("v")]))?,
        // TOML
        "toml.values" => TomlTransformer::new().generate(&values::<()>(fk), 2)?,
        "toml.arrays" => TomlTransformer::new().generate(
            &Schema::<()>::of(vec![
                f("ints", || PrimitiveArray::Int(vec![1, 2])),
                f("chars", || PrimitiveArray::Char(vec!['a', '"'])),
                f("bools", || PrimitiveArray::Boolean(vec![true])),
                f("doubles", || PrimitiveArray::Double(vec![0.1, 1e20])),
                f("floats", || PrimitiveArray::Float(vec![])),
                f("objects", || Value::array([Value::Str("s".into()), Value::Null, Value::list([2])?, Value::Char('c')])),
                f(none(), || "nullName"),
                f(none(), || "dup"),
                f("table", move || composite_field("t", vec![f("in", move || fk.name().first_name())])),
                composite_field("compositeField", vec![f("in", || "z")]),
                f("tables", || {
                    Value::list([
                        Value::Schema(Schema::of(vec![f("id", || 1), f("sub", || Schema::of(vec![f("x", || "y")]))])),
                        Value::Field(composite_field("c", vec![f("id", || 2)])),
                    ])
                }),
                f("after", || "a\\b"),
            ]),
            1,
        )?,
        "toml.badTables" => TomlTransformer::new()
            .generate(&Schema::<()>::of(vec![f("tables", || Value::list([Value::Schema(Schema::of(vec![f("id", || 1)])), Value::Int(2)]))]), 1)?,
        "toml.rows" => TomlTransformer::new()
            .generate_from(&fk.stream().supplier(move || first_name(fk)).len(2).build()?, &Schema::of(vec![identity("v"), f("d", digit)]))?,
        // SQL
        "sql.values" => SqlTransformer::builder().dialect(SqlDialect::Calcite).build()?.generate(&values::<()>(fk), 2)?,
        "sql.valuesNoDialect" => SqlTransformer::builder().build()?.generate(
            &Schema::<()>::of(vec![
                f("a", || "it's"),
                f("b", || 'q'),
                f("c", || Value::map([(Value::Str("k".into()), Value::Null)])),
                f("d", Vec::<String>::new),
            ]),
            1,
        )?,
        "sql.arrays" => SqlTransformer::builder().dialect(SqlDialect::Sparksql).build()?.generate(
            &Schema::<()>::of(vec![
                f("chars", || PrimitiveArray::Char(vec!['a'])),
                f("floats", || PrimitiveArray::Float(vec![0.1])),
                f("objects", || {
                    Value::array([
                        Value::Str("it's".into()),
                        Value::Null,
                        Value::Primitive(PrimitiveArray::Long(vec![1])),
                        Value::list([1])?,
                        Value::map([(1, true)])?,
                    ])
                }),
                composite_field("st", vec![f("x", || 1), f("y", || ()), composite_field("in", vec![f("z", || "w")])]),
            ]),
            1,
        )?,
        "sql.arrayNoDialect" => SqlTransformer::builder().build()?.generate(&Schema::<()>::of(vec![f("ints", || PrimitiveArray::Int(vec![1]))]), 1)?,
        "sql.compositeNoDialect" => SqlTransformer::builder().build()?.generate(&Schema::<()>::of(vec![composite_field("c", vec![f("a", || 1)])]), 1)?,
        "sql.nullName" => SqlTransformer::builder().build()?.generate(&Schema::<()>::of(vec![f(none(), || 1)]), 1)?,
        "sql.nullNameForced" => SqlTransformer::builder().force_use_sql_quote_identifier().build()?.generate(&Schema::<()>::of(vec![f(none(), || 1)]), 1)?,
        "sql.sparkNullName" => SqlTransformer::builder()
            .dialect(SqlDialect::Sparksql)
            .build()?
            .generate(&Schema::<()>::of(vec![composite_field("c", vec![f(none(), || 1)])]), 1)?,
        "sql.emptyIdentifier" => SqlTransformer::builder().sql_quote_identifier("").build()?.generate(&name_schema::<()>(fk), 1)?,
        "sql.identifiers" => {
            let schema: Schema = Schema::of(
                ["lower", "UPPER", "Mixed.Dot", "quo\"te", "back`tick", "br[ack]et", "ÄÖ", "äö"]
                    .iter()
                    .enumerate()
                    .map(|(i, n)| f(*n, move || i as i32 + 1))
                    .collect(),
            );
            let mut sb = String::new();
            for d in SqlDialect::VALUES {
                let sql = SqlTransformer::builder().dialect(*d).table_name("my_table").schema_name("Sch.ema").build()?.generate(&schema, 1)?;
                sb.push_str(&format!("{}: {sql}\n", java_enum(*d)));
            }
            sb.push_str(&SqlTransformer::builder().casing(Casing::ToLower).sql_quote_identifier("[]").table_name(none()).build()?.generate(&schema, 1)?);
            sb
        }
        "sql.dialects" => {
            let cases = [Case::Capital, Case::Lowercase, Case::Uppercase];
            let mut sb = String::new();
            for (i, d) in SqlDialect::VALUES.iter().enumerate() {
                let k = cases[i % cases.len()];
                sb.push_str(&format!("{} {}:\n", java_enum(*d), format!("{k:?}").to_uppercase()));
                sb.push_str(&SqlTransformer::builder().dialect(*d).batch_of(2).keyword_case(k).build()?.generate(&name_schema::<()>(fk), 3)?);
                sb.push('\n');
                let schema: Schema = Schema::of(vec![
                    f("n", || ()),
                    composite_field("c", vec![f("a", move || fk.name().first_name()), f("b", || PrimitiveArray::Int(vec![1]))]),
                ]);
                sb.push_str(&SqlTransformer::builder().dialect(*d).keyword_case(k).build()?.generate(&schema, 1)?);
                sb.push('\n');
            }
            sb
        }
        "sql.batchStream" => join(SqlTransformer::builder().batch_of(2).dialect(SqlDialect::Oracle).build()?.generate_stream(&name_schema::<()>(fk), 4))?,
        "sql.batchAll" => SqlTransformer::builder().batch().build()?.generate(&name_schema::<()>(fk), 3)?,
        "sql.batchAllStream" => {
            let transformer = SqlTransformer::builder().batch().build()?;
            let schema = name_schema::<()>(fk);
            transformer.generate_stream(&schema, 3)?.take(2).collect::<Result<Vec<_>>>()?.join("\n")
        }
        "sql.stream" => join(SqlTransformer::builder().keyword_case(Case::Lowercase).build()?.generate_stream(&name_schema::<()>(fk), 2))?,
        "sql.streamNegative" => join(SqlTransformer::builder().build()?.generate_stream(&name_schema::<()>(fk), -1))?,
        "sql.startStream" => SqlTransformer::builder().build()?.start_stream(&name_schema::<()>(fk))?,
        "sql.rows" => SqlTransformer::builder()
            .batch_of(2)
            .build()?
            .generate_from(&fk.stream().supplier(move || first_name(fk)).len(3).build()?, &Schema::of(vec![identity("v"), f("d", digit)]))?,
        "sql.quote" => SqlTransformer::builder()
            .quote('"')
            .build()?
            .generate(&Schema::<()>::of(vec![f("q", || "say \"hi\" it's"), f("n", move || fk.name().last_name())]), 2)?,
        // Sequences
        "seq.collection" => {
            let suppliers: Vec<Supplier<'_, String>> = vec![Arc::new(move || first_name(fk)), Arc::new(move || digit().to_string())];
            list(fk.collection_of(suppliers).null_rate(0.3)?.len_between(2, 8).build()?.get()?)
        }
        "seq.stream" => list(fk.stream().supplier(move || first_name(fk)).null_rate(0.5)?.len(4).build()?.get()?.collect::<Result<Vec<_>>>()?),
        "seq.streamString" => fk.stream().supplier(|| 1).null_rate(0.25)?.len_between(2, 5).build()?.to_string(),
        "seq.default" => list(fk.collection().supplier(digit).max_len(-3).build()?.get()?),
        "seq.minOnly" => list(fk.collection().supplier(digit).min_len(12).build()?.get()?),
        "seq.singletons" => {
            let seq = fk.collection().supplier(|| "a").supplier(|| "b").supplier(|| "c").null_rate(0.4)?.build()?;
            list((0..6).map(|_| seq.singleton()).collect::<Result<Vec<_>>>()?)
        }
        "seq.noSuppliers" => list(fk.collection::<i32>().len(1).build()?.get()?),
        "seq.nullRateNaN" => list(fk.collection().supplier(|| 1).null_rate(f64::NAN)?.len(3).build()?.get()?),
        "seq.streamMinOnly" => fk.stream().supplier(|| 1).min_len(3).build()?.to_string(),
        other => panic!("unknown case {other}"),
    })
}

/// `SqlDialect.toString()`: the constant's name.
fn java_enum(d: SqlDialect) -> String {
    format!("{d:?}").to_uppercase()
}

fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

#[test]
fn transformations_tsv() {
    let text = include_str!("vectors/transformations.tsv");
    let mut rows = 0;
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let cols: Vec<&str> = line.splitn(3, '\t').collect();
        let seed: i64 = cols[0].parse().unwrap();
        let expected = unescape(cols.get(2).copied().unwrap_or(""));
        let faker = Faker::with_seed(seed);
        let actual = match run(cols[1], &faker) {
            Ok(s) => s,
            Err(e) => format!("ERR {}: {}", e.simple_class(), e.message()),
        };
        assert_eq!(actual, expected, "seed {seed}, case {}", cols[1]);
        rows += 1;
    }
    assert_eq!(rows, 134);
}
