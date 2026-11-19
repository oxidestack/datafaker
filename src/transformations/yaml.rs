//! `YamlTransformer`.

use std::collections::HashSet;

use super::{Field, Input, LINE_SEPARATOR, Schema, Transformer, Value, check_finite, java_trim, null_name};
use crate::Result;

const INDENTATION: &str = "  ";

/// `YamlTransformer`: `key: value` lines, nested schemas indented, collections and arrays as `- item` lists; a
/// field whose (trimmed) name repeats an earlier one is skipped.
#[derive(Debug, Clone, Default)]
pub struct YamlTransformer;

impl YamlTransformer {
    /// A YAML transformer.
    pub fn new() -> YamlTransformer {
        YamlTransformer
    }

    fn apply_fields<'a, IN>(&self, sb: &mut String, input: Option<&IN>, fields: &[Field<'a, IN>], offset: &str) -> Result<()> {
        let mut keys = HashSet::new();
        for field in fields {
            let key = java_trim(field.name().ok_or_else(|| null_name("trim()"))?).to_string();
            if !keys.insert(key.clone()) {
                continue;
            }
            let value = field.transform(input)?;
            sb.push_str(offset);
            sb.push_str(&key);
            sb.push(':');
            if value.as_schema().is_some() {
                sb.push_str(LINE_SEPARATOR);
                self.value_to_string(&value, sb, &format!("{offset}{INDENTATION}"))?;
            } else {
                self.value_to_string(&value, sb, offset)?;
            }
            if !sb.ends_with(LINE_SEPARATOR) {
                sb.push_str(LINE_SEPARATOR);
            }
        }
        Ok(())
    }

    fn add_collection<'a, IN>(&self, sb: &mut String, items: &[Value<'a, IN>], offset: &str) -> Result<()> {
        for value in items {
            self.value_to_string(value, sb, &format!("{offset}-"))?;
            sb.push_str(LINE_SEPARATOR);
        }
        Ok(())
    }

    fn value_to_string<'a, IN>(&self, value: &Value<'a, IN>, sb: &mut String, offset: &str) -> Result<()> {
        if let Some(schema) = value.as_schema() {
            return self.apply_fields(sb, None, schema.fields(), offset);
        }
        match value {
            Value::List(items) | Value::Array(items) => {
                sb.push_str(LINE_SEPARATOR);
                self.add_collection(sb, items, &format!("{offset}{INDENTATION}"))
            }
            Value::Primitive(p) => {
                sb.push_str(LINE_SEPARATOR);
                Err(p.object_array_cast_error())
            }
            other => {
                if !sb.ends_with(':') {
                    sb.push_str(offset);
                }
                sb.push(' ');
                sb.push_str(java_trim(&other.to_java_string()));
                Ok(())
            }
        }
    }
}

impl<'a, IN> Transformer<'a, IN> for YamlTransformer {
    fn apply(&self, input: Option<&IN>, schema: &Schema<'a, IN>) -> Result<String> {
        let mut sb = String::new();
        if schema.fields().is_empty() {
            return Ok(sb);
        }
        self.apply_fields(&mut sb, input, schema.fields(), "")?;
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
