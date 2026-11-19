//! The `List`, `Set`, `Map`, array and `Object...` parameters a Velocity template reaches with its own lists, maps
//! and Java arrays (`$faker.options().option(['a','b'])`), as MockServer 8.0.0's shaded Datafaker 2.7.0 answers them.
//! A caller's value travels as a [`JValue::Foreign`] and comes back as itself.

use datafaker::dispatch::{Target, P};
use datafaker::providers::ProviderKind;
use datafaker::{Faker, JValue};

type Answer = Result<JValue, (String, String)>;

fn call(faker: &Faker, target: Target, name: &str, params: &[P], args: &[JValue]) -> Answer {
    let method = target.methods().iter().find(|m| m.name == name && m.params == params).unwrap_or_else(|| panic!("{name}{params:?}"));
    (method.invoke)(faker, args).map_err(|e| (e.simple_class().to_string(), e.message().to_string()))
}

fn foreign(id: usize, hash: i32, class: &str) -> JValue {
    JValue::Foreign { id, hash, class: class.into() }
}

fn list(items: &[JValue]) -> JValue {
    JValue::List(items.to_vec())
}

fn str(s: &str) -> JValue {
    JValue::str(s)
}

fn err(class: &str, message: &str) -> Answer {
    Err((class.into(), message.into()))
}

const OPTIONS: Target = Target::Provider(ProviderKind::Options);

#[test]
fn option_and_next_element_return_the_item_as_passed() {
    let faker = Faker::new();
    let only = foreign(0, 7, "java.util.ArrayList");
    assert_eq!(call(&faker, OPTIONS, "option", &[P::Object], &[list(&[only.clone(), only.clone()])]), Ok(only.clone()));
    assert_eq!(call(&faker, OPTIONS, "nextElement", &[P::Opaque("List")], &[list(std::slice::from_ref(&only))]), Ok(only.clone()));
    assert_eq!(call(&faker, OPTIONS, "nextElement", &[P::Opaque("Object[]")], &[list(std::slice::from_ref(&only))]), Ok(only.clone()));
    assert_eq!(call(&faker, OPTIONS, "option", &[P::Opaque("char[]")], &[list(std::slice::from_ref(&only))]), Ok(only.clone()));
    assert_eq!(call(&faker, OPTIONS, "option", &[P::Opaque("byte[]")], &[list(std::slice::from_ref(&only))]), Ok(only));
    assert_eq!(call(&faker, OPTIONS, "option", &[P::Object], &[list(&[JValue::Null])]), Ok(JValue::Null));
}

#[test]
fn an_empty_list_is_next_ints_bound_error() {
    let faker = Faker::new();
    assert_eq!(call(&faker, OPTIONS, "nextElement", &[P::Opaque("List")], &[list(&[])]), err("IllegalArgumentException", "bound must be positive"));
    assert_eq!(
        call(&faker, OPTIONS, "nextElement", &[P::Opaque("List")], &[JValue::Null]),
        err("NullPointerException", "Cannot invoke \"java.util.List.size()\" because \"list\" is null")
    );
}

#[test]
fn subset_of_values_is_distinct_and_in_hash_set_order() {
    let faker = Faker::new();
    // subset(9, 'b', 2, true, 'b', 2, [1], {'k':'v'}, 3.5, 7) -> [[1], b, 2, 7, 3.5, {k=v}, true]
    let b = foreign(0, 98, "java.lang.String");
    let two = foreign(1, 2, "java.lang.Integer");
    let yes = foreign(2, 1231, "java.lang.Boolean");
    let one = foreign(3, 32, "java.util.ArrayList");
    let map = foreign(4, 107 ^ 118, "java.util.LinkedHashMap");
    let half = foreign(5, JValue::Double(3.5).hash_code(), "java.lang.Double");
    let seven = foreign(6, 7, "java.lang.Integer");
    let items = [b.clone(), two.clone(), yes.clone(), b.clone(), two.clone(), one.clone(), map.clone(), half.clone(), seven.clone()];
    assert_eq!(call(&faker, OPTIONS, "subset", &[P::Int, P::Object], &[JValue::Int(9), list(&items)]), Ok(list(&[one, b, two, seven, half, map, yes])));
    let five = foreign(0, 5, "java.lang.Integer");
    assert_eq!(
        call(&faker, OPTIONS, "subset", &[P::Int, P::Object], &[JValue::Int(3), list(&[five.clone(), JValue::Null, five.clone()])]),
        Ok(list(&[JValue::Null, five]))
    );
    assert_eq!(call(&faker, OPTIONS, "subset", &[P::Int, P::Object], &[JValue::Int(0), list(&[seven_like()])]), Ok(list(&[])));
    assert_eq!(
        call(&faker, OPTIONS, "subset", &[P::Int, P::Object], &[JValue::Int(-1), list(&[seven_like()])]),
        err("IllegalArgumentException", "size should be not negative: -1")
    );
}

fn seven_like() -> JValue {
    foreign(0, 7, "java.lang.Integer")
}

#[test]
fn java_hash_codes() {
    assert_eq!(JValue::Double(3.5).hash_code(), 0x400C0000);
    assert_eq!(JValue::Long(-1).hash_code(), 0);
    assert_eq!(JValue::Bool(false).hash_code(), 1237);
    assert_eq!(list(&[JValue::Int(1)]).hash_code(), 32);
    assert_eq!(JValue::str("b").hash_code(), 98);
}

#[test]
fn slug_joins_the_words_and_casts_each_to_string() {
    let faker = Faker::new();
    let internet = Target::Provider(ProviderKind::Internet);
    let slug = |words: JValue, glue: JValue| call(&faker, internet, "slug", &[P::Opaque("List"), P::String], &[words, glue]);
    assert_eq!(slug(list(&[str("a"), str("b")]), str("-")), Ok(str("a-b")));
    assert_eq!(slug(list(&[]), str("-")), Ok(str("")));
    assert_eq!(slug(list(&[str("a"), JValue::Null]), str("-")), Ok(str("a-null")));
    assert_eq!(slug(list(&[str("A B")]), JValue::Null), Ok(str("A B")));
    assert_eq!(
        slug(list(&[str("a"), foreign(0, 1, "java.lang.Integer")]), str("-")),
        err(
            "ClassCastException",
            "class java.lang.Integer cannot be cast to class java.lang.String (java.lang.Integer and java.lang.String are in module java.base of loader 'bootstrap')"
        )
    );
    assert_eq!(
        slug(list(&[foreign(0, 1, "[Ljava.lang.String;")]), str("-")),
        err(
            "ClassCastException",
            "class [Ljava.lang.String; cannot be cast to class java.lang.String ([Ljava.lang.String; and java.lang.String are in module java.base of loader 'bootstrap')"
        )
    );
    assert_eq!(
        slug(list(&[foreign(0, 1, "shaded_package.net.datafaker.Faker")]), str("-")),
        err(
            "ClassCastException",
            "class shaded_package.net.datafaker.Faker cannot be cast to class java.lang.String (shaded_package.net.datafaker.Faker is in unnamed \
             module of loader 'app'; java.lang.String is in module java.base of loader 'bootstrap')"
        )
    );
}

#[test]
fn twitter_text_inserts_the_keywords() {
    let faker = Faker::with_seed(5);
    let twitter = Target::Provider(ProviderKind::Twitter);
    let Ok(JValue::Str(text)) = call(&faker, twitter, "text", &[P::Opaque("String[]"), P::Int, P::Int], &[list(&[str("kw1"), str("kw2")]), JValue::Int(3), JValue::Int(5)])
    else {
        panic!()
    };
    let words: Vec<&str> = text.split(' ').collect();
    assert!(words.contains(&"kw1") && words.contains(&"kw2"), "{text}");
}

#[test]
fn templatify_replaces_character_keys_by_a_string_array_option() {
    let faker = Faker::new();
    let templatify = |letters: &str, map: JValue| call(&faker, Target::Faker, "templatify", &[P::String, P::Opaque("Map")], &[str(letters), map]);
    let entry = |value: JValue| JValue::Map(vec![(JValue::Char('a' as u16), value)]);
    assert_eq!(templatify("abca", entry(list(&[str("x"), str("x")]))), Ok(str("xbcx")));
    assert_eq!(templatify("bab", entry(list(&[str("")]))), Ok(str("bb")));
    assert_eq!(templatify("", entry(foreign(0, 0, "java.lang.String"))), Ok(str("")));
    assert_eq!(templatify("abc", JValue::Map(vec![(foreign(0, 97, "java.lang.String"), foreign(1, 120, "java.lang.String"))])), Ok(str("abc")));
    assert_eq!(templatify("abca", entry(JValue::Null)), err("NullPointerException", "Array with available options should be non null"));
    assert_eq!(
        templatify("abca", entry(foreign(0, 0, "java.util.ArrayList"))),
        err(
            "ClassCastException",
            "class java.util.ArrayList cannot be cast to class [Ljava.lang.String; (java.util.ArrayList and [Ljava.lang.String; are in module java.base of loader 'bootstrap')"
        )
    );
    assert_eq!(templatify("abca", entry(list(&[]))), err("IllegalArgumentException", "bound must be positive"));
}

#[test]
fn all_supported_locales_ignores_its_set() {
    let faker = Faker::new();
    let Ok(JValue::List(locales)) =
        call(&faker, Target::Provider(ProviderKind::Locality), "allSupportedLocales", &[P::Opaque("Set")], &[foreign(0, 0, "java.util.LinkedHashMap$LinkedKeySet")])
    else {
        panic!()
    };
    assert_eq!(locales.len(), 127);
}
