//! `TomlTransformer`.

use std::collections::HashSet;

use super::{Field, Input, LINE_SEPARATOR, Schema, Transformer, Value, check_finite};
use crate::Result;

/// `TomlTransformer`: `key = value` lines; a nested schema is a `[table]`, a collection of schemas an array of
/// tables (`[[table]]`), other collections and arrays inline arrays. A field whose name repeats an earlier one is
/// skipped.
#[derive(Debug, Clone, Default)]
pub struct TomlTransformer;

impl TomlTransformer {
    /// A TOML transformer.
    pub fn new() -> TomlTransformer {
        TomlTransformer
    }

    fn write_schema<'a, IN>(&self, sb: &mut String, input: Option<&IN>, fields: &[Field<'a, IN>], path_prefix: &str) -> Result<()> {
        let mut seen = HashSet::new();
        for f in fields {
            if !seen.insert(f.name().map(str::to_string)) {
                continue;
            }
            let value = f.transform(input)?;
            self.write_value(sb, f.name().unwrap_or("null"), path_prefix, &value)?;
        }
        Ok(())
    }

    fn write_value<'a, IN>(&self, sb: &mut String, key: &str, path_prefix: &str, value: &Value<'a, IN>) -> Result<()> {
        let full_path = if path_prefix.is_empty() { key.to_string() } else { format!("{path_prefix}.{key}") };
        if let Some(nested) = value.as_schema() {
            sb.push_str(&format!("[{full_path}]{LINE_SEPARATOR}"));
            return self.write_schema(sb, None, nested.fields(), &full_path);
        }
        match value {
            Value::List(items) => {
                if items.is_empty() {
                    sb.push_str(&format!("{key} = []{LINE_SEPARATOR}"));
                } else if items[0].as_schema().is_some() {
                    for item in items {
                        sb.push_str(&format!("[[{full_path}]]{LINE_SEPARATOR}"));
                        let schema = match item.as_schema() {
                            Some(s) => s,
                            None if item.is_null() => return Err(crate::Error::new(
                                "NullPointerException",
                                "Cannot invoke \"net.datafaker.transformations.Schema.getFields()\" because \"schema\" is null",
                            )),
                            None => return Err(item.class_cast_error("net.datafaker.transformations.Schema")),
                        };
                        self.write_schema(sb, None, schema.fields(), &full_path)?;
                    }
                } else {
                    inline_array(sb, key, items);
                }
            }
            Value::Array(items) => inline_array(sb, key, items),
            Value::Primitive(p) => inline_array(sb, key, &p.boxed::<IN>()),
            scalar => sb.push_str(&format!("{key} = {}{LINE_SEPARATOR}", value_to_string(scalar))),
        }
        Ok(())
    }
}

fn inline_array<IN>(sb: &mut String, key: &str, items: &[Value<'_, IN>]) {
    if items.is_empty() {
        sb.push_str(&format!("{key} = []{LINE_SEPARATOR}"));
        return;
    }
    let joined = items.iter().map(value_to_string).collect::<Vec<_>>().join(", ");
    sb.push_str(&format!("{key} = [ {joined} ]{LINE_SEPARATOR}"));
}

/// `value2String`: `null`, numbers and booleans as they print, anything else a quoted string with `\` and `"`
/// escaped.
fn value_to_string<IN>(value: &Value<'_, IN>) -> String {
    if value.is_null() || value.is_number() || matches!(value, Value::Bool(_)) {
        return value.to_java_string();
    }
    format!("\"{}\"", value.to_java_string().replace('\\', "\\\\").replace('"', "\\\""))
}

impl<'a, IN> Transformer<'a, IN> for TomlTransformer {
    fn apply(&self, input: Option<&IN>, schema: &Schema<'a, IN>) -> Result<String> {
        let mut sb = String::new();
        self.write_schema(&mut sb, input, schema.fields(), "")?;
        if sb.contains(LINE_SEPARATOR) {
            sb.pop();
        }
        Ok(sb)
    }

    fn generate_from(&self, input: &(impl Input<IN> + ?Sized), schema: &Schema<'a, IN>) -> Result<String> {
        check_finite(input)?;
        let mut data = Vec::new();
        for row in input.rows() {
            data.push(self.apply(row?.as_ref(), schema)?);
        }
        Ok(data.join(LINE_SEPARATOR))
    }

    fn generate(&self, schema: &Schema<'a, IN>, limit: i32) -> Result<String> {
        let mut sb = String::new();
        for i in 0..limit {
            sb.push_str(&self.apply(None, schema)?);
            if i < limit - 1 {
                sb.push_str(LINE_SEPARATOR);
            }
        }
        Ok(sb)
    }

    fn start_stream(&self, _schema: &Schema<'a, IN>) -> Result<String> {
        Ok(String::new())
    }

    fn end_stream(&self) -> Result<String> {
        Ok(String::new())
    }
}
