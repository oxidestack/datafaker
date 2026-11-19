//! Java's `Object` as the transformers see it: [`Value`], one variant per runtime type they test with `instanceof`.

use std::any::Any;
use std::fmt;
use std::sync::Arc;

use super::{Field, Schema};
use crate::java::{BigDecimal, Date, Duration, Instant, LocalDate, LocalDateTime, LocalTime, Period};
use crate::{Error, Result};

/// A value a [`Field`] returns: what a Java field function returned, by runtime type.
///
/// `IN` is the input type of the schema the value may carry (a nested [`Schema`], or [`Field`]s in a collection,
/// which the JSON and XML transformers apply to the current input).
pub enum Value<'a, IN = ()> {
    /// `null`.
    Null,
    /// A `Boolean`.
    Bool(bool),
    /// A `Byte`.
    Byte(i8),
    /// A `Short`.
    Short(i16),
    /// A `Integer`.
    Int(i32),
    /// A `Long`.
    Long(i64),
    /// A `Float`.
    Float(f32),
    /// A `Double`.
    Double(f64),
    /// A `BigInteger`, by its `toString()`.
    BigInteger(String),
    /// A `BigDecimal`, by its `toString()`.
    BigDecimal(String),
    /// A `Character`.
    Char(char),
    /// A `String` (or any other `CharSequence`).
    Str(String),
    /// An array of objects (`Object[]`, `String[]`, `Integer[]`, `int[][]`, …).
    Array(Vec<Value<'a, IN>>),
    /// An array of a primitive type (`int[]`, `byte[]`, …).
    Primitive(PrimitiveArray),
    /// A `Collection` (`List`, `Set`), in iteration order.
    List(Vec<Value<'a, IN>>),
    /// A `Map`, in iteration order.
    Map(Vec<(Value<'a, IN>, Value<'a, IN>)>),
    /// A `Schema` as a value: a nested object (YAML, TOML).
    Schema(Schema<'a, IN>),
    /// A `SimpleField` or `CompositeField` as a value: an element of a JSON collection or an XML child.
    Field(Field<'a, IN>),
    /// Any other object: its `toString()` and the object itself (for [`super::JavaObject`]s to take back).
    Object(Object),
}

/// An array of one primitive type.
#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveArray {
    /// A `boolean[]`.
    Boolean(Vec<bool>),
    /// A `byte[]`.
    Byte(Vec<i8>),
    /// A `short[]`.
    Short(Vec<i16>),
    /// A `char[]`.
    Char(Vec<char>),
    /// A `int[]`.
    Int(Vec<i32>),
    /// A `long[]`.
    Long(Vec<i64>),
    /// A `float[]`.
    Float(Vec<f32>),
    /// A `double[]`.
    Double(Vec<f64>),
}

impl PrimitiveArray {
    /// `getClass().getName()`: `[I`, `[B`, …
    pub fn class_name(&self) -> &'static str {
        match self {
            PrimitiveArray::Boolean(_) => "[Z",
            PrimitiveArray::Byte(_) => "[B",
            PrimitiveArray::Short(_) => "[S",
            PrimitiveArray::Char(_) => "[C",
            PrimitiveArray::Int(_) => "[I",
            PrimitiveArray::Long(_) => "[J",
            PrimitiveArray::Float(_) => "[F",
            PrimitiveArray::Double(_) => "[D",
        }
    }

    /// The number of elements.
    pub fn len(&self) -> usize {
        match self {
            PrimitiveArray::Boolean(v) => v.len(),
            PrimitiveArray::Byte(v) => v.len(),
            PrimitiveArray::Short(v) => v.len(),
            PrimitiveArray::Char(v) => v.len(),
            PrimitiveArray::Int(v) => v.len(),
            PrimitiveArray::Long(v) => v.len(),
            PrimitiveArray::Float(v) => v.len(),
            PrimitiveArray::Double(v) => v.len(),
        }
    }

    /// Whether the array is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// `java.lang.reflect.Array.get(array, i)`: the element, boxed.
    pub fn get<'a, IN>(&self, i: usize) -> Value<'a, IN> {
        match self {
            PrimitiveArray::Boolean(v) => Value::Bool(v[i]),
            PrimitiveArray::Byte(v) => Value::Byte(v[i]),
            PrimitiveArray::Short(v) => Value::Short(v[i]),
            PrimitiveArray::Char(v) => Value::Char(v[i]),
            PrimitiveArray::Int(v) => Value::Int(v[i]),
            PrimitiveArray::Long(v) => Value::Long(v[i]),
            PrimitiveArray::Float(v) => Value::Float(v[i]),
            PrimitiveArray::Double(v) => Value::Double(v[i]),
        }
    }

    /// The elements, boxed.
    pub fn boxed<'a, IN>(&self) -> Vec<Value<'a, IN>> {
        (0..self.len()).map(|i| self.get(i)).collect()
    }

    /// The `ClassCastException` of `(Object[]) array`.
    pub(crate) fn object_array_cast_error(&self) -> Error {
        let name = self.class_name();
        Error::new(
            "ClassCastException",
            format!(
                "class {name} cannot be cast to class [Ljava.lang.Object; ({name} and [Ljava.lang.Object; are in module java.base of loader 'bootstrap')"
            ),
        )
    }
}

/// Any other object a field returns: kept for a [`super::JavaObject`] to take back, printed with its `toString()`.
#[derive(Clone)]
pub struct Object {
    value: Arc<dyn Any + Send + Sync>,
    text: String,
}

impl Object {
    /// Wraps `value`, printed with its `Display`.
    pub fn new<T: fmt::Display + Any + Send + Sync>(value: T) -> Object {
        let text = value.to_string();
        Object { value: Arc::new(value), text }
    }

    /// The object, if it is a `T`.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.value.downcast_ref()
    }

    /// `toString()`.
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object({})", self.text)
    }
}

impl<IN> Clone for Value<'_, IN> {
    fn clone(&self) -> Self {
        match self {
            Value::Null => Value::Null,
            Value::Bool(v) => Value::Bool(*v),
            Value::Byte(v) => Value::Byte(*v),
            Value::Short(v) => Value::Short(*v),
            Value::Int(v) => Value::Int(*v),
            Value::Long(v) => Value::Long(*v),
            Value::Float(v) => Value::Float(*v),
            Value::Double(v) => Value::Double(*v),
            Value::BigInteger(v) => Value::BigInteger(v.clone()),
            Value::BigDecimal(v) => Value::BigDecimal(v.clone()),
            Value::Char(v) => Value::Char(*v),
            Value::Str(v) => Value::Str(v.clone()),
            Value::Array(v) => Value::Array(v.clone()),
            Value::Primitive(v) => Value::Primitive(v.clone()),
            Value::List(v) => Value::List(v.clone()),
            Value::Map(v) => Value::Map(v.clone()),
            Value::Schema(v) => Value::Schema(v.clone()),
            Value::Field(v) => Value::Field(v.clone()),
            Value::Object(v) => Value::Object(v.clone()),
        }
    }
}

impl<IN> fmt::Debug for Value<'_, IN> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(s) => write!(f, "Str({s:?})"),
            Value::Array(v) => f.debug_tuple("Array").field(v).finish(),
            Value::List(v) => f.debug_tuple("List").field(v).finish(),
            Value::Primitive(v) => f.debug_tuple("Primitive").field(v).finish(),
            Value::Object(o) => o.fmt(f),
            other => f.write_str(&other.to_java_string()),
        }
    }
}

impl<'a, IN> Value<'a, IN> {
    /// An object array of the values.
    pub fn array<R: IntoValue<'a, IN>>(items: impl IntoIterator<Item = R>) -> Result<Value<'a, IN>> {
        Ok(Value::Array(items.into_iter().map(IntoValue::into_value).collect::<Result<_>>()?))
    }

    /// A collection of the values.
    pub fn list<R: IntoValue<'a, IN>>(items: impl IntoIterator<Item = R>) -> Result<Value<'a, IN>> {
        Ok(Value::List(items.into_iter().map(IntoValue::into_value).collect::<Result<_>>()?))
    }

    /// A map of the entries, in this order.
    pub fn map<K: IntoValue<'a, IN>, V: IntoValue<'a, IN>>(entries: impl IntoIterator<Item = (K, V)>) -> Result<Value<'a, IN>> {
        Ok(Value::Map(entries.into_iter().map(|(k, v)| Ok((k.into_value()?, v.into_value()?))).collect::<Result<_>>()?))
    }

    /// Any other object, printed with its `Display` (its `toString()`).
    pub fn object<T: fmt::Display + Any + Send + Sync>(value: T) -> Value<'a, IN> {
        Value::Object(Object::new(value))
    }

    /// Whether this is `null`.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// `instanceof Number`.
    pub fn is_number(&self) -> bool {
        matches!(
            self,
            Value::Byte(_)
                | Value::Short(_)
                | Value::Int(_)
                | Value::Long(_)
                | Value::Float(_)
                | Value::Double(_)
                | Value::BigInteger(_)
                | Value::BigDecimal(_)
        )
    }

    /// The text of a `String` value.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// A `Schema`, or a `CompositeField` (which is one): `instanceof Schema`.
    pub(crate) fn as_schema(&self) -> Option<Schema<'a, IN>> {
        match self {
            Value::Schema(s) => Some(s.clone()),
            Value::Field(f) if f.is_composite() => Some(f.as_schema()),
            _ => None,
        }
    }

    /// `getClass().getName()` (`None` for `null`). A collection reads as `java.util.ArrayList`, a map as
    /// `java.util.LinkedHashMap` and any other object as `java.lang.Object`: Java's runtime class is not kept.
    pub fn java_class_name(&self) -> Option<&'static str> {
        Some(match self {
            Value::Null => return None,
            Value::Bool(_) => "java.lang.Boolean",
            Value::Byte(_) => "java.lang.Byte",
            Value::Short(_) => "java.lang.Short",
            Value::Int(_) => "java.lang.Integer",
            Value::Long(_) => "java.lang.Long",
            Value::Float(_) => "java.lang.Float",
            Value::Double(_) => "java.lang.Double",
            Value::BigInteger(_) => "java.math.BigInteger",
            Value::BigDecimal(_) => "java.math.BigDecimal",
            Value::Char(_) => "java.lang.Character",
            Value::Str(_) => "java.lang.String",
            Value::Array(_) => "[Ljava.lang.Object;",
            Value::Primitive(p) => p.class_name(),
            Value::List(_) => "java.util.ArrayList",
            Value::Map(_) => "java.util.LinkedHashMap",
            Value::Schema(_) => "net.datafaker.transformations.Schema",
            Value::Field(f) if f.is_composite() => "net.datafaker.transformations.CompositeField",
            Value::Field(_) => "net.datafaker.transformations.SimpleField",
            Value::Object(_) => "java.lang.Object",
        })
    }

    /// The `ClassCastException` of casting this (not `null`) to `class`.
    pub(crate) fn class_cast_error(&self, class: &str) -> Error {
        class_cast_error(self.java_class_name().unwrap_or("java.lang.Object"), class)
    }

    /// `String.valueOf(value)`. Arrays and objects without their own `toString()` print an identity hash in Java,
    /// which no other run reproduces; here it reads `@0`.
    pub fn to_java_string(&self) -> String {
        match self {
            Value::Null => "null".into(),
            Value::Bool(v) => v.to_string(),
            Value::Byte(v) => v.to_string(),
            Value::Short(v) => v.to_string(),
            Value::Int(v) => v.to_string(),
            Value::Long(v) => v.to_string(),
            Value::Float(v) => crate::java::float_to_string(*v),
            Value::Double(v) => crate::java::double_to_string(*v),
            Value::BigInteger(v) | Value::BigDecimal(v) | Value::Str(v) => v.clone(),
            Value::Char(c) => c.to_string(),
            Value::Array(_) => "[Ljava.lang.Object;@0".into(),
            Value::Primitive(p) => format!("{}@0", p.class_name()),
            Value::List(items) => format!("[{}]", items.iter().map(Value::to_java_string).collect::<Vec<_>>().join(", ")),
            Value::Map(entries) => format!(
                "{{{}}}",
                entries.iter().map(|(k, v)| format!("{}={}", k.to_java_string(), v.to_java_string())).collect::<Vec<_>>().join(", ")
            ),
            Value::Schema(s) => format!("net.datafaker.transformations.Schema@{:x}", s.java_hash_code()),
            Value::Field(f) => format!(
                "net.datafaker.transformations.{}@{:x}",
                if f.is_composite() { "CompositeField" } else { "SimpleField" },
                f.java_hash_code()
            ),
            Value::Object(o) => o.text.clone(),
        }
    }
}

/// What a field function may return: anything that becomes a [`Value`], or a `Result` of one (an exception the
/// function threw).
pub trait IntoValue<'a, IN> {
    /// The value, or the error the function threw.
    fn into_value(self) -> Result<Value<'a, IN>>;
}

impl<'a, IN> IntoValue<'a, IN> for Value<'a, IN> {
    fn into_value(self) -> Result<Value<'a, IN>> {
        Ok(self)
    }
}

impl<'a, IN, T: IntoValue<'a, IN>> IntoValue<'a, IN> for Result<T> {
    fn into_value(self) -> Result<Value<'a, IN>> {
        self?.into_value()
    }
}

impl<'a, IN, T: IntoValue<'a, IN>> IntoValue<'a, IN> for Option<T> {
    fn into_value(self) -> Result<Value<'a, IN>> {
        match self {
            Some(v) => v.into_value(),
            None => Ok(Value::Null),
        }
    }
}

/// A `List`.
impl<'a, IN, T: IntoValue<'a, IN>> IntoValue<'a, IN> for Vec<T> {
    fn into_value(self) -> Result<Value<'a, IN>> {
        Value::list(self)
    }
}

impl<'a, IN> IntoValue<'a, IN> for Schema<'a, IN> {
    fn into_value(self) -> Result<Value<'a, IN>> {
        Ok(Value::Schema(self))
    }
}

impl<'a, IN> IntoValue<'a, IN> for Field<'a, IN> {
    fn into_value(self) -> Result<Value<'a, IN>> {
        Ok(Value::Field(self))
    }
}

impl<'a, IN> IntoValue<'a, IN> for PrimitiveArray {
    fn into_value(self) -> Result<Value<'a, IN>> {
        Ok(Value::Primitive(self))
    }
}

/// `null`, as a supplier that returns nothing.
impl<'a, IN> IntoValue<'a, IN> for () {
    fn into_value(self) -> Result<Value<'a, IN>> {
        Ok(Value::Null)
    }
}

impl<'a, IN> IntoValue<'a, IN> for &str {
    fn into_value(self) -> Result<Value<'a, IN>> {
        Ok(Value::Str(self.to_string()))
    }
}

impl<'a, IN> IntoValue<'a, IN> for &String {
    fn into_value(self) -> Result<Value<'a, IN>> {
        Ok(Value::Str(self.clone()))
    }
}

macro_rules! into_value {
    ($($t:ty => $v:ident),* $(,)?) => {
        $(impl<'a, IN> IntoValue<'a, IN> for $t {
            fn into_value(self) -> Result<Value<'a, IN>> {
                Ok(Value::$v(self))
            }
        })*
    };
}

into_value!(String => Str, bool => Bool, i8 => Byte, i16 => Short, i32 => Int, i64 => Long, f32 => Float, f64 => Double, char => Char);

impl<'a, IN> IntoValue<'a, IN> for BigDecimal {
    fn into_value(self) -> Result<Value<'a, IN>> {
        Ok(Value::BigDecimal(self.to_string()))
    }
}

macro_rules! into_object {
    ($($t:ty),*) => {
        $(impl<'a, IN> IntoValue<'a, IN> for $t {
            fn into_value(self) -> Result<Value<'a, IN>> {
                Ok(Value::object(self))
            }
        })*
    };
}

into_object!(LocalDate, LocalTime, LocalDateTime, Instant, Date, Duration, Period);

/// The `ClassCastException` of casting an object of class `from` to `to`, as HotSpot words it: JDK classes are in
/// module `java.base`, Datafaker's on the class path (the unnamed module of the application loader).
pub(crate) fn class_cast_error(from: &str, to: &str) -> Error {
    let module = |class: &str| {
        if class.starts_with("net.datafaker.") { "unnamed module of loader 'app'" } else { "module java.base of loader 'bootstrap'" }
    };
    let detail = if module(from) == module(to) {
        format!("{from} and {to} are in {}", module(from))
    } else {
        format!("{from} is in {}; {to} is in {}", module(from), module(to))
    };
    Error::new("ClassCastException", format!("class {from} cannot be cast to class {to} ({detail})"))
}
