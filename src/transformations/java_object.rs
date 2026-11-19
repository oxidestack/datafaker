//! `JavaObjectTransformer`, adapted: Java finds a class's constructors and fields by reflection, here a
//! [`JavaObject`] implementation says how its type is built (Rust has no reflection over a class, so the type says how it is built).

use std::marker::PhantomData;

use super::{Input, Schema, Value, check_stream_limit};
use crate::{Error, Result};

/// A type [`JavaObjectTransformer`] can build: what reflection finds on a Java class.
///
/// A record, or a class without a public parameterless constructor, is built by [`JavaObject::construct`] from
/// the schema's values in field order (its canonical or first public constructor). A class with one starts from
/// [`JavaObject::new_instance`] and gets each field set by name ([`JavaObject::set_field`], its declared fields).
pub trait JavaObject: Sized {
    /// `getName()` of the class, for error messages.
    fn class_name() -> String {
        std::any::type_name::<Self>().to_string()
    }

    /// The public parameterless constructor, if the type is a class that has one.
    fn new_instance() -> Option<Self> {
        None
    }

    /// The names of the declared fields [`JavaObject::set_field`] sets.
    fn field_names() -> &'static [&'static str] {
        &[]
    }

    /// The canonical constructor of a record, or the first public constructor of a class without a parameterless
    /// one: called with the schema's values in order. A value of the wrong type or count fails (Java's
    /// `IllegalArgumentException: argument type mismatch`).
    fn construct(values: Vec<Value<'_, Self>>) -> Result<Self> {
        let _ = values;
        Err(Error::illegal_argument("argument type mismatch"))
    }

    /// `java.lang.reflect.Field.set(object, value)` for the field called `name`.
    fn set_field(&mut self, name: &str, value: Value<'_, Self>) -> Result<()> {
        let _ = value;
        Err(Error::illegal_argument(format!("Can not set field {}.{name}", Self::class_name())))
    }
}

/// `JavaObjectTransformer`: builds `T`s from a schema whose field names are its fields (or whose values are its
/// constructor's arguments).
pub struct JavaObjectTransformer<T> {
    source: bool,
    kind: PhantomData<fn() -> T>,
}

impl<T> Default for JavaObjectTransformer<T> {
    fn default() -> Self {
        JavaObjectTransformer { source: false, kind: PhantomData }
    }
}

impl<T: JavaObject> JavaObjectTransformer<T> {
    /// A transformer building `T`s.
    pub fn new() -> Self {
        JavaObjectTransformer::default()
    }

    /// `from(Class)`: `T` is the class [`Self::generate_stream`] builds (without it the stream is empty).
    pub fn from(mut self) -> Self {
        self.source = true;
        self
    }

    /// `apply(Class, schema)`: a new `T`.
    pub fn apply(&self, schema: &Schema<'_, T>) -> Result<T> {
        self.build(None, schema)
    }

    /// `apply(object, schema)`: a record (or constructor-built class) is built anew from the schema applied to
    /// `object`; any other class gets its fields set on `object` itself.
    pub fn apply_to(&self, object: T, schema: &Schema<'_, T>) -> Result<T> {
        self.build(Some(object), schema)
    }

    fn build(&self, object: Option<T>, schema: &Schema<'_, T>) -> Result<T> {
        let fields = schema.fields();
        let constructed = object.is_none().then(T::new_instance).flatten();
        let instance = match (object, constructed) {
            (Some(o), _) if T::new_instance().is_none() => {
                return self.construct(fields.iter().map(|f| f.transform(Some(&o))).collect::<Result<_>>()?);
            }
            (Some(o), _) => o,
            (None, Some(o)) => o,
            (None, None) => return self.construct(fields.iter().map(|f| f.transform(None)).collect::<Result<_>>()?),
        };
        let names = T::field_names();
        for f in fields {
            if !f.name().is_some_and(|n| names.contains(&n)) {
                return Err(Error::new(
                    "NullPointerException",
                    "Cannot invoke \"java.lang.reflect.Field.setAccessible(boolean)\" because \"rFields[i]\" is null",
                ));
            }
        }
        let mut instance = instance;
        for f in fields {
            let value = f.transform(Some(&instance))?;
            instance.set_field(f.name().expect("checked"), value)?;
        }
        Ok(instance)
    }

    fn construct(&self, values: Vec<Value<'_, T>>) -> Result<T> {
        T::construct(values).map_err(|e| Error::runtime(format!("Failed to instantiate {}", T::class_name())).with_cause(e))
    }

    /// `generate(input, schema)`: applies the schema to each element; a record's new object is dropped, so the
    /// elements come back as they were (upstream's behaviour), other classes' with their fields set.
    pub fn generate_from(&self, input: &(impl Input<T> + ?Sized), schema: &Schema<'_, T>) -> Result<Vec<Option<T>>> {
        if let Some(s) = input.as_sequence() && s.is_infinite() {
            return Err(Error::illegal_argument(format!("Should be finite size: {}", s.describe())));
        }
        let rows = input.rows().collect::<Result<Vec<_>>>()?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let Some(object) = row else {
                return Err(Error::new("NullPointerException", "Cannot invoke \"Object.getClass()\" because \"input\" is null"));
            };
            if T::new_instance().is_none() {
                let fields = schema.fields();
                self.construct(fields.iter().map(|f| f.transform(Some(&object))).collect::<Result<_>>()?)?;
                out.push(Some(object));
            } else {
                out.push(Some(self.build(Some(object), schema)?));
            }
        }
        Ok(out)
    }

    /// `generateStream(schema, limit)`: `limit` new `T`s, or none without [`Self::from`].
    pub fn generate_stream<'s>(&'s self, schema: &'s Schema<'_, T>, limit: i64) -> Result<Box<dyn Iterator<Item = Result<T>> + 's>> {
        if !self.source {
            return Ok(Box::new(std::iter::empty()));
        }
        check_stream_limit(limit)?;
        Ok(Box::new((0..limit).map(move |_| self.apply(schema))))
    }

    /// `generate(schema, limit)`.
    pub fn generate(&self, schema: &Schema<'_, T>, limit: i32) -> Result<Vec<T>> {
        self.generate_stream(schema, limit as i64)?.collect()
    }

    /// `writeToOutputStream`: not implemented upstream either.
    pub fn write_to_output_stream(&self, _out: &mut dyn std::io::Write, _schema: &Schema<'_, T>, _limit: i64) -> Result<()> {
        Err(Error::new("UnsupportedOperationException", "Not implemented yet"))
    }

    /// `getStartStream`.
    pub fn start_stream(&self, _schema: &Schema<'_, T>) -> Result<String> {
        Err(Error::new("UnsupportedOperationException", "null"))
    }

    /// `getEndStream`.
    pub fn end_stream(&self) -> Result<String> {
        Err(Error::new("UnsupportedOperationException", "null"))
    }
}
