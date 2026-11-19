//! `JsonTransformer`.

use super::{Input, LINE_SEPARATOR, Schema, Transformer, Value, check_finite, csv::simple};
use crate::Result;

/// `JsonTransformer`: one object per row, `{"name": value, …}`; several rows make an array, one row per line,
/// with commas between them unless turned off.
#[derive(Debug, Clone)]
pub struct JsonTransformer {
    comma_between_objects: bool,
}

/// `JsonTransformer.JsonTransformerBuilder`: commas between objects by default.
#[derive(Debug, Clone)]
pub struct JsonTransformerBuilder {
    comma_between_objects: bool,
}

impl JsonTransformerBuilder {
    /// Whether objects are separated by commas.
    pub fn with_comma_between_objects(mut self, comma_between_objects: bool) -> Self {
        self.comma_between_objects = comma_between_objects;
        self
    }

    /// The transformer.
    pub fn build(self) -> JsonTransformer {
        JsonTransformer { comma_between_objects: self.comma_between_objects }
    }
}

impl JsonTransformer {
    /// `JsonTransformer.builder()`.
    pub fn builder() -> JsonTransformerBuilder {
        JsonTransformerBuilder { comma_between_objects: true }
    }

    fn apply_value<'a, IN>(&self, input: Option<&IN>, sb: &mut String, value: &Value<'a, IN>) -> Result<()> {
        match value {
            Value::List(items) | Value::Array(items) => self.generate_collection(input, sb, items),
            Value::Primitive(p) => Err(p.object_array_cast_error()),
            other => {
                value_to_string(other, sb);
                Ok(())
            }
        }
    }

    fn generate_collection<'a, IN>(&self, input: Option<&IN>, sb: &mut String, items: &[Value<'a, IN>]) -> Result<()> {
        sb.push('[');
        for (i, value) in items.iter().enumerate() {
            if i > 0 {
                sb.push_str(", ");
            }
            match value {
                Value::Field(f) if f.is_composite() => sb.push_str(&self.apply(input, &f.as_schema())?),
                other => self.apply_value(input, sb, other)?,
            }
        }
        sb.push(']');
        Ok(())
    }
}

/// `value2String`: numbers (but `Float` and `Byte`) and booleans as they print, `null`, anything else as a
/// string: escaped, and quoted unless it starts with `#{json`.
fn value_to_string<IN>(value: &Value<'_, IN>, sb: &mut String) {
    match value {
        Value::Null => sb.push_str("null"),
        Value::Int(_) | Value::Long(_) | Value::Short(_) | Value::BigInteger(_) | Value::Bool(_) | Value::Double(_) | Value::BigDecimal(_) => {
            sb.push_str(&value.to_java_string())
        }
        other => string_value(&other.to_java_string(), sb),
    }
}

/// `value2String` of a string.
pub(crate) fn string_value(value: &str, sb: &mut String) {
    let wrap = !value.starts_with("#{json");
    if wrap {
        sb.push('"');
    }
    for c in value.chars() {
        match c {
            '\\' => sb.push_str("\\\\"),
            '"' => sb.push_str("\\\""),
            '\u{8}' => sb.push_str("\\b"),
            '\u{c}' => sb.push_str("\\f"),
            '\n' => sb.push_str("\\n"),
            '\r' => sb.push_str("\\r"),
            '\t' => sb.push_str("\\t"),
            '/' => sb.push_str("\\/"),
            '\u{0}'..='\u{1f}' => sb.push_str(&format!("\\u{:04X}", c as u32)),
            other => sb.push(other),
        }
    }
    if wrap {
        sb.push('"');
    }
}

fn name_to_string(name: Option<&str>, sb: &mut String) {
    match name {
        Some(name) => string_value(name, sb),
        None => sb.push_str("null"),
    }
}

impl<'a, IN> Transformer<'a, IN> for JsonTransformer {
    fn apply(&self, input: Option<&IN>, schema: &Schema<'a, IN>) -> Result<String> {
        let fields = schema.fields();
        let mut sb = String::from("{");
        for (i, f) in fields.iter().enumerate() {
            name_to_string(f.name(), &mut sb);
            sb.push_str(": ");
            if f.is_composite() {
                sb.push_str(&self.apply_row(input, &f.as_schema(), i as i64)?);
            } else {
                let value = simple(f)?.transform(input)?;
                self.apply_value(input, &mut sb, &value)?;
            }
            if i < fields.len() - 1 {
                sb.push_str(", ");
            }
        }
        sb.push('}');
        Ok(sb)
    }

    fn generate_from(&self, input: &(impl Input<IN> + ?Sized), schema: &Schema<'a, IN>) -> Result<String> {
        check_finite(input)?;
        let mut data: Vec<String> = Vec::new();
        let mut rows = input.rows().peekable();
        while let Some(row) = rows.next() {
            let object = self.apply(row?.as_ref(), schema)?;
            let comma = self.comma_between_objects && rows.peek().is_some();
            data.push(if comma { object + "," } else { object });
        }
        let data = data.join(LINE_SEPARATOR);
        Ok(if data.encode_utf16().count() > 1 { format!("[{LINE_SEPARATOR}{data}{LINE_SEPARATOR}]") } else { data })
    }

    fn generate(&self, schema: &Schema<'a, IN>, limit: i32) -> Result<String> {
        let mut sb = String::new();
        for i in 0..limit {
            sb.push_str(&self.apply_row(None, schema, i as i64)?);
            if self.comma_between_objects && i < limit - 1 {
                sb.push(',');
                sb.push_str(LINE_SEPARATOR);
            }
        }
        Ok(if limit > 1 { format!("[{LINE_SEPARATOR}{sb}{LINE_SEPARATOR}]") } else { sb })
    }

    fn start_stream(&self, _schema: &Schema<'a, IN>) -> Result<String> {
        Ok("[".into())
    }

    fn end_stream(&self) -> Result<String> {
        Ok("]".into())
    }

    fn element_separator(&self) -> &str {
        ","
    }
}
