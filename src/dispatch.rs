//! What Datafaker finds by reflection, as tables: the public methods of the faker, of each provider and of
//! `RandomService`, with their Java parameter types, so that expression resolution can pick an overload and coerce
//! string arguments exactly as `FakeValuesService.accessor`/`coerceArguments` do.

use crate::providers::ProviderKind;
use crate::{Error, Faker, Result};

/// A Java value returned by (or passed to) a method reached through an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum JValue {
    /// Java's `null`.
    Null,
    /// A `String`.
    Str(String),
    /// An `int` or `Integer`.
    Int(i32),
    /// A `long` or `Long`.
    Long(i64),
    /// A `double` or `Double`.
    Double(f64),
    /// A `float` or `Float`.
    Float(f32),
    /// A `boolean` or `Boolean`.
    Bool(bool),
    /// A `char` or `Character`, as its UTF-16 code unit.
    Char(u16),
    /// A `List` or an array; printed as `[a, b]`.
    List(Vec<JValue>),
    /// An enum constant, by name.
    Enum(&'static str),
    /// A provider object (`faker.name()`).
    Provider(ProviderKind),
    /// The faker itself.
    Faker,
    /// `faker.random()`.
    RandomService,
    /// A caller's value (a template's list, map or object) that Datafaker only passes along, compares or hashes:
    /// `id` names it in the caller's table (the caller gives `equals` values one id), `hash` is its `hashCode()` and
    /// `class` its binary class name.
    Foreign {
        /// The value's id in the caller's table.
        id: usize,
        /// Its `hashCode()`.
        hash: i32,
        /// Its binary class name.
        class: String,
    },
    /// A `Map`, as its entries in iteration order.
    Map(Vec<(JValue, JValue)>),
}

impl JValue {
    /// A [`JValue::Str`].
    pub fn str(s: impl Into<String>) -> JValue {
        JValue::Str(s.into())
    }

    /// `Objects.toString(value)` (`null` for null).
    pub fn to_java_string(&self, faker: &Faker) -> String {
        match self {
            JValue::Null => "null".into(),
            JValue::Str(s) => s.clone(),
            JValue::Int(i) => i.to_string(),
            JValue::Long(l) => l.to_string(),
            JValue::Double(d) => crate::java::double_to_string(*d),
            JValue::Float(f) => crate::java::float_to_string(*f),
            JValue::Bool(b) => b.to_string(),
            JValue::Char(c) => String::from_utf16_lossy(&[*c]),
            JValue::List(l) => format!("[{}]", l.iter().map(|v| v.to_java_string(faker)).collect::<Vec<_>>().join(", ")),
            JValue::Enum(e) => (*e).to_string(),
            // `AbstractProvider.toString()`: `%s(%s)@%s` with the faker and an identity hash.
            JValue::Provider(p) => format!("{}({})@{:x}", p.simple_name(), faker, p.simple_name().len()),
            JValue::Faker => faker.to_string(),
            JValue::RandomService => "RandomService@java.util.Random".into(),
            JValue::Foreign { hash, class, .. } => format!("{class}@{hash:x}"),
            JValue::Map(entries) => {
                format!("{{{}}}", entries.iter().map(|(k, v)| format!("{}={}", k.to_java_string(faker), v.to_java_string(faker))).collect::<Vec<_>>().join(", "))
            }
        }
    }

    /// `Objects.hashCode(value)` for the values a `HashSet` of method arguments holds.
    pub fn hash_code(&self) -> i32 {
        match self {
            JValue::Null => 0,
            JValue::Str(s) => crate::java::string_hash(s),
            JValue::Int(i) => *i,
            JValue::Long(l) => (*l ^ ((*l as u64) >> 32) as i64) as i32,
            JValue::Double(d) => {
                let bits = if d.is_nan() { 0x7ff8000000000000u64 } else { d.to_bits() };
                (bits ^ (bits >> 32)) as i32
            }
            JValue::Float(f) => if f.is_nan() { 0x7fc00000 } else { f.to_bits() as i32 },
            JValue::Bool(b) => if *b { 1231 } else { 1237 },
            JValue::Char(c) => *c as i32,
            JValue::List(items) => items.iter().fold(1i32, |h, v| h.wrapping_mul(31).wrapping_add(v.hash_code())),
            JValue::Map(entries) => entries.iter().fold(0i32, |h, (k, v)| h.wrapping_add(k.hash_code() ^ v.hash_code())),
            JValue::Foreign { hash, .. } => *hash,
            // Enum constants and Datafaker objects hash by identity.
            JValue::Enum(_) | JValue::Provider(_) | JValue::Faker | JValue::RandomService => 0,
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            JValue::Str(s) => s,
            _ => "",
        }
    }

    pub(crate) fn as_int(&self) -> i32 {
        match self {
            JValue::Int(i) => *i,
            _ => 0,
        }
    }

    pub(crate) fn as_long(&self) -> i64 {
        match self {
            JValue::Long(l) => *l,
            JValue::Int(i) => *i as i64,
            _ => 0,
        }
    }

    pub(crate) fn as_double(&self) -> f64 {
        match self {
            JValue::Double(d) => *d,
            _ => 0.0,
        }
    }

    pub(crate) fn as_bool(&self) -> bool {
        matches!(self, JValue::Bool(true))
    }

    pub(crate) fn as_char(&self) -> u16 {
        match self {
            JValue::Char(c) => *c,
            _ => 0,
        }
    }

    pub(crate) fn as_list(&self) -> &[JValue] {
        match self {
            JValue::List(l) => l,
            _ => &[],
        }
    }
}

impl From<String> for JValue {
    fn from(s: String) -> Self {
        JValue::Str(s)
    }
}

impl From<i32> for JValue {
    fn from(i: i32) -> Self {
        JValue::Int(i)
    }
}

impl From<i64> for JValue {
    fn from(i: i64) -> Self {
        JValue::Long(i)
    }
}

impl From<bool> for JValue {
    fn from(b: bool) -> Self {
        JValue::Bool(b)
    }
}

impl From<Vec<String>> for JValue {
    fn from(v: Vec<String>) -> Self {
        JValue::List(v.into_iter().map(JValue::Str).collect())
    }
}

/// A Java parameter type, as far as overload choice and argument coercion look at it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P {
    /// `byte`.
    Byte,
    /// `short`.
    Short,
    /// `int`.
    Int,
    /// `long`.
    Long,
    /// `float`.
    Float,
    /// `double`.
    Double,
    /// `boolean`.
    Boolean,
    /// `char`.
    Char,
    /// `String`.
    String,
    /// `CharSequence`.
    CharSequence,
    /// `Object`.
    Object,
    /// An enum, with its constants in declaration order.
    Enum(&'static str, &'static [&'static str]),
    /// A class with no `(String)` constructor (`List`, `Map`, `Locale`, `Function`, …): never coercible.
    Opaque(&'static str),
    /// `java.util.Date` (what `<T extends Date>` erases to): coerced through `new Date(String)` (`Date.parse`); the
    /// method body receives the millis as a [`JValue::Long`].
    Date,
}

impl P {
    fn is_primitive(self) -> bool {
        matches!(self, P::Byte | P::Short | P::Int | P::Long | P::Float | P::Double | P::Boolean | P::Char)
    }

    /// `Class.isAssignableFrom` for the reference types in the tables.
    fn is_assignable_from(self, other: P) -> bool {
        self == other || self == P::Object && !other.is_primitive() || self == P::CharSequence && other == P::String
    }
}

/// The body of a method of the faker, a ported provider or `RandomService`.
pub type Invoke = fn(&Faker, &[JValue]) -> Result<JValue>;

/// A public method: its name, parameter types (the last one is the component type when `varargs`) and body. The
/// body is an [`Invoke`] for the faker, the ported providers and `RandomService`, and an `fn(&T, &Faker, &[JValue])`
/// for the methods of a user class `T` ([`ObjectMethod`]).
#[derive(Clone, Copy)]
pub struct Method<F = Invoke> {
    /// The method's name.
    pub name: &'static str,
    /// The parameter types.
    pub params: &'static [P],
    /// Whether the last parameter is a varargs array.
    pub varargs: bool,
    /// The body.
    pub invoke: F,
    /// `getReturnType().getSimpleName()` (`String`, `int`, `void`, `Age`), for
    /// [`object_methods`](crate::object_methods); empty where nothing asks for it.
    pub returns: &'static str,
    /// Whether the body takes a [`JValue::Null`] argument (one [`accessor_nullable`] binds to a reference
    /// parameter) and does what the Java method does with `null`; a method without it is not found for one.
    pub nulls: bool,
    /// Whether the body also takes a varargs array holding a `null` (`option("a", null)`).
    pub null_items: bool,
}

/// A public method of the user class `T` (see [`JavaClass`]).
pub type ObjectMethod<T> = Method<fn(&T, &Faker, &[JValue]) -> Result<JValue>>;

impl<T> Method<fn(&T, &Faker, &[JValue]) -> Result<JValue>> {
    /// A method of a user class.
    pub const fn object(name: &'static str, params: &'static [P], invoke: fn(&T, &Faker, &[JValue]) -> Result<JValue>) -> Self {
        Method { name, params, varargs: false, invoke, returns: "", nulls: false, null_items: false }
    }

    /// A varargs method of a user class: its last parameter type is the component type, and its argument a
    /// [`JValue::List`].
    pub const fn object_varargs(name: &'static str, params: &'static [P], invoke: fn(&T, &Faker, &[JValue]) -> Result<JValue>) -> Self {
        Method { name, params, varargs: true, invoke, returns: "", nulls: false, null_items: false }
    }
}

impl<F> Method<F> {
    /// The same method with the simple name of its return type (`getReturnType().getSimpleName()`).
    pub const fn returning(mut self, returns: &'static str) -> Self {
        self.returns = returns;
        self
    }

    /// The same method with a body that handles a `null` argument as the Java one does (see [`Method::nulls`]).
    pub const fn accepting_null(mut self) -> Self {
        self.nulls = true;
        self
    }

    /// [`Method::accepting_null`], with a body that also takes `null` elements of its varargs array.
    pub const fn accepting_null_items(mut self) -> Self {
        self.nulls = true;
        self.null_items = true;
        self
    }
}

impl Method {
    /// A method of the faker, a provider or `RandomService` with fixed parameters.
    pub const fn new(name: &'static str, params: &'static [P], invoke: fn(&Faker, &[JValue]) -> Result<JValue>) -> Method {
        Method { name, params, varargs: false, invoke, returns: "", nulls: false, null_items: false }
    }

    /// [`Self::new`] for a varargs method.
    pub const fn varargs(name: &'static str, params: &'static [P], invoke: fn(&Faker, &[JValue]) -> Result<JValue>) -> Method {
        Method { name, params, varargs: true, invoke, returns: "", nulls: false, null_items: false }
    }
}

impl<F> std::fmt::Debug for Method<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:?})", self.name, self.params)
    }
}

/// An object methods can be called on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// The faker.
    Faker,
    /// A provider.
    Provider(ProviderKind),
    /// `RandomService`.
    RandomService,
}

impl Target {
    /// The target `value` is, if methods can be called on it.
    pub fn of(value: &JValue) -> Option<Target> {
        match value {
            JValue::Faker => Some(Target::Faker),
            JValue::Provider(p) => Some(Target::Provider(*p)),
            JValue::RandomService => Some(Target::RandomService),
            _ => None,
        }
    }

    /// The target's public methods.
    pub fn methods(self) -> &'static [Method] {
        match self {
            Target::Faker => crate::faker::FAKER_METHODS.as_slice(),
            Target::Provider(p) => p.methods(),
            Target::RandomService => crate::random_methods::METHODS,
        }
    }

    /// `getClass().getSimpleName()`.
    pub fn simple_name(self) -> &'static str {
        match self {
            Target::Faker => "Faker",
            Target::Provider(p) => p.simple_name(),
            Target::RandomService => "RandomService",
        }
    }

    /// The target as a value.
    pub fn value(self) -> JValue {
        match self {
            Target::Faker => JValue::Faker,
            Target::Provider(p) => JValue::Provider(p),
            Target::RandomService => JValue::RandomService,
        }
    }
}

/// `ObjectMethods.getMethodByName(object, name)`: a public method with no parameters, by exact name.
pub fn method_by_name(target: Target, name: &str) -> Option<&'static Method> {
    target.methods().iter().find(|m| m.name == name && m.params.is_empty())
}

/// A method found by `FakeValuesService.accessor`, with its arguments already coerced.
#[derive(Debug, Clone)]
pub struct Accessor {
    /// The method.
    pub method: &'static Method,
    /// Its arguments, coerced to its parameter types.
    pub args: Vec<JValue>,
}

/// `FakeValuesService.accessor(clazz, name, args)`: the methods whose lower-cased name equals `name` (underscores
/// removed), whose arity fits and whose arguments coerce; the most restrictive one wins, the first one on a tie.
pub fn accessor(target: Target, name: &str, args: &[String]) -> Result<Option<Accessor>> {
    Ok(find_accessor(target.methods(), name, args)?.map(|(method, args)| Accessor { method, args }))
}

/// [`accessor`] for a call made by reflection from outside an expression (a Velocity template's
/// `$faker.dune().quote(null)`), where an argument may be `null` (`None`): it binds to any reference parameter as
/// [`JValue::Null`] and to no primitive one. The method chosen for a `null` argument is not found unless its body
/// takes one ([`Method::accepting_null`]).
///
/// A call holding a `null` is resolved as Velocity's `MethodMap.getBestMatch` resolves it, not by Datafaker's rule:
/// with only `null` arguments a method ending in an array (varargs or not) loses to one that does not; otherwise the
/// most specific overload wins, and overloads neither of which is more specific (`Date` against `String`) make the
/// call ambiguous, so nothing is found and the template renders the reference as written. A single `null` in a
/// varargs position is passed as the array itself, as `VelMethodImpl.invoke` passes it.
pub fn accessor_nullable(target: Target, name: &str, args: &[Option<String>]) -> Result<Option<Accessor>> {
    let args: Vec<Option<&str>> = args.iter().map(Option::as_deref).collect();
    let found = if args.iter().any(Option::is_none) { velocity_best_match(target.methods(), name, &args)? } else { find_accessor_of(target.methods(), name, &args)? };
    let takes = |method: &Method, args: &[JValue]| {
        args.iter().all(|a| match a {
            JValue::Null => method.nulls,
            JValue::List(items) => method.null_items || !items.iter().any(holds_null),
            _ => true,
        })
    };
    Ok(found.filter(|(method, args)| takes(method, args)).map(|(method, args)| Accessor { method, args }))
}

/// Whether `m`'s last parameter is an array (`MethodMap`'s "varargs" test, which a `char[]` parameter passes too).
fn ends_in_array<F>(m: &Method<F>) -> bool {
    m.varargs || matches!(m.params.last(), Some(P::Opaque(name)) if name.ends_with("[]"))
}

/// The type of `m`'s parameter that argument `j` binds to.
fn param_at<F>(m: &Method<F>, j: usize) -> P {
    m.params[j.min(m.params.len() - 1)]
}

/// Widening order of the primitives: `byte` converts to `short`, …, `float` to `double`.
fn widening_rank(p: P) -> Option<u8> {
    [P::Byte, P::Short, P::Int, P::Long, P::Float, P::Double].iter().position(|q| *q == p).map(|i| i as u8)
}

/// `MethodMap.compare` for two overloads over `n` arguments: `Less` when `a` is more specific than `b`, `Greater`
/// when it is less specific, `Equal` when neither converts to the other one way only (incomparable or equivalent).
fn more_specific<F>(a: &Method<F>, b: &Method<F>, n: usize) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    let (mut a_to_b, mut b_to_a) = (true, true);
    for j in 0..n {
        let (pa, pb) = (param_at(a, j), param_at(b, j));
        if pa == pb {
            continue;
        }
        let (a_fits_b, b_fits_a) = match (widening_rank(pa), widening_rank(pb)) {
            (Some(ra), Some(rb)) => (ra < rb, rb < ra),
            _ => (pb.is_assignable_from(pa) || pb == P::Object, pa.is_assignable_from(pb) || pa == P::Object),
        };
        a_to_b &= a_fits_b;
        b_to_a &= b_fits_a;
    }
    match (a_to_b, b_to_a) {
        (true, false) => Less,
        (false, true) => Greater,
        _ => match (ends_in_array(a), ends_in_array(b)) {
            (false, true) => Less,
            (true, false) => Greater,
            _ => Equal,
        },
    }
}

/// `MethodMap.getBestMatch` over the overloads whose arity fits and whose arguments coerce: `None` when none does or
/// the call is ambiguous.
fn velocity_best_match<'m, F>(methods: &'m [Method<F>], name: &str, args: &[Option<&str>]) -> Result<Option<(&'m Method<F>, Vec<JValue>)>> {
    let only_nulls = args.iter().all(Option::is_none);
    let mut best: Vec<(&Method<F>, Vec<JValue>)> = Vec::new();
    for m in methods.iter().filter(|m| m.name.to_ascii_lowercase() == name) {
        let n = m.params.len();
        if !(n == args.len() || n < args.len() && m.varargs) {
            continue;
        }
        // Velocity converts no string to an enum constant, a `char` or a `Date` (`$faker.planet().symbol('EARTH')`
        // renders as written), though Datafaker's own coercion would.
        if (0..args.len()).any(|j| args[j].is_some() && matches!(param_at(m, j), P::Enum(..) | P::Char | P::Date)) {
            continue;
        }
        let Some(mut coerced) = coerce_arguments(m, args)? else { continue };
        if m.varargs && n == args.len() && args[n - 1].is_none() {
            coerced[n - 1] = JValue::Null;
        }
        let mut keep = true;
        best.retain(|(b, _)| {
            if !keep {
                return true;
            }
            let order = if only_nulls {
                match (ends_in_array(m), ends_in_array(b)) {
                    (false, true) => std::cmp::Ordering::Less,
                    (true, false) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                }
            } else {
                more_specific(m, b, args.len())
            };
            match order {
                std::cmp::Ordering::Less => false,
                std::cmp::Ordering::Greater => {
                    keep = false;
                    true
                }
                std::cmp::Ordering::Equal => true,
            }
        });
        if keep {
            best.push((m, coerced));
        }
    }
    Ok(if best.len() == 1 { best.pop() } else { None })
}

/// `String.valueOf(value)` of a `String` argument a body concatenates: `null` for null.
pub(crate) fn java_str(value: &JValue) -> &str {
    match value {
        JValue::Null => "null",
        value => value.as_str(),
    }
}

/// A `String` argument, `None` for null (`separatorOrNull == null ? default : separatorOrNull`).
pub(crate) fn opt_str(value: &JValue) -> Option<&str> {
    match value {
        JValue::Null => None,
        value => Some(value.as_str()),
    }
}

fn holds_null(value: &JValue) -> bool {
    match value {
        JValue::Null => true,
        JValue::List(items) => items.iter().any(holds_null),
        _ => false,
    }
}

/// Argument `i` of a method body that dereferences it, or the `NullPointerException` (Java's helpful text
/// `message`) the Java body throws for a `null` one.
pub(crate) fn non_null<'a>(args: &'a [JValue], i: usize, message: &str) -> Result<&'a JValue> {
    match &args[i] {
        JValue::Null => Err(Error::new("NullPointerException", message)),
        value => Ok(value),
    }
}

/// The `NullPointerException` a Java body throws for a `null` argument it dereferences, with Java's helpful text.
pub(crate) fn npe(message: &str) -> Error {
    Error::new("NullPointerException", message)
}

/// The elements of a `List<String>` or `String[]` argument as a Java body appends them: `null` for a null one; any
/// other object fails the checkcast to `String`.
pub(crate) fn java_strings(list: &JValue) -> Result<Vec<String>> {
    list.as_list()
        .iter()
        .map(|v| match v {
            JValue::Str(s) => Ok(s.clone()),
            JValue::Null => Ok("null".into()),
            JValue::Foreign { class, .. } => Err(class_cast(class, "java.lang.String")),
            other => Err(class_cast(&format!("{other:?}"), "java.lang.String")),
        })
        .collect()
}

/// The `ClassCastException` of a checkcast of an object of class `from` (a binary name: `java.lang.Integer`,
/// `[Ljava.lang.String;`) to `to`, with the JVM's text naming each class's module and loader: the JDK's classes are
/// in `java.base`, anything else (MockServer's and its shaded libraries') in the application loader's unnamed module.
pub fn class_cast(from: &str, to: &str) -> Error {
    fn jdk(class: &str) -> bool {
        let element = class.trim_start_matches('[');
        let element = element.strip_prefix('L').and_then(|c| c.strip_suffix(';')).unwrap_or(element);
        element.len() == 1 || element.starts_with("java.")
    }
    fn place(class: &str) -> &'static str {
        if jdk(class) { "module java.base of loader 'bootstrap'" } else { "unnamed module of loader 'app'" }
    }
    let detail = if jdk(from) == jdk(to) {
        format!("{from} and {to} are in {}", place(from))
    } else {
        format!("{from} is in {}; {to} is in {}", place(from), place(to))
    };
    Error::new("ClassCastException", format!("class {from} cannot be cast to class {to} ({detail})"))
}

/// [`accessor`] over any method table: the method chosen and its coerced arguments.
pub(crate) fn find_accessor<'m, F>(methods: &'m [Method<F>], name: &str, args: &[String]) -> Result<Option<(&'m Method<F>, Vec<JValue>)>> {
    let args: Vec<Option<&str>> = args.iter().map(|a| Some(a.as_str())).collect();
    find_accessor_of(methods, name, &args)
}

fn find_accessor_of<'m, F>(methods: &'m [Method<F>], name: &str, args: &[Option<&str>]) -> Result<Option<(&'m Method<F>, Vec<JValue>)>> {
    let mut best: Option<(&Method<F>, Vec<JValue>)> = None;
    for m in methods.iter().filter(|m| m.name.to_ascii_lowercase() == name) {
        let n = m.params.len();
        if !(n == args.len() || n < args.len() && m.varargs) {
            continue;
        }
        let coerced = if args.is_empty() && n == 0 { Some(Vec::new()) } else { coerce_arguments(m, args)? };
        if let Some(coerced) = coerced && right_is_most_restrictive(best.as_ref().map(|b| b.0), m) {
            best = Some((m, coerced));
        }
    }
    Ok(best)
}

fn param_type<F>(m: &Method<F>, j: usize) -> (P, bool) {
    (m.params[j], m.varargs && j == m.params.len() - 1)
}

/// `FakeValuesService.rightIsMostRestrictive(method1, method2)`.
fn right_is_most_restrictive<F>(method1: Option<&Method<F>>, method2: &Method<F>) -> bool {
    let Some(method1) = method1 else { return true };
    const PRIMITIVES: [P; 6] = [P::Byte, P::Short, P::Int, P::Long, P::Float, P::Double];
    for j in 0..method1.params.len() {
        let (p1, array1) = param_type(method1, j);
        let (p2, array2) = param_type(method2, j);
        if (p1, array1) == (p2, array2) {
            continue;
        }
        let prim1 = p1.is_primitive() && !array1;
        let prim2 = p2.is_primitive() && !array2;
        if prim1 && !prim2 {
            return false;
        }
        if !prim1 && prim2 {
            return true;
        }
        if prim1 {
            for primitive in PRIMITIVES {
                if primitive == p1 {
                    return false;
                } else if primitive == p2 {
                    return true;
                }
            }
        }
        // `parameterTypes1[j].isAssignableFrom(parameterTypes2[j])`, arrays covariant.
        let assignable = match (array1, array2) {
            (false, false) => p1.is_assignable_from(p2),
            (false, true) => p1 == P::Object,
            (true, true) => !p1.is_primitive() && p1.is_assignable_from(p2),
            (true, false) => false,
        };
        if assignable {
            return true;
        }
    }
    false
}

/// `FakeValuesService.coerceArguments`: `None` where an argument does not convert (the overload is skipped).
fn coerce_arguments<F>(m: &Method<F>, args: &[Option<&str>]) -> Result<Option<Vec<JValue>>> {
    let mut coerced = Vec::with_capacity(m.params.len());
    for (i, &ty) in m.params.iter().enumerate() {
        let is_vararg = m.varargs && i == m.params.len() - 1;
        if is_vararg {
            let mut items = Vec::new();
            for a in &args[i..] {
                match a.map_or_else(|| null_for(ty), |a| coerce_vararg(ty, a)) {
                    Some(v) => items.push(v),
                    None => return Ok(None),
                }
            }
            coerced.push(JValue::List(items));
            continue;
        }
        let value = match args[i] {
            Some(arg) => coerce(ty, arg)?,
            None => null_for(ty),
        };
        match value {
            Some(v) => coerced.push(v),
            None => return Ok(None),
        }
    }
    Ok(Some(coerced))
}

/// A `null` argument: assignable to a reference parameter, never to a primitive one.
fn null_for(ty: P) -> Option<JValue> {
    (!ty.is_primitive()).then_some(JValue::Null)
}

/// A varargs element goes through the component type's `(String)` constructor.
fn coerce_vararg(ty: P, arg: &str) -> Option<JValue> {
    match ty {
        P::Enum(_, constants) => enum_value(constants, arg),
        // `Object` and `CharSequence` have no `(String)` constructor.
        P::String => Some(JValue::str(arg)),
        P::Int => java_parse_long(arg).filter(|v| i32::try_from(*v).is_ok()).map(|v| JValue::Int(v as i32)),
        P::Long => java_parse_long(arg).map(JValue::Long),
        P::Double => java_parse_double(arg).map(JValue::Double),
        P::Boolean => Some(JValue::Bool(arg.eq_ignore_ascii_case("true"))),
        P::Date => crate::java::Date::parse_millis(arg).ok().map(JValue::Long),
        _ => None,
    }
}

fn enum_value(constants: &'static [&'static str], arg: &str) -> Option<JValue> {
    let name = arg.split_once('.').map_or(arg, |(_, rest)| rest);
    constants.iter().find(|c| **c == name).map(|c| JValue::Enum(c))
}

fn coerce(ty: P, arg: &str) -> Result<Option<JValue>> {
    Ok(match ty {
        P::Enum(_, constants) => enum_value(constants, arg),
        P::Char => match arg.encode_utf16().next() {
            Some(c) => Some(JValue::Char(c)),
            None => return Err(Error::new("StringIndexOutOfBoundsException", "Index 0 out of bounds for length 0")),
        },
        P::Boolean => Some(JValue::Bool(arg.eq_ignore_ascii_case("true"))),
        P::Int => java_parse_long(arg).filter(|v| i32::try_from(*v).is_ok()).map(|v| JValue::Int(v as i32)),
        P::Long => java_parse_long(arg).map(JValue::Long),
        P::Short => java_parse_long(arg).filter(|v| i16::try_from(*v).is_ok()).map(|v| JValue::Int(v as i32)),
        P::Byte => java_parse_long(arg).filter(|v| i8::try_from(*v).is_ok()).map(|v| JValue::Int(v as i32)),
        P::Double => java_parse_double(arg).map(JValue::Double),
        P::Float => java_parse_double(arg).map(|d| JValue::Float(d as f32)),
        P::String | P::CharSequence => Some(JValue::str(arg)),
        P::Date => crate::java::Date::parse_millis(arg).ok().map(JValue::Long),
        P::Object | P::Opaque(_) => None,
    })
}

/// `Long.parseLong(s)`: optional sign, decimal digits, no spaces.
pub fn java_parse_long(s: &str) -> Option<i64> {
    let digits = s.strip_prefix(['+', '-']).unwrap_or(s);
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// `Double.valueOf(s)`: trimmed, an optional `d`/`f` suffix, `NaN` and `Infinity`.
pub fn java_parse_double(s: &str) -> Option<f64> {
    let t = s.trim_matches(|c: char| c <= ' ');
    let body = t.strip_prefix(['+', '-']).unwrap_or(t);
    let negative = t.starts_with('-');
    match body {
        "NaN" => return Some(f64::NAN),
        "Infinity" => return Some(if negative { f64::NEG_INFINITY } else { f64::INFINITY }),
        _ => {}
    }
    let body = body.strip_suffix(['d', 'D', 'f', 'F']).unwrap_or(body);
    if body.is_empty() || !body.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-')) {
        return None;
    }
    let v: f64 = body.parse().ok()?;
    Some(if negative { -v } else { v })
}

/// A class of the user's that expressions reach, as Datafaker reaches it by reflection: a custom provider (an
/// `AbstractProvider` subclass a faker creates with [`Faker::get_provider`] or declares with
/// [`Faker::declare_provider`]), or any object passed as the current object of
/// [`FakeValuesService::resolve_on`](crate::FakeValuesService::resolve_on).
///
/// ```
/// use datafaker::dispatch::{JValue, JavaClass, ObjectMethod};
///
/// struct Insect;
///
/// impl JavaClass for Insect {
///     const NAME: &'static str = "com.example.Insect";
///
///     const METHODS: &'static [ObjectMethod<Self>] = &[ObjectMethod::object("nextInsectName", &[], |_, faker, _| {
///         let names = ["Ant", "Beetle", "Butterfly", "Wasp"];
///         Ok(JValue::str(names[faker.random().next_int_bounded(4)? as usize]))
///     })];
/// }
///
/// let faker = datafaker::Faker::with_seed(1);
/// faker.declare_provider("insect", |_| Insect);
/// assert!(!faker.expression("#{Insect.nextInsectName}").unwrap().is_empty());
/// ```
pub trait JavaClass: Send + Sync + Sized + 'static {
    /// `getName()`: the binary name (`com.example.CustomFakerTest$Insect`). Its simple name (after the last `.` or
    /// `$`, a local class's digits dropped) is how `#{Insect.method}` finds the provider; the white list sees the
    /// whole name.
    const NAME: &'static str;

    /// Whether the class extends `AbstractProvider`: a bare `#{method}` resolved with it as the current object then
    /// tries its no-argument methods by exact name first.
    const PROVIDER: bool = true;

    /// Its public methods.
    const METHODS: &'static [ObjectMethod<Self>];

    /// Its public methods without parameters that return another user object (`Person.age()` returning an `Age`),
    /// which [`object_methods`](crate::object_methods) reaches; expressions do not call them.
    const OBJECTS: &'static [ObjectGetter<Self>] = &[];
}

/// A public method without parameters of the user class `T` that returns a user object (see
/// [`JavaClass::OBJECTS`]).
pub struct ObjectGetter<T: 'static> {
    /// The method's name.
    pub name: &'static str,
    /// `getReturnType().getSimpleName()`.
    pub returns: &'static str,
    /// The body.
    pub get: fn(&T, &Faker) -> Result<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
}

impl<T> ObjectGetter<T> {
    /// A getter `name` returning the class `returns`.
    pub const fn new(name: &'static str, returns: &'static str, get: fn(&T, &Faker) -> Result<std::sync::Arc<dyn std::any::Any + Send + Sync>>) -> Self {
        ObjectGetter { name, returns, get }
    }
}

impl<T> std::fmt::Debug for ObjectGetter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}() -> {}", self.name, self.returns)
    }
}

/// `getSimpleName()` of a binary name: what follows the last `.` or `$`, without the digits a local class's binary
/// name carries there (`Outer$1Property` → `Property`).
pub(crate) fn simple_name_of(name: &str) -> &str {
    name.rsplit(['.', '$']).next().unwrap_or(name).trim_start_matches(|c: char| c.is_ascii_digit())
}

/// A user object, its class erased.
pub(crate) trait AnyObject: Send + Sync {
    fn class_name(&self) -> &'static str;
    fn is_provider(&self) -> bool;
    /// `ObjectMethods.getMethodByName`: a no-argument method by exact name.
    fn method_by_name(&self, name: &str) -> Option<usize>;
    /// [`accessor`] over the class's methods.
    fn accessor(&self, name: &str, args: &[String]) -> Result<Option<(usize, Vec<JValue>)>>;
    fn method_name(&self, index: usize) -> &'static str;
    fn invoke(&self, faker: &Faker, index: usize, args: &[JValue]) -> Result<JValue>;
    fn into_any(self: std::sync::Arc<Self>) -> std::sync::Arc<dyn std::any::Any + Send + Sync>;
}

impl<T: JavaClass> AnyObject for T {
    fn class_name(&self) -> &'static str {
        T::NAME
    }

    fn is_provider(&self) -> bool {
        T::PROVIDER
    }

    fn method_by_name(&self, name: &str) -> Option<usize> {
        T::METHODS.iter().position(|m| m.name == name && m.params.is_empty())
    }

    fn accessor(&self, name: &str, args: &[String]) -> Result<Option<(usize, Vec<JValue>)>> {
        let methods = T::METHODS;
        Ok(find_accessor(methods, name, args)?.map(|(m, args)| (methods.iter().position(|x| std::ptr::eq(x, m)).unwrap_or(0), args)))
    }

    fn method_name(&self, index: usize) -> &'static str {
        T::METHODS[index].name
    }

    fn invoke(&self, faker: &Faker, index: usize, args: &[JValue]) -> Result<JValue> {
        (T::METHODS[index].invoke)(self, faker, args)
    }

    fn into_any(self: std::sync::Arc<Self>) -> std::sync::Arc<dyn std::any::Any + Send + Sync> {
        self
    }
}

impl std::fmt::Debug for dyn AnyObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.class_name())
    }
}
