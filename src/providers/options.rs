//! `net.datafaker.providers.base.Options`.

use crate::dispatch::{npe, JValue, Method, P};
use crate::java;
use crate::{Error, Faker, Result};

/// Random choice among given options.
#[derive(Clone, Copy)]
pub struct Options<'a> {
    faker: &'a Faker,
}

impl<'a> Options<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Options { faker }
    }

    fn index(&self, len: usize) -> Result<usize> {
        Ok(self.faker.random().next_int_bounded(len as i32)? as usize)
    }

    /// `option(options...)`. An empty slice is `nextInt(0)`'s `IllegalArgumentException`.
    pub fn option<T: Clone>(&self, options: &[T]) -> Result<T> {
        Ok(options[self.index(options.len())?].clone())
    }

    /// `nextElement(array)` / `nextElement(list)`.
    pub fn next_element<T: Clone>(&self, items: &[T]) -> Result<T> {
        self.option(items)
    }

    /// `subset(size, options...)` for any element type: distinct options drawn without replacement, in draw order
    /// (upstream returns a `HashSet`, whose order depends on each type's `hashCode`; use [`Options::subset`] for
    /// strings to get that order).
    pub fn subset_of<T: Clone + PartialEq>(&self, size: i32, options: &[T]) -> Result<Vec<T>> {
        if size < 0 {
            return Err(Error::illegal_argument(format!("size should be not negative: {size}")));
        }
        let mut opts: Vec<T> = Vec::new();
        for o in options {
            if !opts.contains(o) {
                opts.push(o.clone());
            }
        }
        if size == 0 {
            return Ok(Vec::new());
        }
        if size as usize >= opts.len() {
            return Ok(opts);
        }
        let mut set = Vec::new();
        for _ in 0..size {
            let i = self.index(opts.len())?;
            set.push(opts.remove(i));
        }
        Ok(set)
    }

    /// `subset(size, options...)` for strings: distinct options drawn without replacement, in the order a
    /// `HashSet<String>` iterates them.
    pub fn subset(&self, size: i32, options: &[&str]) -> Result<Vec<String>> {
        let options: Vec<Option<&str>> = options.iter().copied().map(Some).collect();
        Ok(self.subset_nullable(size, &options)?.into_iter().map(Option::unwrap_or_default).collect())
    }

    /// [`Options::subset`] over options that may be `null` (`None`, whose hash is 0).
    pub fn subset_nullable(&self, size: i32, options: &[Option<&str>]) -> Result<Vec<Option<String>>> {
        if size < 0 {
            return Err(Error::illegal_argument(format!("size should be not negative: {size}")));
        }
        if size == 0 {
            return Ok(Vec::new());
        }
        let mut opts: Vec<Option<&str>> = Vec::new();
        for o in options {
            if !opts.contains(o) {
                opts.push(*o);
            }
        }
        let chosen: Vec<Option<&str>> = if size as usize >= opts.len() {
            opts
        } else {
            let mut set = Vec::new();
            for _ in 0..size {
                let i = self.index(opts.len())?;
                let o = opts.remove(i);
                if !set.contains(&o) {
                    set.push(o);
                }
            }
            set
        };
        let hashes: Vec<i32> = chosen.iter().map(|s| s.map_or(0, java::string_hash)).collect();
        Ok(java::hash_map_order(&hashes).into_iter().map(|i| chosen[i].map(str::to_string)).collect())
    }
}

impl Options<'_> {
    /// `subset(size, options...)` over any values: distinct options (by [`JValue`] equality, which is `equals` for
    /// the values a caller passes) drawn without replacement, in the order a `HashSet` iterates them by
    /// [`JValue::hash_code`].
    pub fn subset_values(&self, size: i32, options: &[JValue]) -> Result<Vec<JValue>> {
        if size < 0 {
            return Err(Error::illegal_argument(format!("size should be not negative: {size}")));
        }
        if size == 0 {
            return Ok(Vec::new());
        }
        let mut opts: Vec<JValue> = Vec::new();
        for o in options {
            if !opts.contains(o) {
                opts.push(o.clone());
            }
        }
        let chosen = if size as usize >= opts.len() {
            opts
        } else {
            let mut set = Vec::new();
            for _ in 0..size {
                let i = self.index(opts.len())?;
                set.push(opts.remove(i));
            }
            set
        };
        let hashes: Vec<i32> = chosen.iter().map(JValue::hash_code).collect();
        Ok(java::hash_map_order(&hashes).into_iter().map(|i| chosen[i].clone()).collect())
    }
}

/// A varargs `String` argument's elements, `None` for a null one.
fn items(list: &JValue) -> Vec<Option<&str>> {
    list.as_list().iter().map(|v| if *v == JValue::Null { None } else { Some(v.as_str()) }).collect()
}

fn nullable(value: Option<String>) -> JValue {
    value.map_or(JValue::Null, JValue::Str)
}

/// `option(E...)`, `option(char[])` and the other primitive arrays, `nextElement(E[])`, `nextElement(List)`: one of
/// the items, as the caller passed it (a caller's value comes back as its [`JValue::Foreign`]).
fn pick(f: &Faker, items: &JValue) -> Result<JValue> {
    f.options().option(items.as_list())
}

pub(crate) static METHODS: &[Method] = &[
    Method::varargs("option", &[P::Object], |f, a| pick(f, &a[0])).accepting_null_items(),
    Method::new("option", &[P::Opaque("char[]")], |f, a| pick(f, &a[0])),
    Method::varargs("option", &[P::String], |f, a| {
        let opts = items(crate::dispatch::non_null(a, 0, "Cannot read the array length because \"<parameter1>\" is null")?);
        Ok(nullable(f.options().option(&opts)?.map(str::to_string)))
    })
    .accepting_null_items(),
    // Reached only with a null class (`nextEnum(null)`).
    Method::new("option", &[P::Opaque("Class")], |_, _| Err(npe("Cannot invoke \"java.lang.Class.getEnumConstants()\" because \"klass\" is null")))
        .accepting_null(),
    Method::varargs("subset", &[P::Int, P::Object], |f, a| Ok(JValue::List(f.options().subset_values(a[0].as_int(), a[1].as_list())?)))
        .accepting_null_items(),
    Method::varargs("subset", &[P::Int, P::String], |f, a| {
        let opts = items(crate::dispatch::non_null(a, 1, "Cannot read the array length because \"<parameter1>\" is null")?);
        Ok(JValue::List(f.options().subset_nullable(a[0].as_int(), &opts)?.into_iter().map(nullable).collect()))
    })
    .accepting_null_items(),
    Method::new("nextElement", &[P::Opaque("Object[]")], |f, a| pick(f, &a[0])),
    Method::new("nextElement", &[P::Opaque("List")], |f, a| match &a[0] {
        JValue::Null => Err(npe("Cannot invoke \"java.util.List.size()\" because \"list\" is null")),
        list => pick(f, list),
    })
    .accepting_null(),
    Method::new("option", &[P::Opaque("int[]")], |f, a| pick(f, &a[0])),
    Method::new("option", &[P::Opaque("long[]")], |f, a| pick(f, &a[0])),
    Method::new("option", &[P::Opaque("float[]")], |f, a| pick(f, &a[0])),
    Method::new("option", &[P::Opaque("double[]")], |f, a| pick(f, &a[0])),
    Method::new("option", &[P::Opaque("short[]")], |f, a| pick(f, &a[0])),
    Method::new("option", &[P::Opaque("boolean[]")], |f, a| pick(f, &a[0])),
    Method::new("option", &[P::Opaque("byte[]")], |f, a| pick(f, &a[0])),
];
