//! `CsvTransformer`.

use super::{Field, Input, LINE_SEPARATOR, Schema, Transformer, Value, check_finite, value::class_cast_error};
use crate::Result;

/// `CsvTransformer.DEFAULT_SEPARATOR`.
pub const DEFAULT_SEPARATOR: &str = ";";

/// `CsvTransformer.DEFAULT_QUOTE`.
pub const DEFAULT_QUOTE: char = '"';

/// `CsvTransformer`: one line per row, values joined by the separator; strings quoted (a quote inside doubled),
/// other values as their `toString()`; a header line of the quoted field names.
#[derive(Debug, Clone)]
pub struct CsvTransformer {
    separator: String,
    quote: char,
    with_header: bool,
}

/// `CsvTransformer.CsvTransformerBuilder`: `;`, `"` and a header by default.
#[derive(Debug, Clone)]
pub struct CsvTransformerBuilder {
    separator: String,
    quote: char,
    with_header: bool,
}

impl Default for CsvTransformerBuilder {
    fn default() -> Self {
        CsvTransformerBuilder { separator: DEFAULT_SEPARATOR.into(), quote: DEFAULT_QUOTE, with_header: true }
    }
}

impl CsvTransformerBuilder {
    /// Sets the quote character.
    pub fn quote(mut self, quote: char) -> Self {
        self.quote = quote;
        self
    }

    /// Sets the separator.
    pub fn separator(mut self, separator: &str) -> Self {
        self.separator = separator.into();
        self
    }

    /// Whether the first line names the columns.
    pub fn header(mut self, header: bool) -> Self {
        self.with_header = header;
        self
    }

    /// The transformer.
    pub fn build(self) -> CsvTransformer {
        CsvTransformer { separator: self.separator, quote: self.quote, with_header: self.with_header }
    }
}

impl CsvTransformer {
    /// `CsvTransformer.builder()`.
    pub fn builder() -> CsvTransformerBuilder {
        CsvTransformerBuilder::default()
    }

    fn add_line<IN>(&self, sb: &mut String, value: &Value<'_, IN>) {
        match value {
            Value::Str(s) => self.add_char_sequence(sb, s),
            other => sb.push_str(&other.to_java_string()),
        }
    }

    fn add_char_sequence(&self, sb: &mut String, s: &str) {
        sb.push(self.quote);
        for c in s.chars() {
            sb.push(c);
            if c == self.quote {
                sb.push(self.quote);
            }
        }
        sb.push(self.quote);
    }

    fn generate_header<IN>(&self, schema: &Schema<'_, IN>, sb: &mut String, insert_separator: bool) {
        if !self.with_header {
            return;
        }
        let fields = schema.fields();
        for (i, f) in fields.iter().enumerate() {
            match f.name() {
                Some(name) => self.add_char_sequence(sb, name),
                None => sb.push_str("null"),
            }
            if i < fields.len() - 1 {
                sb.push_str(&self.separator);
            }
        }
        if insert_separator {
            sb.push_str(LINE_SEPARATOR);
        }
    }
}

/// `(SimpleField) field`.
pub(crate) fn simple<'f, 'a, IN>(field: &'f Field<'a, IN>) -> Result<&'f Field<'a, IN>> {
    if field.is_composite() {
        return Err(class_cast_error("net.datafaker.transformations.CompositeField", "net.datafaker.transformations.SimpleField"));
    }
    Ok(field)
}

impl<'a, IN> Transformer<'a, IN> for CsvTransformer {
    fn apply(&self, input: Option<&IN>, schema: &Schema<'a, IN>) -> Result<String> {
        let fields = schema.fields();
        let mut sb = String::new();
        for (i, f) in fields.iter().enumerate() {
            let value = simple(f)?.transform(input)?;
            self.add_line(&mut sb, &value);
            if i < fields.len() - 1 {
                sb.push_str(&self.separator);
            }
        }
        Ok(sb)
    }

    fn generate_from(&self, input: &(impl Input<IN> + ?Sized), schema: &Schema<'a, IN>) -> Result<String> {
        check_finite(input)?;
        let mut sb = String::new();
        self.generate_header(schema, &mut sb, true);
        let mut rows = input.rows().peekable();
        while let Some(row) = rows.next() {
            sb.push_str(&self.apply(row?.as_ref(), schema)?);
            if rows.peek().is_some() {
                sb.push_str(LINE_SEPARATOR);
            }
        }
        Ok(sb)
    }

    fn generate(&self, schema: &Schema<'a, IN>, limit: i32) -> Result<String> {
        let mut sb = String::new();
        self.generate_header(schema, &mut sb, true);
        for i in 0..limit {
            sb.push_str(&self.apply_row(None, schema, i as i64)?);
            if i < limit - 1 {
                sb.push_str(LINE_SEPARATOR);
            }
        }
        Ok(sb)
    }

    fn start_stream(&self, schema: &Schema<'a, IN>) -> Result<String> {
        let mut sb = String::new();
        self.generate_header(schema, &mut sb, false);
        Ok(sb)
    }

    fn end_stream(&self) -> Result<String> {
        Ok(String::new())
    }
}
