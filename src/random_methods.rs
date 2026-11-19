//! `RandomService`'s public methods as expressions reach them (`#{random.nextInt '10'}`).

use crate::dispatch::{npe, JValue, Method, P};

pub(crate) static METHODS: &[Method] = &[
    Method::new("nextInt", &[], |f, _| Ok(JValue::Int(f.random().next_int()))),
    Method::new("nextInt", &[P::Int], |f, a| Ok(JValue::Int(f.random().next_int_bounded(a[0].as_int())?))),
    Method::new("nextInt", &[P::Int, P::Int], |f, a| Ok(JValue::Int(f.random().next_int_between(a[0].as_int(), a[1].as_int())?))),
    // A template reaches the `Range` overloads with `null` only.
    Method::new("nextInt", &[P::Opaque("Range")], |_, _| {
        Err(npe("Cannot invoke \"net.datafaker.service.Range.cast(java.util.function.Function)\" because \"range\" is null"))
    })
    .accepting_null(),
    Method::new("nextFloat", &[], |f, _| Ok(JValue::Float(f.random().next_float()))),
    Method::new("nextLong", &[], |f, _| Ok(JValue::Long(f.random().next_long()))),
    Method::new("nextLong", &[P::Long], |f, a| Ok(JValue::Long(f.random().next_long_bounded(a[0].as_long())?))),
    Method::new("nextLong", &[P::Opaque("Range")], |_, _| Err(npe("Cannot invoke \"net.datafaker.service.Range.from()\" because \"range\" is null")))
        .accepting_null(),
    Method::new("nextLong", &[P::Long, P::Long], |f, a| Ok(JValue::Long(f.random().next_long_between(a[0].as_long(), a[1].as_long())?))),
    Method::new("nextDouble", &[], |f, _| Ok(JValue::Double(f.random().next_double()))),
    Method::new("nextDouble", &[P::Double, P::Double], |f, a| {
        Ok(JValue::Double(f.random().next_double_between(a[0].as_double(), a[1].as_double())))
    }),
    Method::new("nextBoolean", &[], |f, _| Ok(JValue::Bool(f.random().next_boolean()))),
    Method::new("hex", &[], |f, _| Ok(JValue::Str(f.random().hex()))),
    Method::new("hex", &[P::Int], |f, a| Ok(JValue::Str(f.random().hex_with(a[0].as_int(), true)))),
    Method::new("hex", &[P::Int, P::Boolean], |f, a| Ok(JValue::Str(f.random().hex_with(a[0].as_int(), a[1].as_bool())))),
];
