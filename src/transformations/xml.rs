//! `XmlTransformer`.

use std::cell::Cell;

use super::{Field, Input, LINE_SEPARATOR, Schema, Transformer, Value, check_finite, java_trim, null_name};
use crate::{Error, Result};

const INDENTATION_STEP: i32 = 4;

/// `XmlTransformer`: each field an element named after it. A string value is its text, a collection of fields its
/// children, `null` or an empty collection makes it empty (`<tag/>`). A composite field's named fields are
/// attributes and its first unnamed field the content. Pretty output puts children on their own lines, indented by
/// four spaces per level.
///
/// Like upstream, the nesting depth is state of the transformer (not reset when a row fails).
#[derive(Debug, Default)]
pub struct XmlTransformer {
    pretty: bool,
    tag_index: Cell<i32>,
}

/// `XmlTransformer.XmlTransformerBuilder`: not pretty by default.
#[derive(Debug, Clone, Default)]
pub struct XmlTransformerBuilder {
    pretty: bool,
}

impl XmlTransformerBuilder {
    /// A builder of a compact (not pretty) transformer.
    pub fn new() -> XmlTransformerBuilder {
        XmlTransformerBuilder::default()
    }

    /// Whether elements go on lines of their own, indented.
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// The transformer.
    pub fn build(self) -> XmlTransformer {
        XmlTransformer { pretty: self.pretty, tag_index: Cell::new(0) }
    }
}

impl XmlTransformer {
    /// `new XmlTransformer.XmlTransformerBuilder<>()`.
    pub fn builder() -> XmlTransformerBuilder {
        XmlTransformerBuilder::default()
    }

    fn apply_node<'a, IN>(&self, input: Option<&IN>, sb: &mut String, node: &Field<'a, IN>) -> Result<()> {
        if self.pretty && self.tag_index.get() > 0 {
            sb.push_str(LINE_SEPARATOR);
            sb.push_str(&offset(self.tag_index.get()));
        }
        let tag = java_trim(node.name().ok_or_else(|| null_name("trim()"))?).to_string();
        sb.push('<');
        sb.push_str(&tag);
        let content = if node.is_composite() {
            let attrs = node.fields();
            self.apply_attributes(input, sb, attrs)?;
            attrs.iter().find(|f| f.name().is_none())
        } else {
            Some(node)
        };
        self.apply_tag(input, sb, content, &tag)
    }

    fn apply_tag<'a, IN>(&self, input: Option<&IN>, sb: &mut String, field: Option<&Field<'a, IN>>, tag: &str) -> Result<()> {
        let Some(field) = field else {
            apply_value(sb, tag, None);
            return Ok(());
        };
        match field.transform(input)? {
            Value::List(children) => {
                if children.is_empty() {
                    apply_value(sb, tag, None);
                } else {
                    sb.push('>');
                    self.tag_index.set(self.tag_index.get() + 1);
                    for child in &children {
                        match child {
                            Value::Field(f) => self.apply_node(input, sb, f)?,
                            Value::Null => return Err(field_null()),
                            other => return Err(other.class_cast_error("net.datafaker.transformations.Field")),
                        }
                    }
                    self.tag_index.set(self.tag_index.get() - 1);
                    if self.pretty {
                        sb.push_str(LINE_SEPARATOR);
                        sb.push_str(&offset(self.tag_index.get()));
                    }
                    sb.push_str("</");
                    sb.push_str(tag);
                    sb.push('>');
                }
            }
            Value::Str(s) => apply_value(sb, tag, Some(&s)),
            Value::Null => apply_value(sb, tag, None),
            _ => {}
        }
        Ok(())
    }

    fn apply_attributes<'a, IN>(&self, input: Option<&IN>, sb: &mut String, attrs: &[Field<'a, IN>]) -> Result<()> {
        for attr in attrs {
            if let Some(name) = attr.name() {
                let value = match attr.transform(input)? {
                    Value::Str(s) => s,
                    Value::Null => {
                        return Err(Error::new("NullPointerException", "Cannot invoke \"String.length()\" because \"str\" is null"));
                    }
                    other => return Err(other.class_cast_error("java.lang.String")),
                };
                sb.push(' ');
                sb.push_str(name);
                sb.push_str("=\"");
                sb.push_str(&escape(&value));
                sb.push('"');
            }
        }
        Ok(())
    }
}

/// A `null` child: `xmlNode.getName()` on it.
fn field_null() -> Error {
    Error::new("NullPointerException", "Cannot invoke \"net.datafaker.transformations.Field.getName()\" because \"xmlNode\" is null")
}

fn apply_value(sb: &mut String, tag: &str, value: Option<&str>) {
    match value {
        Some(v) => {
            sb.push('>');
            sb.push_str(&escape(v));
            sb.push_str("</");
            sb.push_str(tag);
            sb.push('>');
        }
        None => sb.push_str("/>"),
    }
}

fn offset(level: i32) -> String {
    " ".repeat((level * INDENTATION_STEP).max(0) as usize)
}

fn escape(s: &str) -> String {
    let mut sb = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => sb.push_str("&lt;"),
            '>' => sb.push_str("&gt;"),
            '&' => sb.push_str("&amp;"),
            '\'' => sb.push_str("&apos;"),
            '"' => sb.push_str("&quot;"),
            other => sb.push(other),
        }
    }
    sb
}

impl<'a, IN> Transformer<'a, IN> for XmlTransformer {
    fn apply(&self, input: Option<&IN>, schema: &Schema<'a, IN>) -> Result<String> {
        let mut sb = String::new();
        for f in schema.fields() {
            self.apply_node(input, &mut sb, f)?;
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
