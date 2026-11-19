//! `net.datafaker.transformations`: a [`Schema`] of named [`Field`]s, and the transformers that render rows of it as
//! CSV, JSON, SQL, XML, YAML and TOML text or build objects from it.
//!
//! A Java field is a function of the row's input object or a supplier; here a closure returning anything that
//! becomes a [`Value`] (Java's `Object`, by runtime type), or a `Result` of it:
//!
//! ```
//! use datafaker::Faker;
//! use datafaker::transformations::{field, CsvTransformer, Schema, Transformer};
//!
//! let faker = Faker::with_seed(10);
//! let schema: Schema = Schema::of(vec![field("Number", || faker.number().random_digit()), field("Bool", || faker.bool().bool())]);
//! let csv = CsvTransformer::builder().separator(",").build().generate(&schema, 2).unwrap();
//! assert_eq!(csv.lines().count(), 3);
//! ```

mod csv;
mod java_object;
mod json;
pub mod sql;
mod toml;
mod value;
mod xml;
mod yaml;

use std::io::Write;
use std::sync::Arc;

pub use csv::{CsvTransformer, CsvTransformerBuilder, DEFAULT_QUOTE, DEFAULT_SEPARATOR};
pub use java_object::{JavaObject, JavaObjectTransformer};
pub use json::{JsonTransformer, JsonTransformerBuilder};
pub use sql::{Casing, SqlDialect, SqlTransformer, SqlTransformerBuilder};
pub use toml::TomlTransformer;
pub use value::{IntoValue, Object, PrimitiveArray, Value};
pub use xml::{XmlTransformer, XmlTransformerBuilder};
pub use yaml::YamlTransformer;

use crate::{Error, Result};

/// `Transformer.LINE_SEPARATOR` (`System.lineSeparator()`) on the platforms MockServer runs on.
pub const LINE_SEPARATOR: &str = "\n";

type FunctionFn<'a, IN> = dyn Fn(&IN) -> Result<Value<'a, IN>> + Send + Sync + 'a;
type SupplierFn<'a, IN> = dyn Fn() -> Result<Value<'a, IN>> + Send + Sync + 'a;

enum Kind<'a, IN> {
    Function(Arc<FunctionFn<'a, IN>>),
    Supplier(Arc<SupplierFn<'a, IN>>),
    Composite(Vec<Field<'a, IN>>),
}

/// `Field`: a `SimpleField` (a name and a function of the input or a supplier) or a `CompositeField` (a name and
/// fields of its own). A name may be `null` (`None`): XML reads such a field of a composite as its content.
pub struct Field<'a, IN = ()> {
    name: Option<String>,
    kind: Kind<'a, IN>,
}

impl<IN> Clone for Field<'_, IN> {
    fn clone(&self) -> Self {
        let kind = match &self.kind {
            Kind::Function(f) => Kind::Function(f.clone()),
            Kind::Supplier(s) => Kind::Supplier(s.clone()),
            Kind::Composite(fields) => Kind::Composite(fields.clone()),
        };
        Field { name: self.name.clone(), kind }
    }
}

impl<IN> std::fmt::Debug for Field<'_, IN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            Kind::Composite(fields) => f.debug_struct("CompositeField").field("name", &self.name).field("fields", fields).finish(),
            _ => f.debug_struct("SimpleField").field("name", &self.name).finish(),
        }
    }
}

/// A field's name: a string, or `None` for Java's `null`.
pub trait FieldName {
    /// The name, or `None` for `null`.
    fn into_name(self) -> Option<String>;
}

impl FieldName for &str {
    fn into_name(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl FieldName for String {
    fn into_name(self) -> Option<String> {
        Some(self)
    }
}

impl FieldName for Option<&str> {
    fn into_name(self) -> Option<String> {
        self.map(str::to_string)
    }
}

impl FieldName for Option<String> {
    fn into_name(self) -> Option<String> {
        self
    }
}

/// `Field.field(name, supplier)`: a field whose value does not depend on the input.
pub fn field<'a, IN, R: IntoValue<'a, IN>>(name: impl FieldName, supplier: impl Fn() -> R + Send + Sync + 'a) -> Field<'a, IN> {
    Field { name: name.into_name(), kind: Kind::Supplier(Arc::new(move || supplier().into_value())) }
}

/// `Field.field(name, function)`: a field computed from the row's input object.
pub fn field_of<'a, IN, R: IntoValue<'a, IN>>(name: impl FieldName, function: impl Fn(&IN) -> R + Send + Sync + 'a) -> Field<'a, IN> {
    Field { name: name.into_name(), kind: Kind::Function(Arc::new(move |input| function(input).into_value())) }
}

/// `Field.compositeField(name, fields)`: a nested object (JSON), a row value (SQL), an element with attributes
/// (XML).
pub fn composite_field<'a, IN>(name: impl FieldName, fields: Vec<Field<'a, IN>>) -> Field<'a, IN> {
    Field { name: name.into_name(), kind: Kind::Composite(fields) }
}

impl<'a, IN> Field<'a, IN> {
    /// `getName()`.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// `instanceof CompositeField`.
    pub fn is_composite(&self) -> bool {
        matches!(self.kind, Kind::Composite(_))
    }

    /// A composite field's fields (`CompositeField.getFields()`); none for a simple one.
    pub fn fields(&self) -> &[Field<'a, IN>] {
        match &self.kind {
            Kind::Composite(fields) => fields,
            _ => &[],
        }
    }

    /// `transform(input)`: the supplier's value, or the function's on the input (an input is required then). A
    /// composite field transforms to `null`.
    pub fn transform(&self, input: Option<&IN>) -> Result<Value<'a, IN>> {
        match &self.kind {
            Kind::Supplier(supplier) => supplier(),
            Kind::Function(function) => match input {
                Some(input) => function(input),
                None => Err(Error::illegal_argument("Input could be null only if suppliers are defined")),
            },
            Kind::Composite(_) => Ok(Value::Null),
        }
    }

    /// `hashCode()`: a simple field's is its name's; a composite field's adds its fields'.
    pub(crate) fn java_hash_code(&self) -> i32 {
        let name = self.name.as_deref().map_or(0, crate::java::string_hash);
        match &self.kind {
            Kind::Composite(fields) => arrays_hash_code(fields).wrapping_mul(31).wrapping_add(name),
            _ => name,
        }
    }

    pub(crate) fn as_schema(&self) -> Schema<'a, IN> {
        Schema { fields: self.fields().to_vec() }
    }
}

fn arrays_hash_code<IN>(fields: &[Field<'_, IN>]) -> i32 {
    fields.iter().fold(1i32, |h, f| h.wrapping_mul(31).wrapping_add(f.java_hash_code()))
}

/// `Schema`: the fields of a row, in order.
pub struct Schema<'a, IN = ()> {
    fields: Vec<Field<'a, IN>>,
}

impl<IN> Clone for Schema<'_, IN> {
    fn clone(&self) -> Self {
        Schema { fields: self.fields.clone() }
    }
}

impl<IN> std::fmt::Debug for Schema<'_, IN> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Schema").field("fields", &self.fields).finish()
    }
}

impl<'a, IN> Schema<'a, IN> {
    /// `Schema.of(fields...)`.
    pub fn of(fields: Vec<Field<'a, IN>>) -> Schema<'a, IN> {
        Schema { fields }
    }

    /// `getFields()`.
    pub fn fields(&self) -> &[Field<'a, IN>] {
        &self.fields
    }

    pub(crate) fn java_hash_code(&self) -> i32 {
        arrays_hash_code(&self.fields)
    }
}

/// What a `FakeSequence` tells the transformers about itself.
pub trait SequenceInfo {
    /// `isInfinite()`.
    fn is_infinite(&self) -> bool;

    /// `toString()`.
    fn describe(&self) -> String;
}

/// The `Iterable<IN>` the transformers' `generate(input, schema)` takes: the rows, each an input object or `null`.
pub trait Input<IN> {
    /// The rows (a fake sequence generates them now; generating one may fail).
    fn rows(&self) -> Box<dyn Iterator<Item = Result<Option<IN>>> + '_>;

    /// The `FakeSequence` this is, if it is one.
    fn as_sequence(&self) -> Option<&dyn SequenceInfo> {
        None
    }
}

impl<IN: Clone> Input<IN> for Vec<IN> {
    fn rows(&self) -> Box<dyn Iterator<Item = Result<Option<IN>>> + '_> {
        Box::new(self.iter().map(|i| Ok(Some(i.clone()))))
    }
}

impl<IN: Clone> Input<IN> for Vec<Option<IN>> {
    fn rows(&self) -> Box<dyn Iterator<Item = Result<Option<IN>>> + '_> {
        Box::new(self.iter().map(|i| Ok(i.clone())))
    }
}

/// The `IllegalArgumentException` the text transformers throw for an infinite sequence.
pub(crate) fn check_finite<IN>(input: &(impl Input<IN> + ?Sized)) -> Result<()> {
    match input.as_sequence() {
        Some(s) if s.is_infinite() => Err(Error::illegal_argument(format!("The sequence should be finite of size: {}", s.describe()))),
        _ => Ok(()),
    }
}

/// `Transformer<IN, CharSequence>`: renders rows of a schema as text.
pub trait Transformer<'a, IN> {
    /// `apply(input, schema)`: one row.
    fn apply(&self, input: Option<&IN>, schema: &Schema<'a, IN>) -> Result<String>;

    /// `apply(input, schema, rowId)`: one row that knows its position (SQL batches); others ignore it.
    fn apply_row(&self, input: Option<&IN>, schema: &Schema<'a, IN>, row_id: i64) -> Result<String> {
        let _ = row_id;
        self.apply(input, schema)
    }

    /// `generate(input, schema)`: a row per element of `input`.
    fn generate_from(&self, input: &(impl Input<IN> + ?Sized), schema: &Schema<'a, IN>) -> Result<String>
    where
        Self: Sized;

    /// `generate(schema, limit)`: `limit` rows without input.
    fn generate(&self, schema: &Schema<'a, IN>, limit: i32) -> Result<String>;

    /// `getStartStream(schema)`.
    fn start_stream(&self, schema: &Schema<'a, IN>) -> Result<String>;

    /// `getEndStream()`.
    fn end_stream(&self) -> Result<String>;

    /// `getLineSeparator()`.
    fn line_separator(&self) -> &str {
        LINE_SEPARATOR
    }

    /// `getElementSeparator()`.
    fn element_separator(&self) -> &str {
        ""
    }

    /// `generateStream(schema, limit)`: `limit` rows without input, generated as they are read; the first carries
    /// the start of the stream, the last its end. A negative `limit` fails as `Stream.limit` does.
    fn generate_stream<'s>(&'s self, schema: &'s Schema<'a, IN>, limit: i64) -> Result<Box<dyn Iterator<Item = Result<String>> + 's>> {
        check_stream_limit(limit)?;
        let mut current = 0i64;
        Ok(Box::new(std::iter::from_fn(move || {
            if current >= limit {
                return None;
            }
            let item = (|| {
                let mut res = String::new();
                if current == 0 {
                    res.push_str(&self.start_stream(schema)?);
                    res.push_str(self.line_separator());
                }
                res.push_str(&self.apply_row(None, schema, current)?);
                if current == limit - 1 {
                    res.push_str(self.line_separator());
                    res.push_str(&self.end_stream()?);
                } else if !self.element_separator().is_empty() {
                    res.push_str(self.element_separator());
                }
                Ok(res)
            })();
            current += 1;
            Some(item)
        })))
    }

    /// `writeToOutputStream(outputStream, schema, limit)`: each item of [`Self::generate_stream`] and a line
    /// separator, in UTF-8.
    fn write_to_output_stream(&self, out: &mut dyn Write, schema: &Schema<'a, IN>, limit: i64) -> Result<()> {
        for item in self.generate_stream(schema, limit)? {
            let bytes = format!("{}{}", item?, LINE_SEPARATOR);
            out.write_all(bytes.as_bytes()).map_err(|e| Error::runtime(format!("java.io.IOException: {e}")))?;
        }
        Ok(())
    }
}

/// `Stream.limit(maxSize)` refuses a negative size.
pub(crate) fn check_stream_limit(limit: i64) -> Result<()> {
    if limit < 0 {
        return Err(Error::illegal_argument(limit.to_string()));
    }
    Ok(())
}

/// `String.trim()`: strips the characters up to U+0020 at both ends.
pub(crate) fn java_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c <= ' ')
}

/// The `NullPointerException` of calling a method on a field's `null` name.
pub(crate) fn null_name(method: &str) -> Error {
    Error::new(
        "NullPointerException",
        format!("Cannot invoke \"String.{method}\" because the return value of \"net.datafaker.transformations.Field.getName()\" is null"),
    )
}
