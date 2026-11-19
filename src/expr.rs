//! `FakeValuesService`'s expression resolution: `#{directive 'arg1','arg2'}` inside YAML values and
//! `faker.expression(…)`, with upstream's search order (methods on the current provider, YAML keys, methods on the
//! faker, `Provider.method` references, the YAML spelling of the reference) and its per-expression resolver cache.

use std::sync::Arc;

use crate::dispatch::{self, Accessor, AnyObject, JValue, Target};
use crate::{Error, Faker, Result};

/// The current object of a resolution: the faker, a ported provider, `RandomService`, or a user object.
#[derive(Debug, Clone)]
pub(crate) enum Current {
    Target(Target),
    Object(Arc<dyn AnyObject>),
}

impl From<Target> for Current {
    fn from(target: Target) -> Current {
        Current::Target(target)
    }
}

impl Current {
    /// `getClass().getSimpleName()`.
    fn simple_name(&self) -> &str {
        match self {
            Current::Target(t) => t.simple_name(),
            Current::Object(o) => dispatch::simple_name_of(o.class_name()),
        }
    }

    fn is_faker(&self) -> bool {
        matches!(self, Current::Target(Target::Faker))
    }
}

/// A way to produce a value for an expression (`FakeValuesService.ValueResolver`).
#[derive(Debug, Clone)]
pub(crate) enum Resolver {
    /// `ConstantResolver`: `EMPTY_STRING` or `NULL_VALUE`.
    Const(Option<String>),
    /// `SafeFetchResolver`.
    SafeFetch(String),
    /// `MethodResolver` and `MethodAndCoercedArgsResolver`.
    Invoke(Target, Accessor),
    /// The same on a user object: the method's index in its class's table, and the coerced arguments.
    InvokeObject(Arc<dyn AnyObject>, usize, Vec<JValue>),
}

impl Resolver {
    fn resolve(&self, faker: &Faker) -> Result<Option<String>> {
        match self {
            Resolver::Const(v) => Ok(v.clone()),
            Resolver::SafeFetch(key) => faker.fake_values_service().safe_fetch(key, None),
            Resolver::Invoke(target, accessor) => match (accessor.method.invoke)(faker, &accessor.args) {
                Ok(JValue::Null) => Ok(None),
                Ok(v) => Ok(Some(v.to_java_string(faker))),
                Err(e) => Err(Error::runtime(format!(
                    "Failed to invoke MethodAndCoercedArgs[method={}.{}(), coerced={}] on {}",
                    declaring_class(*target),
                    accessor.method.name,
                    coerced_to_string(faker, accessor),
                    target.value().to_java_string(faker),
                ))
                .with_cause(e)),
            },
            Resolver::InvokeObject(object, index, args) => match object.invoke(faker, *index, args) {
                Ok(JValue::Null) => Ok(None),
                Ok(v) => Ok(Some(v.to_java_string(faker))),
                Err(e) => {
                    let class = object.class_name();
                    let coerced = args.iter().map(|a| a.to_java_string(faker)).collect::<Vec<_>>().join(", ");
                    Err(Error::runtime(format!(
                        "Failed to invoke MethodAndCoercedArgs[method={}.{}(), coerced=[{coerced}]] on {class}@{:x}",
                        dispatch::simple_name_of(class),
                        object.method_name(*index),
                        class.len(),
                    ))
                    .with_cause(e))
                }
            },
        }
    }
}

/// `method.getDeclaringClass().getSimpleName()`: the faker's methods are declared on `BaseFaker`.
fn declaring_class(target: Target) -> &'static str {
    match target {
        Target::Faker => "BaseFaker",
        other => other.simple_name(),
    }
}

/// `Arrays.toString(coerced)`: a varargs array prints as `[Ljava.lang.String;@<identity hash>`.
fn coerced_to_string(faker: &Faker, accessor: &Accessor) -> String {
    let m = accessor.method;
    let items: Vec<String> = accessor
        .args
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            if m.varargs && i == m.params.len() - 1 {
                let descriptor = match m.params[i] {
                    dispatch::P::Int => "[I".to_string(),
                    dispatch::P::Long => "[J".to_string(),
                    dispatch::P::Double => "[D".to_string(),
                    dispatch::P::Boolean => "[Z".to_string(),
                    dispatch::P::Char => "[C".to_string(),
                    dispatch::P::String => "[Ljava.lang.String;".to_string(),
                    _ => "[Ljava.lang.Object;".to_string(),
                };
                format!("{descriptor}@{:x}", identity_hash(arg))
            } else {
                match (m.params[i], arg) {
                    // `ChronoUnit.toString()` is its display name (`Days`), not its constant name.
                    (dispatch::P::Enum("ChronoUnit", _), JValue::Enum(name)) => {
                        crate::java::ChronoUnit::value_of(name).map_or_else(|| name.to_string(), |u| u.to_string())
                    }
                    // A `Date` built by `new Date(String)` prints with `Date.toString()`.
                    (dispatch::P::Date, JValue::Long(millis)) => crate::java::Date::new(*millis).to_string(),
                    _ => arg.to_java_string(faker),
                }
            }
        })
        .collect();
    format!("[{}]", items.join(", "))
}

/// A stand-in for `System.identityHashCode`, which differs between JVM runs anyway.
fn identity_hash(value: &JValue) -> u32 {
    value as *const JValue as usize as u32 & 0x7fff_ffff
}

/// `FakeValuesService.resolve(key, current, root, context)`: the YAML value at `key`, resolved with `current` as
/// the provider in whose name bare references are looked up.
pub(crate) fn resolve_key(faker: &Faker, key: &str, current: Option<Current>) -> Result<String> {
    let expression = faker.fake_values_service().safe_fetch(key, None)?;
    let Some(expression) = expression else {
        return Err(Error::runtime(format!("{key} resulted in null expression")));
    };
    resolve_expression(faker, &expression, current)
}

/// `FakeValuesService.resolveExpression(expression, current, root, context)`.
pub(crate) fn resolve_expression(faker: &Faker, expression: &str, current: Option<Current>) -> Result<String> {
    if !expression.contains('}') {
        return Ok(expression.to_string());
    }
    let units: Vec<u16> = expression.encode_utf16().collect();
    let parts = split_expressions(&units);
    let mut result = String::new();
    for (i, part) in parts.iter().enumerate() {
        let expr = String::from_utf16_lossy(part);
        if i % 2 == 0 {
            result.push_str(&expr);
            continue;
        }
        let cached = faker.resolvers.lock().expect("resolver cache").get(&expr).cloned();
        let resolved = match cached {
            Some(resolver) => resolver.resolve(faker)?,
            None => {
                let mut j = 0;
                while j < part.len() && !crate::java::is_whitespace(part[j]) {
                    j += 1;
                }
                let directive = String::from_utf16_lossy(&part[..j]);
                while j < part.len() && crate::java::is_whitespace(part[j]) {
                    j += 1;
                }
                let args = split_arguments(&part[j..]);
                res_exp(faker, &directive, &args, current.as_ref(), &expr)?
            }
        };
        let Some(resolved) = resolved else {
            return Err(Error::runtime(format!(
                "Unable to resolve #{{{expr}}} directive for FakerContext FakerContext{{SingletonLocale{{{}}}, RandomService@java.util.Random@0}}.",
                faker.locale()
            )));
        };
        result.push_str(&resolve_expression(faker, &resolved, current.clone())?);
    }
    Ok(result)
}

/// `FakeValuesService.splitExpressions`: alternating text and `#{…}` contents (odd indices); a `}` inside an odd
/// number of single quotes does not close the expression.
fn split_expressions(expression: &[u16]) -> Vec<Vec<u16>> {
    let length = expression.len();
    let mut list = Vec::new();
    let mut is_expression = false;
    let mut start = 0;
    let mut quote_cnt = 0;
    let mut i = 0;
    while i < length {
        let c = expression[i];
        if is_expression {
            if c == b'}' as u16 && quote_cnt % 2 == 0 {
                list.push(expression[start..i].to_vec());
                start = i + 1;
                is_expression = false;
            } else if c == b'\'' as u16 {
                quote_cnt += 1;
            }
        } else if i + 2 < length && c == b'#' as u16 && expression[i + 1] == b'{' as u16 {
            list.push(expression[start..i].to_vec());
            is_expression = true;
            start = i + 2;
            i += 1;
        }
        i += 1;
    }
    if start < length {
        list.push(expression[start..].to_vec());
    }
    list
}

/// `FakeValuesService.splitArguments`: `'a','b'` → `[a, b]`; `''` inside an argument is a quote.
fn split_arguments(arguments: &[u16]) -> Vec<String> {
    let length = arguments.len();
    let mut result = Vec::new();
    let mut start = 0;
    let mut args_started = false;
    let mut i = 0;
    while i < length {
        if args_started {
            let mut cnt = 0;
            while i < length && arguments[i] == b'\'' as u16 {
                cnt += 1;
                i += 1;
            }
            if cnt % 2 == 1 {
                result.push(String::from_utf16_lossy(&arguments[start..i - 1]).replace("''", "'"));
                args_started = false;
            }
        } else if arguments[i] == b'\'' as u16 {
            args_started = true;
            start = i + 1;
        }
        i += 1;
    }
    result
}

enum Resolution {
    Text(String),
    Resolvers(Vec<Option<Resolver>>),
}

/// `FakeValuesService.resExp`: the first resolver that yields a value wins and is cached for `expr`.
fn res_exp(faker: &Faker, directive: &str, args: &[String], current: Option<&Current>, expr: &str) -> Result<Option<String>> {
    match resolve_directive(faker, directive, args, current)? {
        Resolution::Text(s) => {
            if s.is_empty() {
                faker.resolvers.lock().expect("resolver cache").insert(expr.to_string(), Resolver::Const(Some(String::new())));
            }
            Ok(Some(s))
        }
        Resolution::Resolvers(list) => {
            for resolver in list.into_iter().flatten() {
                if let Some(value) = resolver.resolve(faker)? {
                    faker.resolvers.lock().expect("resolver cache").insert(expr.to_string(), resolver);
                    return Ok(Some(value));
                }
            }
            Ok(None)
        }
    }
}

/// `FakeValuesService.resolveExpression(directive, args, current, root, context)`: the candidate resolvers, in
/// upstream's search order.
fn resolve_directive(faker: &Faker, directive: &str, args: &[String], current: Option<&Current>) -> Result<Resolution> {
    if directive.is_empty() {
        return Ok(Resolution::Text(String::new()));
    }
    let dot = directive.find('.');
    let mut res: Vec<Option<Resolver>> = Vec::new();
    if args.is_empty() {
        if dot.is_none() {
            // `current instanceof AbstractProvider`: a no-argument method by exact name.
            match current {
                Some(Current::Target(Target::Provider(p))) => {
                    if let Some(m) = dispatch::method_by_name(Target::Provider(*p), directive) {
                        res.push(Some(Resolver::Invoke(Target::Provider(*p), Accessor { method: m, args: Vec::new() })));
                        return Ok(Resolution::Resolvers(res));
                    }
                }
                Some(Current::Object(o)) if o.is_provider() => {
                    if let Some(index) = o.method_by_name(directive) {
                        res.push(Some(Resolver::InvokeObject(o.clone(), index, Vec::new())));
                        return Ok(Resolution::Resolvers(res));
                    }
                }
                _ => {}
            }
            res.push(resolve_from_method_on(current, directive, args)?);
        }
        if let Some(d) = dot.filter(|d| *d > 0) {
            let provider_class = &directive[..d];
            let method_name = &directive[d + 1..];
            match provider_by_class_name(faker, provider_class)? {
                Some(Current::Target(ap)) => {
                    if let Some(m) = dispatch::method_by_name(ap, method_name) {
                        res.push(Some(Resolver::Invoke(ap, Accessor { method: m, args: Vec::new() })));
                        return Ok(Resolution::Resolvers(res));
                    }
                }
                Some(Current::Object(o)) => {
                    if let Some(index) = o.method_by_name(method_name) {
                        res.push(Some(Resolver::InvokeObject(o, index, Vec::new())));
                        return Ok(Resolution::Resolvers(res));
                    }
                }
                None => {}
            }
        }
    }
    let simple_directive = match (dot, current) {
        (None, Some(c)) => format!("{}.{directive}", java_name_to_yaml_name(c.simple_name())),
        _ => directive.to_string(),
    };
    if args.is_empty() {
        res.push(Some(Resolver::SafeFetch(simple_directive.clone())));
    }
    if dot.is_none() && !current.is_some_and(Current::is_faker) {
        res.push(resolve_from_method_on(Some(&Current::Target(Target::Faker)), directive, args)?);
    }
    if let Some(d) = dot {
        res.push(resolve_faker_object_and_method(faker, directive, d, args)?);
        res.push(Some(Resolver::SafeFetch(java_name_to_yaml_name(&simple_directive))));
    }
    Ok(Resolution::Resolvers(res))
}

/// `ProviderRegistration.getProvider(simpleClassName)`: the faker's no-argument method returning that class (a
/// ported provider, `RandomService`, or a declared user provider), called; the white list applies.
fn provider_by_class_name(faker: &Faker, name: &str) -> Result<Option<Current>> {
    if name == "RandomService" {
        return Ok(Some(Current::Target(Target::RandomService)));
    }
    if let Some(object) = faker.declared_provider_by_class(name)? {
        return Ok(Some(Current::Object(object)));
    }
    match crate::providers::ProviderKind::ALL.iter().find(|p| p.simple_name() == name) {
        Some(kind) => {
            faker
                .check_white_list(kind.class_name())
                .map_err(|e| Error::runtime(format!("Failed to call method net.datafaker.Faker.{name}()")).with_cause(e))?;
            Ok(Some(Current::Target(Target::Provider(*kind))))
        }
        None => Ok(None),
    }
}

/// `FakeValuesService.resolveFromMethodOn`: `None` without an object, `NULL_VALUE` without a matching method.
fn resolve_from_method_on(obj: Option<&Current>, directive: &str, args: &[String]) -> Result<Option<Resolver>> {
    let name = remove_underscore_chars(directive);
    Ok(match obj {
        None => None,
        Some(Current::Target(target)) => Some(match dispatch::accessor(*target, &name, args)? {
            Some(accessor) => Resolver::Invoke(*target, accessor),
            None => Resolver::Const(None),
        }),
        Some(Current::Object(o)) => Some(match o.accessor(&name, args)? {
            Some((index, coerced)) => Resolver::InvokeObject(o.clone(), index, coerced),
            None => Resolver::Const(None),
        }),
    })
}

/// `FakeValuesService.resolveFakerObjectAndMethod`: `Class.method` as `faker.class().method(args)`; a declared
/// accessor of the user's comes before the faker's own methods, as a subclass's method would.
fn resolve_faker_object_and_method(faker: &Faker, key: &str, dot: usize, args: &[String]) -> Result<Option<Resolver>> {
    let class_name = &key[..dot];
    let method_name = if dot == key.len() - 1 { "" } else { &key[dot + 1..] };
    let accessor_name = remove_underscore_chars(class_name);
    let object = match faker.declared_provider_by_accessor(&accessor_name)? {
        Some(object) => Current::Object(object),
        None => {
            let Some(faker_accessor) = dispatch::accessor(Target::Faker, &accessor_name, &[])? else {
                return Ok(None);
            };
            let object = (faker_accessor.method.invoke)(faker, &faker_accessor.args)?;
            let Some(target) = Target::of(&object) else { return Ok(Some(Resolver::Const(None))) };
            Current::Target(target)
        }
    };
    resolve_from_method_on(Some(&object), method_name, args)
}

/// `FakeValuesService.removeUnderscoreChars`: underscores dropped, lower-cased.
pub(crate) fn remove_underscore_chars(string: &str) -> String {
    if !string.contains('_') {
        return string.to_lowercase();
    }
    let original: Vec<u16> = string.encode_utf16().collect();
    let mut res = original.clone();
    let str_len = original.len() as isize;
    let mut offset: isize = 0;
    let mut length: isize = 0;
    let mut i = str_len - 1;
    while i >= offset {
        while i > offset && original[(i - offset) as usize] == b'_' as u16 {
            offset += 1;
        }
        res[i as usize] = res[(i - offset) as usize];
        if res[i as usize] != b'_' as u16 {
            length += 1;
        }
        i -= 1;
    }
    String::from_utf16_lossy(&res[(str_len - length) as usize..str_len as usize]).to_lowercase()
}

/// `FakeValuesService.javaNameToYamlName`: `PhoneNumber` → `phone_number`, `Name.firstName` → `name.first_name`.
pub(crate) fn java_name_to_yaml_name(expression: &str) -> String {
    let units: Vec<u16> = expression.encode_utf16().collect();
    let first_upper = units.first().is_some_and(|c| crate::java::is_upper_case(*c));
    let mut cnt = usize::from(first_upper) + units.iter().skip(1).filter(|c| crate::java::is_upper_case(**c)).count();
    if cnt == 0 {
        return expression.to_string();
    }
    let mut res: Vec<u16> = Vec::with_capacity(units.len() + cnt);
    for &c in &units {
        if cnt > 0 && crate::java::is_upper_case(c) {
            if !res.is_empty() {
                res.push(b'_' as u16);
            }
            res.push(crate::java::to_lower_case(c));
            cnt -= 1;
        } else {
            res.push(c);
        }
    }
    String::from_utf16_lossy(&res)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    #[test]
    fn expressions_split_into_text_and_directives() {
        let parts: Vec<String> = split_expressions(&u("a #{x} b #{y 'c}'}")).iter().map(|p| String::from_utf16_lossy(p)).collect();
        assert_eq!(parts, ["a ", "x", " b ", "y 'c}'"]);
    }

    #[test]
    fn arguments_split_on_quotes() {
        assert_eq!(split_arguments(&u("'1','10'")), ["1", "10"]);
        assert_eq!(split_arguments(&u("'it''s'")), ["it's"]);
    }

    #[test]
    fn names() {
        assert_eq!(remove_underscore_chars("first_name"), "firstname");
        assert_eq!(remove_underscore_chars("Number_Between"), "numberbetween");
        assert_eq!(java_name_to_yaml_name("Name.firstName"), "name.first_name");
        assert_eq!(java_name_to_yaml_name("RandomService"), "random_service");
        assert_eq!(java_name_to_yaml_name("lower"), "lower");
    }
}
