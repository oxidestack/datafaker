//! Datafaker's locale data: `FakeValuesContext`, `FakeValues` (one file of one locale), `FakeValuesGrouping` (the
//! files of a locale grouped by the top-level key, English's spread over `en/*.yml` by `EnFile` path), the
//! load-time rewriting of `FakeValues.enrichMapWithJavaNames`, and `JavaNames`.
//!
//! Files are parsed on first use and kept for the life of the process, as upstream's static caches keep them.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};

use crate::en_files::EN_FILES;
use crate::java;
use crate::locale::Locale;
use crate::yaml::{self, Map, Value};
use crate::{Error, Result};

/// The content of an embedded resource, by upstream's absolute resource path (`/en/name.yml`).
fn resource(path: &str) -> Option<&'static str> {
    static INDEX: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    let index = INDEX.get_or_init(|| crate::resources::FILES.iter().map(|(p, c)| (*p, *c)).collect());
    index.get(path.strip_prefix('/')?).copied()
}

/// `FakeValuesContext`: which data a [`FakeValues`] holds. A bundled file of a locale (`of`), an English file
/// searched under one top-level key (`of_file`), or a YAML document at a URL (`of_url`), for `addUrl`/`addPath`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FakeValuesContext {
    locale: Locale,
    filename: Option<String>,
    path: Option<String>,
    url: Option<String>,
}

impl FakeValuesContext {
    /// `FakeValuesContext.of(locale)`: file name and path are both `getFilename(locale)`.
    pub fn of(locale: &Locale) -> FakeValuesContext {
        let filename = locale_filename(locale);
        FakeValuesContext { locale: locale.clone(), filename: Some(filename.clone()), path: Some(filename), url: None }
    }

    /// `FakeValuesContext.of(locale, url)`: no path, so the paths are the document's top-level keys.
    pub fn of_url(locale: &Locale, url: &str) -> FakeValuesContext {
        FakeValuesContext { locale: locale.clone(), filename: Some(locale_filename(locale)), path: None, url: Some(url.to_string()) }
    }

    /// `FakeValuesContext.of(locale, filename, path)`.
    pub fn of_file(locale: &Locale, filename: &str, path: Option<&str>) -> FakeValuesContext {
        FakeValuesContext { locale: locale.clone(), filename: Some(filename.to_string()), path: path.map(str::to_string), url: None }
    }

    /// `FakeValuesContext.of(locale, filename, path, url)`.
    pub fn of_file_url(locale: &Locale, filename: &str, path: Option<&str>, url: Option<&str>) -> FakeValuesContext {
        FakeValuesContext {
            locale: locale.clone(),
            filename: Some(filename.to_string()),
            path: path.map(str::to_string),
            url: url.map(str::to_string),
        }
    }

    /// The locale.
    pub fn locale(&self) -> &Locale {
        &self.locale
    }

    /// The file name, `en.yml`.
    pub fn filename(&self) -> &str {
        self.filename.as_deref().unwrap_or("")
    }

    /// The path within the file, if only part of it is loaded.
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// The URL loaded instead of the bundled file, if any.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
}

impl fmt::Display for FakeValuesContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let opt = |o: &Option<String>| o.clone().unwrap_or_else(|| "null".into());
        write!(
            f,
            "FakeValuesContext{{SingletonLocale{{{}}}, filename='{}', path='{}', url={}}}",
            self.locale,
            opt(&self.filename),
            opt(&self.path),
            opt(&self.url)
        )
    }
}

/// `FakeValues`: the data of one [`FakeValuesContext`], loaded on first use. `FakeValues::of` returns one shared
/// instance per context, as upstream's `FAKE_VALUES_MAP` does.
#[derive(Debug)]
pub struct FakeValues {
    context: FakeValuesContext,
    values: OnceLock<std::result::Result<Option<&'static Map>, Error>>,
}

impl PartialEq for FakeValues {
    /// Two `FakeValues` are equal when their contexts are.
    fn eq(&self, other: &Self) -> bool {
        self.context == other.context
    }
}

impl FakeValues {
    /// `FakeValues.of(context)`.
    pub fn of(context: FakeValuesContext) -> &'static FakeValues {
        static CACHE: OnceLock<Mutex<HashMap<FakeValuesContext, &'static FakeValues>>> = OnceLock::new();
        let mut cache = CACHE.get_or_init(Default::default).lock().expect("fake values cache");
        cache.entry(context.clone()).or_insert_with(|| Box::leak(Box::new(FakeValues { context, values: OnceLock::new() })))
    }

    /// The context the values were loaded for.
    pub fn context(&self) -> &FakeValuesContext {
        &self.context
    }

    /// `getLocale()`.
    pub fn locale(&self) -> &Locale {
        &self.context.locale
    }

    /// The loaded map under `<language or file name>.faker`; `None` for a locale without data (upstream's empty
    /// map). Reading a URL that does not load is an error (a `RuntimeException` upstream).
    pub fn values(&self) -> Result<Option<&'static Map>> {
        self.values.get_or_init(|| self.load()).clone()
    }

    /// `get(key)`: the top-level node `key`.
    pub fn get(&self, key: &str) -> Result<Option<&'static Value>> {
        Ok(self.values()?.and_then(|m| m.get(key)))
    }

    /// `getPaths()`: the context's path, else the top-level keys of the data (`None` when there are none).
    pub fn paths(&self) -> Result<Option<Vec<String>>> {
        if let Some(path) = &self.context.path {
            return Ok(Some(vec![path.clone()]));
        }
        Ok(self.values()?.filter(|m| !m.is_empty()).map(|m| m.keys().cloned().collect()))
    }

    /// `loadValues()`: the URL if there is one, else `/<language>/<filename>`, `/<filename>.yml` and
    /// `/<language>.yml`, in that order.
    fn load(&self) -> Result<Option<&'static Map>> {
        let language = self.context.locale.language();
        let filename = self.context.filename();
        if let Some(url) = &self.context.url {
            let failed = |e: String| Error::runtime(format!("Failed to read fake values from {url}: {e}"));
            let text = read_url(url).map_err(|e| failed(e.to_string()))?;
            return read_document(&text, language, filename).map_err(failed);
        }
        let paths = if filename.is_empty() {
            vec![format!("/{language}.yml")]
        } else {
            vec![format!("/{language}/{filename}"), format!("/{filename}.yml"), format!("/{language}.yml")]
        };
        for path in paths {
            if let Some(text) = resource(&path) {
                let read = read_document(text, language, filename);
                if let Some(map) = read.unwrap_or_else(|e| panic!("bundled {path} does not parse: {e}")) {
                    return Ok(Some(map));
                }
            }
        }
        Ok(None)
    }
}

impl fmt::Display for FakeValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FakeValues{{{}}}", self.context)
    }
}

/// `URL.openStream()` for the URLs `addPath` makes (`file:` URLs) and plain file paths.
fn read_url(url: &str) -> std::io::Result<String> {
    let path = match url.strip_prefix("file:") {
        Some(rest) => percent_decode(rest.strip_prefix("//").unwrap_or(rest)),
        None if url.contains("://") => {
            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "only file: URLs are read"));
        }
        None => url.to_string(),
    };
    std::fs::read_to_string(path)
}

/// `%XX` escapes of a URL path.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() && let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
            out.push(b);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `readFromStream` and `enrichMapWithJavaNames`: the map under the locale's language, else under the file name,
/// then under `faker`.
fn read_document(text: &str, language: &str, filename: &str) -> std::result::Result<Option<&'static Map>, String> {
    let Some(Value::Map(mut top)) = yaml::load(text)? else { return Ok(None) };
    let key = if top.contains_key(language) { language } else { filename };
    let Some(Value::Map(mut locale_map)) = top.swap_remove(key) else { return Ok(None) };
    let Some(Value::Map(mut faker)) = locale_map.swap_remove("faker") else { return Ok(None) };
    enrich_map_with_java_names(&mut faker);
    Ok(Some(Box::leak(Box::new(faker))))
}

/// `FakeValuesContext.getFilename(locale)`.
pub(crate) fn locale_filename(locale: &Locale) -> String {
    let lang = match locale.language() {
        "iw" => "he",
        "in" => "id",
        "ji" => "yi",
        other => other,
    };
    if locale.country().is_empty() {
        lang.to_string()
    } else if locale.language().is_empty() {
        format!("_{}", locale.country())
    } else {
        format!("{lang}-{}", locale.country())
    }
}

/// `FakeValuesGrouping`: [`FakeValues`] by path; `get(key)` merges the maps of every one registered under `key`.
///
/// Upstream keeps each path's `FakeValues` in a `HashSet`, whose order depends on identity hash codes; here they
/// are merged in the order they were added (a later key wins over an earlier one).
#[derive(Debug, Clone, Default)]
pub struct FakeValuesGrouping {
    fake_values: HashMap<String, Vec<&'static FakeValues>>,
    merged: Arc<Mutex<HashMap<String, Option<&'static Value>>>>,
}

impl FakeValuesGrouping {
    /// `new FakeValuesGrouping()`.
    pub fn new() -> FakeValuesGrouping {
        FakeValuesGrouping::default()
    }

    /// `new FakeValuesGrouping(values)`.
    pub fn with(values: &'static FakeValues) -> Result<FakeValuesGrouping> {
        let mut grouping = FakeValuesGrouping::new();
        grouping.add(FakeValuesInterface::Values(values))?;
        Ok(grouping)
    }

    /// `getEnglishFakeValueGrouping()`: every `EnFile` under its path.
    pub fn english() -> &'static FakeValuesGrouping {
        static ENGLISH: OnceLock<FakeValuesGrouping> = OnceLock::new();
        ENGLISH.get_or_init(|| {
            let mut grouping = FakeValuesGrouping::new();
            for (file, path) in EN_FILES {
                let values = FakeValues::of(FakeValuesContext::of_file(&Locale::english(), file, Some(path)));
                grouping.fake_values.entry(path.to_string()).or_default().push(values);
            }
            grouping
        })
    }

    /// `add(fakeValue)`: a `FakeValues` under each of its paths; a grouping's paths replace this one's.
    pub fn add(&mut self, fake_value: FakeValuesInterface) -> Result<()> {
        match fake_value {
            FakeValuesInterface::Values(values) => {
                for path in values.paths()?.unwrap_or_default() {
                    let set = self.fake_values.entry(path).or_default();
                    if !set.contains(&values) {
                        set.push(values);
                    }
                }
            }
            FakeValuesInterface::English => self.add(FakeValuesInterface::Grouping(Arc::new(FakeValuesGrouping::english().clone())))?,
            FakeValuesInterface::Grouping(other) => {
                self.fake_values.extend(other.fake_values.iter().map(|(k, v)| (k.clone(), v.clone())));
            }
        }
        self.merged = Arc::default();
        Ok(())
    }

    /// `get(key)`: the maps under `key` merged with `putAll`, the first one first.
    pub fn get(&self, key: &str) -> Result<Option<&'static Value>> {
        if let Some(v) = self.merged.lock().expect("merged").get(key) {
            return Ok(*v);
        }
        let mut merged: Option<Value> = None;
        for values in self.fake_values.get(key).into_iter().flatten() {
            let part = values.get(key)?;
            match (&mut merged, part) {
                (None, Some(v)) => merged = Some(v.clone()),
                (Some(Value::Map(m)), Some(Value::Map(more))) => m.extend(more.iter().map(|(k, v)| (k.clone(), v.clone()))),
                _ => {}
            }
        }
        let leaked: Option<&'static Value> = merged.map(|v| &*Box::leak(Box::new(v)));
        Ok(*self.merged.lock().expect("merged").entry(key.to_string()).or_insert(leaked))
    }

    /// The paths with at least one `FakeValues`.
    pub fn paths(&self) -> HashSet<&str> {
        self.fake_values.keys().map(String::as_str).collect()
    }
}

/// `FakeValuesInterface`: what a faker's `FakeValuesService` holds for one locale.
#[derive(Debug, Clone)]
pub enum FakeValuesInterface {
    /// The English grouping (`DEFAULT_LOCALE`), searched through the per-key cache of `english_top_level`.
    English,
    /// One locale file's values.
    Values(&'static FakeValues),
    /// The values of several files for one locale.
    Grouping(Arc<FakeValuesGrouping>),
}

impl FakeValuesInterface {
    /// `getCachedFakeValue(locale)`: the English grouping for `Locale.ENGLISH`, else the locale's bundled data.
    pub(crate) fn bundled(locale: &Locale) -> FakeValuesInterface {
        if locale.is_default() {
            FakeValuesInterface::English
        } else {
            FakeValuesInterface::Values(FakeValues::of(FakeValuesContext::of(locale)))
        }
    }

    /// `get(key)`: the top-level node `key`.
    pub fn get(&self, key: &str) -> Result<Option<&'static Value>> {
        match self {
            FakeValuesInterface::English => Ok(english_top_level(key)),
            FakeValuesInterface::Values(v) => v.get(key),
            FakeValuesInterface::Grouping(g) => g.get(key),
        }
    }
}

/// `FakeValuesGrouping.getEnglishFakeValueGrouping().get(key)`, cached per key as a node.
///
/// Upstream iterates the files of one path in `HashSet` order, which depends on `System.nanoTime()` and so changes
/// between runs; here they are merged in `EnFile` order. No bundled key is defined twice under one path with
/// different values (`values::tests::grouped_files_do_not_conflict`), so the order never shows.
fn english_top_level(key: &str) -> Option<&'static Value> {
    FakeValuesGrouping::english().get(key).expect("bundled English files load")
}

/// One English `FakeValues` (`FakeValuesContext.of(ENGLISH, file, path)`).
#[cfg(test)]
fn english_file(file: &str, path: &str) -> Option<&'static Map> {
    FakeValues::of(FakeValuesContext::of_file(&Locale::english(), file, Some(path))).values().expect("bundled")
}

/// `FakeValues.enrichMapWithJavaNames`: under each top-level map, qualify bare `#{x}` references in string lists
/// and add a Java-style alias (`firstName`) for every key; add a Java-style alias (`PhoneNumber`) for each top-level
/// key with an underscore.
fn enrich_map_with_java_names(result: &mut Map) {
    let mut extra: Vec<(String, Value)> = Vec::new();
    let keys: Vec<String> = result.keys().cloned().collect();
    for key in keys {
        let value = result.get_mut(&key).expect("key");
        if let Value::Map(entry_map) = value {
            prefix_unqualified_in_map(entry_map, &key);
            let aliases: Vec<(String, Value)> = entry_map.iter().map(|(k, v)| (to_java_names(k, true), v.clone())).collect();
            // `entryMap.putAll(nestedMap)`: `nestedMap` is a `HashMap`, so equal aliases resolve in its order.
            let hashes: Vec<i32> = aliases.iter().map(|(k, _)| java::string_hash(k)).collect();
            for i in java::hash_map_order(&hashes) {
                let (k, v) = aliases[i].clone();
                entry_map.insert(k, v);
            }
        }
        if key.contains('_') {
            extra.push((to_java_names(&key, false), result[&key].clone()));
        }
    }
    for (k, v) in extra {
        result.insert(k, v);
    }
}

fn prefix_unqualified_in_map(node: &mut Map, provider_key: &str) {
    for v in node.values_mut() {
        prefix_unqualified_expressions(v, provider_key);
    }
}

/// `FakeValues.prefixUnqualifiedExpressions(node, providerKey)`: rewrites the string lists anywhere under `node`
/// (scalars outside lists stay as they are).
pub fn prefix_unqualified_expressions(node: &mut Value, provider_key: &str) {
    match node {
        Value::Map(m) => prefix_unqualified_in_map(m, provider_key),
        Value::List(l) => rewrite_list(l, provider_key),
        _ => {}
    }
}

/// `FakeValues.rewriteList`: `#{first_name}` → `#{Name.firstName}` in each string of the list, stopping at the
/// first item that is not a string or is shorter than two characters.
fn rewrite_list(list: &mut [Value], provider_key: &str) {
    let capitalized = capitalize(provider_key);
    for item_value in list.iter_mut() {
        let Value::Str(item) = item_value else { break };
        let units: Vec<u16> = item.encode_utf16().collect();
        let len = units.len();
        if len < 2 {
            break;
        }
        let mut j = 0;
        let mut sb: Option<Vec<u16>> = None;
        let mut start = 0;
        while j < len {
            while j + 2 < len && !(units[j] == b'#' as u16 && units[j + 1] == b'{' as u16) {
                j += 1;
            }
            let start_word = j + 2;
            let mut letter_or_digit_only = true;
            j = start_word;
            while j < len && units[j] != b'}' as u16 {
                let c = units[j];
                letter_or_digit_only &= java::is_letter(c) || java::is_digit(c) || c == b'_' as u16;
                j += 1;
            }
            if start < len && start_word < len && letter_or_digit_only {
                let out = sb.get_or_insert_with(Vec::new);
                out.extend_from_slice(&units[start..start_word]);
                out.extend(capitalized.encode_utf16());
                out.push(b'.' as u16);
                out.extend(to_java_names(&String::from_utf16_lossy(&units[start_word..j]), true).encode_utf16());
                out.push(b'}' as u16);
                start = j + 1;
            }
        }
        if let Some(mut out) = sb {
            if start < len {
                out.extend_from_slice(&units[start..]);
            }
            *item = String::from_utf16_lossy(&out);
        }
    }
}

/// `WordUtils.capitalize`: the first UTF-16 unit upper-cased unless it already is.
pub fn capitalize(input: &str) -> String {
    let units: Vec<u16> = input.encode_utf16().collect();
    match units.first() {
        None => String::new(),
        Some(&c) if java::is_upper_case(c) => input.to_string(),
        Some(&c) => {
            let mut out = vec![java::to_upper_case(c)];
            out.extend_from_slice(&units[1..]);
            String::from_utf16_lossy(&out)
        }
    }
}

/// `JavaNames.toJavaNames(string, isMethod)`: `first_name` → `firstName` (method) or `FirstName` (class).
pub fn to_java_names(string: &str, is_method: bool) -> String {
    #[derive(Clone, Copy)]
    enum T {
        Same,
        Lower,
        Upper,
    }
    let mut next = if is_method { T::Lower } else { T::Upper };
    let mut res: Vec<u16> = Vec::with_capacity(string.len());
    for c in string.encode_utf16() {
        if java::is_letter(c) {
            res.push(match next {
                T::Same => c,
                T::Lower => java::to_lower_case(c),
                T::Upper => java::to_upper_case(c),
            });
            next = T::Same;
        } else if c == b'_' as u16 {
            next = T::Upper;
        } else {
            res.push(c);
            next = T::Same;
        }
    }
    String::from_utf16_lossy(&res)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strs(v: &[&str]) -> Vec<Value> {
        v.iter().map(|s| Value::Str(s.to_string())).collect()
    }

    #[test]
    fn bare_references_are_qualified_with_the_provider() {
        let mut l = strs(&["#{first_name} #{last_name}", "#{Name.x}", "plain"]);
        rewrite_list(&mut l, "name");
        assert_eq!(l, strs(&["#{Name.firstName} #{Name.lastName}", "#{Name.x}", "plain"]));
    }

    /// Upstream merges the files of one path in an order that changes between runs; that is only invisible while no
    /// key is defined twice with different values.
    #[test]
    fn grouped_files_do_not_conflict() {
        let mut conflicts = Vec::new();
        for (file, path) in EN_FILES {
            for (other, other_path) in EN_FILES {
                if path != other_path || file >= other {
                    continue;
                }
                let (Some(a), Some(b)) = (english_file(file, path), english_file(other, other_path)) else { continue };
                let (Some(Value::Map(a)), Some(Value::Map(b))) = (a.get(*path), b.get(*path)) else { continue };
                for (k, v) in a {
                    if b.get(k).is_some_and(|w| w != v) {
                        conflicts.push(format!("{path}.{k}: {file} vs {other}"));
                    }
                }
            }
        }
        assert!(conflicts.is_empty(), "{conflicts:#?}");
    }

    #[test]
    fn java_names() {
        assert_eq!(to_java_names("first_name", true), "firstName");
        assert_eq!(to_java_names("phone_number", false), "PhoneNumber");
        assert_eq!(to_java_names("a1_b", true), "a1B");
    }
}
