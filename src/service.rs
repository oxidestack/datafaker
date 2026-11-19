//! `net.datafaker.service.FakeValuesService`: YAML lookup along the locale chain, and the template functions
//! (`numerify`, `letterify`, `bothify`, `examplify`, `templatify`, `regexify`). Expression resolution is in
//! `expr.rs`.

use crate::random_service::RandomService;
use crate::transformations::{CsvTransformer, DEFAULT_QUOTE, DEFAULT_SEPARATOR, Field, JsonTransformer, Schema, Transformer, field};
use crate::yaml::Value;
use crate::{Error, Faker, Result};

const DIGITS: &[u8; 10] = b"0123456789";

/// `FakeValuesService` bound to one faker's `FakerContext`.
pub struct FakeValuesService<'a> {
    faker: &'a Faker,
}

impl<'a> FakeValuesService<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        FakeValuesService { faker }
    }

    fn random(&self) -> RandomService {
        self.faker.random()
    }

    /// `fetchObject(key, context)`: the node at the dotted `key` in the first locale of the chain that has one.
    pub fn fetch_object(&self, key: &str) -> Option<&'static Value> {
        let mut path = split(key);
        if let Some(Some(first)) = path.first_mut() {
            *first = first.to_lowercase();
        }
        for locale in self.faker.locale_chain() {
            let Some(Some(first)) = path.first() else { return None };
            // A locale this faker's service has no entry for is skipped (upstream's `fakeValuesInterfaceMap`).
            let Some(interface) = self.faker.fake_values_interface(&locale) else { continue };
            let mut current = interface.get(first).unwrap_or_else(|e| panic!("{e}"));
            for segment in &path[1..] {
                current = match (current, segment) {
                    (Some(Value::Map(m)), Some(s)) => m.get(s.as_str()),
                    _ => None,
                };
            }
            // A YAML null reads as `null` from the map, the same as a missing key.
            if current.is_some_and(|v| *v != Value::Null) {
                return current;
            }
        }
        None
    }

    /// `fetch(key, context)`: a random element of the list at `key` (the only one without a draw), else `None`.
    pub fn fetch(&self, key: &str) -> Result<Option<&'static Value>> {
        let Some(Value::List(values)) = self.fetch_object(key) else { return Ok(None) };
        Ok(match values.len() {
            0 => None,
            1 => Some(&values[0]),
            n => Some(&values[self.random().next_int_bounded(n as i32)? as usize]),
        })
    }

    /// `fetchString(key, context)`.
    pub fn fetch_string(&self, key: &str) -> Result<Option<String>> {
        match self.fetch(key)? {
            None => Ok(None),
            Some(Value::Str(s)) => Ok(Some(s.clone())),
            Some(other) => Err(class_cast(other)),
        }
    }

    /// `safeFetch(key, context, defaultIfNull)`: a random element of a list, a slash-delimited regex turned into
    /// `#{regexify '…'}`, or the string itself.
    pub fn safe_fetch(&self, key: &str, default_if_null: Option<&str>) -> Result<Option<String>> {
        let Some(o) = self.fetch_object(key) else { return Ok(default_if_null.map(str::to_string)) };
        if let Value::List(values) = o {
            let picked = match values.len() {
                0 => return Ok(default_if_null.map(str::to_string)),
                1 => &values[0],
                n => &values[self.random().next_int_bounded(n as i32)? as usize],
            };
            return match picked {
                Value::Str(s) => Ok(Some(s.clone())),
                Value::Null => Ok(None),
                other => Err(class_cast(other)),
            };
        }
        let text = o.to_java_string();
        if text.starts_with('/') && text.ends_with('/') && !text.is_empty() {
            let inner = if text.len() >= 2 { &text[1..text.len() - 1] } else { "" };
            return Ok(Some(format!("#{{regexify '{inner}'}}")));
        }
        match o {
            Value::Str(s) => Ok(Some(s.clone())),
            other => Err(class_cast(other)),
        }
    }

    /// `resolve(key, current, root, context)` with this faker as the root: the YAML value at `key`, its expressions
    /// resolved with `current` as the object whose methods and YAML section bare references name first (`None`
    /// for none, which is also what an object without matching methods amounts to).
    pub fn resolve(&self, key: &str, current: Option<crate::dispatch::Target>) -> Result<String> {
        crate::expr::resolve_key(self.faker, key, current.map(Into::into))
    }

    /// `resolve(key, current, root, context)` with a user object as `current`, as an `AbstractProvider` subclass's
    /// `resolve(key)` passes itself: a bare `#{name}` tries the object's no-argument `name()` (a provider's), then its
    /// methods by `FakeValuesService.accessor`, its class's YAML section (`dummy_service.name`) and the faker. A
    /// resolver found for an expression is cached per faker with the object it was found on, as upstream.
    pub fn resolve_on<T: crate::dispatch::JavaClass>(&self, key: &str, current: &std::sync::Arc<T>) -> Result<String> {
        let object: std::sync::Arc<dyn crate::dispatch::AnyObject> = current.clone();
        crate::expr::resolve_key(self.faker, key, Some(crate::expr::Current::Object(object)))
    }

    /// `expression(expression, faker, context)`.
    pub fn expression(&self, expression: &str) -> Result<String> {
        crate::expr::resolve_expression(self.faker, expression, None)
    }

    /// `fileExpression(path, faker, context)`: each line of the UTF-8 file resolved as an expression, joined with
    /// the line separator (`\n`).
    pub fn file_expression(&self, path: impl AsRef<std::path::Path>) -> Result<String> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|_| Error::runtime(format!("Failed to read \"{}\"", path.display())))?;
        let lines = read_all_lines(&text);
        let resolved: Result<Vec<String>> = lines.iter().map(|line| self.expression(line)).collect();
        Ok(resolved?.join("\n"))
    }

    /// `csv(limit, columnExpressions...)`: `;`-separated, `"`-quoted, with a header.
    pub fn csv(&self, limit: i32, column_expressions: &[&str]) -> Result<String> {
        self.csv_with(DEFAULT_SEPARATOR, DEFAULT_QUOTE, true, limit, column_expressions)
    }

    /// `csv(separator, quote, withHeader, limit, columnExpressions...)`: a `CsvTransformer` over a schema of string
    /// fields; name and value alternate in `column_expressions`. The values are not resolved here; the expression that called `csv` resolves the
    /// `#{…}` left in the text, one occurrence at a time.
    pub fn csv_with(&self, separator: &str, quote: char, with_header: bool, limit: i32, column_expressions: &[&str]) -> Result<String> {
        self.csv_with_nullable(separator, quote, with_header, limit, &some(column_expressions))
    }

    /// [`csv_with`](Self::csv_with) over expressions that may be `null` (`None`), as a template passes them: a
    /// `null` name or value is written as `null`, unquoted.
    pub fn csv_with_nullable(&self, separator: &str, quote: char, with_header: bool, limit: i32, column_expressions: &[Option<&str>]) -> Result<String> {
        if column_expressions.len() % 2 == 1 {
            return Err(Error::illegal_argument(format!(
                "Total number of column names and column values should be even (received {} columns: {})",
                column_expressions.len(),
                array_to_string(column_expressions)
            )));
        }
        let fields: Vec<Field<'_>> = column_expressions.chunks(2).map(|c| field(c[0], move || c[1])).collect();
        CsvTransformer::builder().separator(separator).quote(quote).header(with_header).build().generate(&Schema::of(fields), limit)
    }

    /// `json(fieldExpressions...)`: one JSON object from a `JsonTransformer` over string fields; name and value
    /// alternate. A value that starts with `#{json` is written as it is (a nested object, once resolved), any other
    /// one as a string.
    pub fn json(&self, field_expressions: &[&str]) -> Result<String> {
        self.json_nullable(&some(field_expressions))
    }

    /// [`json`](Self::json) over expressions that may be `null` (`None`): a `null` name or value is written as
    /// `null`, unquoted.
    pub fn json_nullable(&self, field_expressions: &[Option<&str>]) -> Result<String> {
        if field_expressions.len() % 2 == 1 {
            return Err(Error::illegal_argument(format!(
                "Total number of field names and field values should be even (received {} fields: {})",
                field_expressions.len(),
                array_to_string(field_expressions)
            )));
        }
        let fields: Vec<Field<'_>> = field_expressions.chunks(2).map(|c| field(c[0], move || c[1])).collect();
        JsonTransformer::builder().build().generate(&Schema::of(fields), 1)
    }

    /// `jsona(fieldExpressions...)`: like `json` with triples of count, name and value; a positive count makes the
    /// value an array (`Object[]`) of that many copies.
    pub fn jsona(&self, field_expressions: &[&str]) -> Result<String> {
        self.jsona_nullable(&some(field_expressions))
    }

    /// [`jsona`](Self::jsona) over expressions that may be `null` (`None`): a `null` count makes a plain field
    /// (`fieldExpressions[i] != null && …`), a `null` name or value is written as `null`.
    pub fn jsona_nullable(&self, field_expressions: &[Option<&str>]) -> Result<String> {
        if !field_expressions.len().is_multiple_of(3) {
            return Err(Error::illegal_argument(format!(
                "Total number of field names and field values should be dividable by 3 (received {} field expressions: {})",
                field_expressions.len(),
                array_to_string(field_expressions)
            )));
        }
        let mut fields: Vec<Field<'_>> = Vec::new();
        for c in field_expressions.chunks(3) {
            let count = match c[0] {
                None => 0,
                Some(n) => crate::dispatch::java_parse_long(n)
                    .filter(|v| i32::try_from(*v).is_ok())
                    .ok_or_else(|| Error::new("NumberFormatException", format!("For input string: \"{n}\"")))?,
            };
            fields.push(if count > 0 {
                let objects = vec![c[2]; count as usize];
                field(c[1], move || objects.clone())
            } else {
                field(c[1], move || c[2])
            });
        }
        JsonTransformer::builder().build().generate(&Schema::of(fields), 1)
    }

    /// `numerify(numberString, context)`.
    pub fn numerify(&self, number_string: &str) -> String {
        self.bothify_impl(number_string, false, true, false)
    }

    /// `letterify(letterString, context, isUpper)`.
    pub fn letterify(&self, letter_string: &str, is_upper: bool) -> String {
        self.bothify_impl(letter_string, is_upper, false, true)
    }

    /// `bothify(input, context, isUpper)`.
    pub fn bothify(&self, input: &str, is_upper: bool) -> String {
        self.bothify_impl(input, is_upper, true, true)
    }

    fn bothify_impl(&self, input: &str, is_upper: bool, numerify: bool, letterify: bool) -> String {
        let base = if is_upper { b'A' } else { b'a' } as u16;
        let mut changed = false;
        let mut res: Vec<u16> = input.encode_utf16().collect();
        let random = self.random();
        let mut i = 0;
        while i < res.len() {
            match res[i] {
                0x23 if numerify => {
                    changed = true;
                    let symbol = res[i];
                    i += generate_and_set_number(i, &mut res, symbol, &random) - 1;
                }
                0xD8 if numerify => {
                    changed = true;
                    res[i] = DIGITS[random.next_int_between(1, 9).expect("1 <= 9") as usize] as u16;
                }
                0x3F if letterify => {
                    changed = true;
                    res[i] = base + random.next_int_bounded(26).expect("26 > 0") as u16;
                }
                _ => {}
            }
            i += 1;
        }
        if changed { String::from_utf16_lossy(&res) } else { input.to_string() }
    }

    /// `regexify(regex, context)`.
    pub fn regexify(&self, regex: &str) -> Result<String> {
        let generator = crate::rgxgen::RgxGen::parse(regex)?;
        generator.try_generate(self.random().random())
    }

    /// `examplify(example, context)`.
    pub fn examplify(&self, example: &str) -> String {
        let mut chars: Vec<u16> = example.encode_utf16().collect();
        for c in chars.iter_mut() {
            if crate::java::is_letter(*c) {
                let upper = crate::java::is_upper_case(*c);
                *c = self.letterify("?", upper).encode_utf16().next().expect("one letter");
            } else if crate::java::is_digit(*c) {
                *c = DIGITS[self.random().next_int_bounded(10).expect("10 > 0") as usize] as u16;
            }
        }
        String::from_utf16_lossy(&chars)
    }

    /// `templatify(letterString, optionsMap, context)`.
    pub fn templatify(&self, letter_string: &str, options: &[(char, &[&str])]) -> Result<String> {
        let mut sb = String::new();
        for key in letter_string.chars() {
            match options.iter().find(|(k, _)| *k == key) {
                Some((_, opts)) => sb.push_str(opts[self.random().next_int_bounded(opts.len() as i32)? as usize]),
                None => sb.push(key),
            }
        }
        Ok(sb)
    }
}

/// `Arrays.toString(String[])`.
/// `Arrays.toString` of a `String[]`: `null` for a null element.
fn array_to_string(items: &[Option<&str>]) -> String {
    format!("[{}]", items.iter().map(|i| i.unwrap_or("null")).collect::<Vec<_>>().join(", "))
}

fn some<'a>(items: &[&'a str]) -> Vec<Option<&'a str>> {
    items.iter().copied().map(Some).collect()
}

/// `Files.readAllLines`: lines end at `\n`, `\r` or `\r\n`, and a final terminator adds no empty line.
fn read_all_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\n' => lines.push(std::mem::take(&mut current)),
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                lines.push(std::mem::take(&mut current));
            }
            other => current.push(other),
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn class_cast(v: &Value) -> Error {
    let class = match v {
        Value::Int(_) => "java.lang.Integer",
        Value::Float(_) => "java.lang.Double",
        Value::Bool(_) => "java.lang.Boolean",
        Value::List(_) => "java.util.ArrayList",
        Value::Map(_) => "java.util.LinkedHashMap",
        Value::Str(_) | Value::Null => "java.lang.String",
    };
    Error::new(
        "ClassCastException",
        format!("class {class} cannot be cast to class java.lang.String ({class} and java.lang.String are in module java.base of loader 'bootstrap')"),
    )
}

/// `FakeValuesService.split`: dot-separated segments, empty ones dropped from the middle, which leaves `None`
/// slots at the end (upstream sizes the array by the number of dots).
fn split(key: &str) -> Vec<Option<String>> {
    let units: Vec<u16> = key.encode_utf16().collect();
    let dots = units.iter().filter(|c| **c == b'.' as u16).count();
    let mut result: Vec<Option<String>> = vec![None; dots + 1];
    let mut start = 0;
    let mut j = 0;
    for (i, c) in units.iter().enumerate() {
        if *c == b'.' as u16 {
            if i > start {
                result[j] = Some(String::from_utf16_lossy(&units[start..i]));
                j += 1;
            }
            start = i + 1;
        }
    }
    result[j] = Some(String::from_utf16_lossy(&units[start..]));
    result
}

/// `GenerationUtils.generateAndSetNumber`: fills the run of `symbol` starting at `position` with digits, nine at a
/// time, and returns the length of the run.
fn generate_and_set_number(position: usize, target: &mut [u16], symbol: u16, random: &RandomService) -> usize {
    const INT_LIMIT_DIGITS: usize = 9;
    const TENS: [i32; 11] = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000, 1_000_000_000, 1_410_065_408];
    let generate = |digits: usize| random.next_int_bounded(TENS[digits + 1]).expect("positive bound");
    let insert = |number: i32, digits: usize, target: &mut [u16], offset: usize| {
        for k in 0..digits {
            target[offset + k] = DIGITS[(number / TENS[k] % 10) as usize] as u16;
        }
    };
    let mut symbol_counter = 0;
    let mut generated = 0;
    loop {
        symbol_counter += 1;
        if symbol_counter - generated == INT_LIMIT_DIGITS {
            let r = generate(INT_LIMIT_DIGITS);
            insert(r, INT_LIMIT_DIGITS, target, position + generated);
            generated += INT_LIMIT_DIGITS;
        }
        if !(position + symbol_counter < target.len() && target[position + symbol_counter] == symbol) {
            break;
        }
    }
    let diff = symbol_counter - generated;
    if diff > 0 {
        let r = generate(diff);
        insert(r, diff, target, position + generated);
    }
    symbol_counter
}

#[cfg(test)]
mod tests {
    use super::split;

    #[test]
    fn split_keeps_upstream_slots() {
        assert_eq!(split("a.b"), [Some("a".into()), Some("b".into())]);
        assert_eq!(split("a..b"), [Some("a".into()), Some("b".into()), None]);
        assert_eq!(split("a."), [Some("a".into()), Some("".into())]);
    }
}
