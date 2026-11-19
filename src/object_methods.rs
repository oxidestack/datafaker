//! `net.datafaker.providers.base.ObjectMethods` over user classes ([`JavaClass`]): a method without parameters by
//! name, and the one whose return type has a given simple name, run (`chooseFirstByAlphabet` when several return
//! the same type). Upstream scans `getMethods()` by reflection; here a class's [`JavaClass::METHODS`] (with their
//! [`Method::returning`](crate::dispatch::Method::returning) types) and [`JavaClass::OBJECTS`] stand for them.

use std::any::Any;
use std::sync::Arc;

use crate::dispatch::{JValue, JavaClass, ObjectGetter, ObjectMethod};
use crate::{Error, Faker, Result};

/// `ObjectMethods.IGNORED_METHODS`.
const IGNORED_METHODS: [&str; 5] = ["equals", "hashCode", "toString", "Builder", "stream"];

/// A method `ObjectMethods` finds: one of [`JavaClass::METHODS`] or of [`JavaClass::OBJECTS`].
pub enum Member<T: 'static> {
    /// A method returning a Java value.
    Method(&'static ObjectMethod<T>),
    /// A method returning another user object.
    Object(&'static ObjectGetter<T>),
}

impl<T> Member<T> {
    /// The method's name.
    pub fn name(&self) -> &'static str {
        match self {
            Member::Method(m) => m.name,
            Member::Object(o) => o.name,
        }
    }

    /// The simple name of the method's return type.
    pub fn return_type(&self) -> &'static str {
        match self {
            Member::Method(m) => m.returns,
            Member::Object(o) => o.returns,
        }
    }

    /// Whether this is the same method of `T` as `other`, as `Method.equals` decides: by name, parameter types and
    /// return type. (Not by address: each use of a `JavaClass`'s table constant may promote its own copy, so two
    /// references to one entry can differ, as they do under incremental compilation.)
    pub fn is(&self, other: &Member<T>) -> bool {
        match (self, other) {
            (Member::Method(a), Member::Method(b)) => a.name == b.name && a.params == b.params && a.varargs == b.varargs && a.returns == b.returns,
            (Member::Object(a), Member::Object(b)) => a.name == b.name && a.returns == b.returns,
            _ => false,
        }
    }
}

impl<T> std::fmt::Debug for Member<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}() -> {}", self.name(), self.return_type())
    }
}

/// What a method returned: a Java value, or a user object (downcast it to its type).
#[derive(Debug, Clone)]
pub enum Returned {
    /// A Java value.
    Value(JValue),
    /// A user object.
    Object(Arc<dyn Any + Send + Sync>),
}

/// `ObjectMethods.isUseful`: no parameters, not `void`, not one of the ignored names.
fn useful<T: JavaClass>() -> impl Iterator<Item = Member<T>> {
    let methods = T::METHODS.iter().filter(|m| m.params.is_empty() && !m.varargs).map(Member::Method);
    let objects = T::OBJECTS.iter().map(Member::Object);
    methods.chain(objects).filter(|m| m.return_type() != "void" && !IGNORED_METHODS.contains(&m.name()))
}

/// `ObjectMethods.getMethodByName(object, methodName)`.
pub fn get_method_by_name<T: JavaClass>(method_name: &str) -> Option<Member<T>> {
    useful::<T>().find(|m| m.name() == method_name)
}

/// `ObjectMethods.getMethodByReturnType`: of the methods returning `return_type`, the first by name.
fn get_method_by_return_type<T: JavaClass>(return_type: &str) -> Option<Member<T>> {
    let utf16 = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    useful::<T>().filter(|m| m.return_type() == return_type).min_by(|a, b| utf16(a.name()).cmp(&utf16(b.name())))
}

/// `ObjectMethods.executeMethodByReturnType(object, returnTypeSimpleName)`: `None` when no method returns that type;
/// a failure of the method as `RuntimeException("Failed to call method <class>.<type>()")` with it as the cause.
pub fn execute_method_by_return_type<T: JavaClass>(object: &T, faker: &Faker, return_type: &str) -> Result<Option<Returned>> {
    let Some(member) = get_method_by_return_type::<T>(return_type) else { return Ok(None) };
    let result = match member {
        Member::Method(m) => (m.invoke)(object, faker, &[]).map(Returned::Value),
        Member::Object(o) => (o.get)(object, faker).map(Returned::Object),
    };
    result
        .map(Some)
        .map_err(|e| Error::runtime(format!("Failed to call method {}.{return_type}()", T::NAME)).with_cause(e))
}
