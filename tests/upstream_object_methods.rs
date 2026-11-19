//! `ObjectMethodsTest`, Datafaker 2.7.0, `providers/base`: `ObjectMethods` over the test's own classes, declared as
//! `JavaClass`es with their return types (`Person.age()` and `Person.name()` return user objects, so they are
//! `OBJECTS`).

use std::sync::Arc;

use datafaker::Faker;
use datafaker::dispatch::{JValue, JavaClass, ObjectGetter, ObjectMethod};
use datafaker::object_methods::{Member, Returned, execute_method_by_return_type, get_method_by_name};

struct Person;

struct Age;

struct PersonName;

impl JavaClass for Person {
    const NAME: &'static str = "net.datafaker.providers.base.ObjectMethodsTest$Person";
    const PROVIDER: bool = false;
    const METHODS: &'static [ObjectMethod<Self>] = &[];
    const OBJECTS: &'static [ObjectGetter<Self>] = &[
        ObjectGetter::new("age", "Age", |_, _| Ok(Arc::new(Age))),
        ObjectGetter::new("name", "PersonName", |_, _| Ok(Arc::new(PersonName))),
    ];
}

impl JavaClass for Age {
    const NAME: &'static str = "net.datafaker.providers.base.ObjectMethodsTest$Age";
    const PROVIDER: bool = false;
    const METHODS: &'static [ObjectMethod<Self>] = &[ObjectMethod::object("value", &[], |_, _, _| Ok(JValue::Int(23))).returning("int")];
}

impl JavaClass for PersonName {
    const NAME: &'static str = "net.datafaker.providers.base.ObjectMethodsTest$PersonName";
    const PROVIDER: bool = false;
    const METHODS: &'static [ObjectMethod<Self>] = &[
        ObjectMethod::object("firstName", &[], |_, _, _| Ok(JValue::str("John"))).returning("String"),
        ObjectMethod::object("lastName", &[], |_, _, _| Ok(JValue::str("Smith"))).returning("CharSequence"),
        // Declared in this order so that `chooseFirstByAlphabet`, not the table order, picks `weight`.
        ObjectMethod::object("width", &[], |_, _, _| Ok(JValue::Float(60.0))).returning("float"),
        ObjectMethod::object("weight", &[], |_, _, _| Ok(JValue::Float(90.0))).returning("float"),
    ];
}

/// `Class.getMethod(name)` of the test: the table entry of that name.
fn class_method<T: JavaClass>(name: &str) -> Member<T> {
    T::OBJECTS
        .iter()
        .find(|o| o.name == name)
        .map(Member::Object)
        .or_else(|| T::METHODS.iter().find(|m| m.name == name).map(Member::Method))
        .expect("a method")
}

/// `methodByName`.
#[test]
fn method_by_name() {
    assert!(get_method_by_name::<Person>("age").unwrap().is(&class_method::<Person>("age")));
    assert!(get_method_by_name::<Person>("name").unwrap().is(&class_method::<Person>("name")));
    assert!(get_method_by_name::<PersonName>("firstName").unwrap().is(&class_method::<PersonName>("firstName")));
    assert!(get_method_by_name::<PersonName>("lastName").unwrap().is(&class_method::<PersonName>("lastName")));
    // Members compare by signature (not by address, which incremental compilation does not keep), so others differ.
    assert!(!get_method_by_name::<Person>("age").unwrap().is(&class_method::<Person>("name")));
    assert!(!get_method_by_name::<PersonName>("width").unwrap().is(&class_method::<PersonName>("weight")));
}

/// `methodByReturnType`.
#[test]
fn method_by_return_type() {
    let faker = Faker::new();
    let object = |r: Option<Returned>| match r {
        Some(Returned::Object(o)) => o,
        other => panic!("{other:?}"),
    };
    assert!(object(execute_method_by_return_type(&Person, &faker, "Age").unwrap()).downcast::<Age>().is_ok());
    assert!(object(execute_method_by_return_type(&Person, &faker, "PersonName").unwrap()).downcast::<PersonName>().is_ok());
    let value = |r: Option<Returned>| match r {
        Some(Returned::Value(v)) => v,
        other => panic!("{other:?}"),
    };
    assert_eq!(value(execute_method_by_return_type(&PersonName, &faker, "String").unwrap()), JValue::str("John"));
    assert_eq!(value(execute_method_by_return_type(&PersonName, &faker, "CharSequence").unwrap()), JValue::str("Smith"));
}

/// `multipleMethodsWithSameReturnType`.
#[test]
fn multiple_methods_with_same_return_type() {
    let faker = Faker::new();
    match execute_method_by_return_type(&PersonName, &faker, "float").unwrap() {
        Some(Returned::Value(v)) => assert_eq!(v, JValue::Float(90.0)),
        other => panic!("{other:?}"),
    }
}
