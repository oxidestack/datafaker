//! Loads YAML as SnakeYAML 2.6's `new Yaml().loadAs(stream, Map.class)` does for Datafaker's files: mappings keep
//! insertion order (`LinkedHashMap`, a duplicate key replaces the value in place), aliases repeat the anchored node,
//! and plain scalars are typed with SnakeYAML's YAML 1.1 resolver (booleans such as `yes`/`off`, integers including
//! `0x`/`0b`/leading-zero octal, floats, nulls).
//!
//! Not resolved: sexagesimal numbers (`1:20`) and timestamps stay strings, and explicit tags are ignored. No
//! bundled file uses them.

use indexmap::IndexMap;
use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser};
use yaml_rust2::scanner::{Marker, TScalarStyle};

/// A loaded YAML node.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A string.
    Str(String),
    /// `Integer`, `Long` or `BigInteger`; all print alike.
    Int(i128),
    /// A floating-point number.
    Float(f64),
    /// A boolean.
    Bool(bool),
    /// `~` or an empty value.
    Null,
    /// A sequence.
    List(Vec<Value>),
    /// A mapping.
    Map(Map),
}

/// A YAML mapping. Keys that SnakeYAML would not make a `String` (`yes:`, `1:`) are kept under a key no `String`
/// lookup can produce, so `map.get("yes")` misses them as Java's does.
pub type Map = IndexMap<String, Value>;

impl Value {
    /// The string, if this is one.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// The mapping, if this is one.
    pub fn as_map(&self) -> Option<&Map> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    /// The sequence, if this is one.
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    /// `Object.toString()` of the Java object SnakeYAML built.
    pub fn to_java_string(&self) -> String {
        match self {
            Value::Str(s) => s.clone(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => crate::java::double_to_string(*f),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::List(l) => format!("[{}]", l.iter().map(Value::to_java_string).collect::<Vec<_>>().join(", ")),
            Value::Map(m) => format!(
                "{{{}}}",
                m.iter().map(|(k, v)| format!("{}={}", display_key(k), v.to_java_string())).collect::<Vec<_>>().join(", ")
            ),
        }
    }
}

const TYPED_KEY: char = '\u{0}';

fn display_key(k: &str) -> &str {
    k.strip_prefix(TYPED_KEY).unwrap_or(k)
}

/// Loads one document; `None` for an empty stream.
///
/// SnakeYAML accepts lines inside a flow collection (`[…]`, `{…}`) indented no deeper than the key that owns it (five
/// bundled files have such lines); `yaml-rust2` rejects them as invalid indentation. The rejected line is indented
/// further and the text parsed again. Leading whitespace on a line inside a flow collection carries no meaning (it
/// is stripped from folded scalars too), so the result is SnakeYAML's.
pub fn load(text: &str) -> Result<Option<Value>, String> {
    // Indenting every continuation line of a flow collection up front spares a parse per rejected line (pt-BR.yml has
    // hundreds: parsing it took seconds); the loop still indents whatever this pass did not foresee.
    let mut text = indent_flow_continuations(text);
    loop {
        let mut builder = Builder::default();
        let mut parser = Parser::new_from_str(&text);
        match parser.load(&mut builder, false) {
            Ok(()) => return Ok(builder.root),
            Err(e) if e.info() == "invalid indentation" => {
                let line = e.marker().line();
                let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
                let target = line.checked_sub(1).and_then(|i| lines.get_mut(i)).filter(|l| !l.starts_with(FLOW_INDENT));
                let Some(l) = target else { return Err(e.to_string()) };
                l.insert_str(0, FLOW_INDENT);
                text = std::borrow::Cow::Owned(lines.join("\n"));
            }
            Err(e) => return Err(e.to_string()),
        }
    }
}

/// The indentation [`load`] gives a rejected line.
const FLOW_INDENT: &str = "                                        ";

/// `text` with each line that continues a flow collection opened on an earlier line indented by [`FLOW_INDENT`]. A
/// collection opens at a `[` or `{` that starts a line's content or follows `: ` or `- `, or that is met inside an
/// open one; quoted scalars and comments are skipped.
fn indent_flow_continuations(text: &str) -> std::borrow::Cow<'_, str> {
    let mut depth = 0usize;
    let mut out = String::with_capacity(text.len());
    let mut changed = false;
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if depth > 0 && !line.starts_with(FLOW_INDENT) && !line.trim().is_empty() {
            out.push_str(FLOW_INDENT);
            changed = true;
        }
        out.push_str(line);
        let chars: Vec<char> = line.chars().collect();
        let mut j = 0;
        let mut quote: Option<char> = None;
        while j < chars.len() {
            let c = chars[j];
            match quote {
                Some('"') if c == '\\' => j += 1,
                Some('\'') if c == '\'' && chars.get(j + 1) == Some(&'\'') => j += 1,
                Some(q) if c == q => quote = None,
                Some(_) => {}
                None => match c {
                    '#' if j == 0 || chars[j - 1].is_whitespace() => break,
                    // a quote opens a scalar only where a scalar starts
                    '"' | '\'' if chars[..j].iter().rev().find(|p| !p.is_whitespace()).is_none_or(|p| matches!(p, '[' | '{' | ',' | ':' | '-' | '?')) => quote = Some(c),
                    '[' | '{' if depth > 0 || opens_value(&chars[..j]) => depth += 1,
                    ']' | '}' if depth > 0 => depth -= 1,
                    _ => {}
                },
            }
            j += 1;
        }
    }
    if changed { std::borrow::Cow::Owned(out) } else { std::borrow::Cow::Borrowed(text) }
}

/// Whether a value may start after `before`: at the start of the line's content, or after `: ` or `- `.
fn opens_value(before: &[char]) -> bool {
    let trimmed: String = before.iter().collect::<String>().trim_end().to_string();
    trimmed.is_empty() || (trimmed.ends_with(':') || trimmed.ends_with('-')) && before.last().is_some_and(|c| c.is_whitespace())
}

#[derive(Default)]
struct Builder {
    stack: Vec<Frame>,
    anchors: Vec<(usize, Value)>,
    root: Option<Value>,
}

enum Frame {
    List(usize, Vec<Value>),
    Map(usize, Map, Option<String>),
}

impl Builder {
    fn push(&mut self, v: Value) {
        match self.stack.last_mut() {
            None => self.root = Some(v),
            Some(Frame::List(_, l)) => l.push(v),
            Some(Frame::Map(_, m, key)) => match key.take() {
                None => *key = Some(key_string(&v)),
                Some(k) => {
                    m.insert(k, v);
                }
            },
        }
    }

    fn anchor(&mut self, id: usize, v: &Value) {
        if id > 0 {
            self.anchors.push((id, v.clone()));
        }
    }
}

fn key_string(v: &Value) -> String {
    match v {
        Value::Str(s) => s.clone(),
        other => format!("{TYPED_KEY}{}", other.to_java_string()),
    }
}

impl MarkedEventReceiver for Builder {
    fn on_event(&mut self, ev: Event, _mark: Marker) {
        match ev {
            Event::Scalar(text, style, anchor, _tag) => {
                let v = if style == TScalarStyle::Plain { resolve_plain(&text) } else { Value::Str(text) };
                self.anchor(anchor, &v);
                self.push(v);
            }
            Event::SequenceStart(anchor, _) => self.stack.push(Frame::List(anchor, Vec::new())),
            Event::MappingStart(anchor, _) => self.stack.push(Frame::Map(anchor, Map::new(), None)),
            Event::SequenceEnd => {
                if let Some(Frame::List(anchor, l)) = self.stack.pop() {
                    let v = Value::List(l);
                    self.anchor(anchor, &v);
                    self.push(v);
                }
            }
            Event::MappingEnd => {
                if let Some(Frame::Map(anchor, m, _)) = self.stack.pop() {
                    let v = Value::Map(m);
                    self.anchor(anchor, &v);
                    self.push(v);
                }
            }
            Event::Alias(id) => {
                let v = self.anchors.iter().rev().find(|(a, _)| *a == id).map(|(_, v)| v.clone()).unwrap_or(Value::Null);
                self.push(v);
            }
            _ => {}
        }
    }
}

/// SnakeYAML's implicit resolvers (`Resolver.addImplicitResolvers`), in its order: BOOL, INT, FLOAT, NULL.
fn resolve_plain(s: &str) -> Value {
    match s {
        "yes" | "Yes" | "YES" | "true" | "True" | "TRUE" | "on" | "On" | "ON" => return Value::Bool(true),
        "no" | "No" | "NO" | "false" | "False" | "FALSE" | "off" | "Off" | "OFF" => return Value::Bool(false),
        "" | "~" | "null" | "Null" | "NULL" => return Value::Null,
        _ => {}
    }
    if let Some(i) = parse_int(s) {
        return Value::Int(i);
    }
    if let Some(f) = parse_float(s) {
        return Value::Float(f);
    }
    Value::Str(s.to_string())
}

fn split_sign(s: &str) -> (i128, &str) {
    match s.as_bytes().first() {
        Some(b'-') => (-1, &s[1..]),
        Some(b'+') => (1, &s[1..]),
        _ => (1, s),
    }
}

/// `[-+]?0b_*[01]+[01_]*`, `[-+]?0_*[0-7]+[0-7_]*`, `[-+]?(?:0|[1-9][0-9_]*)`, `[-+]?0x_*[0-9a-fA-F]+[0-9a-fA-F_]*`,
/// constructed as `SafeConstructor.ConstructYamlInt` does.
fn parse_int(s: &str) -> Option<i128> {
    let (sign, body) = split_sign(s);
    let digits = |rest: &str, radix: u32| -> Option<i128> {
        let clean: String = rest.chars().filter(|c| *c != '_').collect();
        if clean.is_empty() || !rest.trim_start_matches('_').starts_with(|c: char| c.is_digit(radix)) {
            return None;
        }
        i128::from_str_radix(&clean, radix).ok().map(|v| sign * v)
    };
    if let Some(rest) = body.strip_prefix("0b") {
        return digits(rest, 2);
    }
    if let Some(rest) = body.strip_prefix("0x") {
        return digits(rest, 16);
    }
    if body == "0" {
        return Some(0);
    }
    if let Some(rest) = body.strip_prefix('0') {
        return if rest.chars().all(|c| c == '_' || ('0'..='7').contains(&c)) { digits(rest, 8) } else { None };
    }
    if body.starts_with(|c: char| ('1'..='9').contains(&c)) && body.chars().all(|c| c == '_' || c.is_ascii_digit()) {
        return digits(body, 10);
    }
    None
}

/// `[-+]?(\.[0-9]+|[0-9_]+(\.[0-9_]*)?)([eE][-+]?[0-9]+)?`, `[-+]?\.(inf|Inf|INF)`, `\.(nan|NaN|NAN)`.
fn parse_float(s: &str) -> Option<f64> {
    match s {
        ".nan" | ".NaN" | ".NAN" => return Some(f64::NAN),
        _ => {}
    }
    let (sign, body) = split_sign(s);
    let sign = sign as f64;
    if matches!(body, ".inf" | ".Inf" | ".INF") {
        return Some(sign * f64::INFINITY);
    }
    let (mantissa, exp) = match body.find(['e', 'E']) {
        Some(i) => (&body[..i], Some(&body[i + 1..])),
        None => (body, None),
    };
    if let Some(e) = exp {
        let e = e.strip_prefix(['-', '+']).unwrap_or(e);
        if e.is_empty() || !e.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
    }
    let ok = if let Some(frac) = mantissa.strip_prefix('.') {
        !frac.is_empty() && frac.chars().all(|c| c.is_ascii_digit())
    } else {
        let (int, frac) = mantissa.split_once('.').map_or((mantissa, ""), |(a, b)| (a, b));
        !int.is_empty() && int.chars().all(|c| c.is_ascii_digit() || c == '_') && frac.chars().all(|c| c.is_ascii_digit() || c == '_')
    };
    if !ok {
        return None;
    }
    let clean: String = body.chars().filter(|c| *c != '_').collect();
    clean.parse::<f64>().ok().map(|v| sign * v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_scalars_are_typed_as_yaml_1_1() {
        assert_eq!(resolve_plain("yes"), Value::Bool(true));
        assert_eq!(resolve_plain("Off"), Value::Bool(false));
        assert_eq!(resolve_plain("~"), Value::Null);
        assert_eq!(resolve_plain("012"), Value::Int(10));
        assert_eq!(resolve_plain("0x1F"), Value::Int(31));
        assert_eq!(resolve_plain("1_000"), Value::Int(1000));
        assert_eq!(resolve_plain("08"), Value::Float(8.0));
        assert_eq!(resolve_plain("1.5e3"), Value::Float(1500.0));
        assert_eq!(resolve_plain("1.2.3"), Value::Str("1.2.3".into()));
        assert_eq!(resolve_plain("Mr."), Value::Str("Mr.".into()));
    }

    #[test]
    fn aliases_repeat_the_anchored_node() {
        let v = load("a: &x [p, q]\nb: *x\nc: 'yes'\nyes: 1\n").expect("yaml").expect("doc");
        let m = v.as_map().expect("map");
        assert_eq!(m["a"], m["b"]);
        assert_eq!(m["c"], Value::Str("yes".into()));
        assert!(!m.contains_key("yes"));
    }
}
