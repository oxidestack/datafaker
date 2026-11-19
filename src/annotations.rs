//! `net.datafaker.annotations`: `@FakeForSchema` and `FakeResolver`, adapted. Java reads the annotation and calls
//! the static method it names by reflection; here the type implements [`FakeForSchema`], whose
//! [`FakeForSchema::default_schema`] is that method (Rust has no reflection over a class, so the type says how it is built).

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, LazyLock, Mutex};

use crate::transformations::{JavaObject, JavaObjectTransformer, Schema};
use crate::{Faker, Result};

/// `@FakeForSchema("path#method")`: the schema a type is populated with by default.
pub trait FakeForSchema: JavaObject + 'static {
    /// The method the annotation names. Called once per type: like upstream, the schema is cached and its fields
    /// keep drawing from the generators they captured.
    fn default_schema() -> Schema<'static, Self>;
}

/// `FakeResolver.DEFAULT_SCHEMA_CACHE`: each type's default schema, an `Arc<Schema<'static, T>>`.
static DEFAULT_SCHEMA_CACHE: LazyLock<Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// `FakeResolver<T>`: populates `T`s.
pub struct FakeResolver<T> {
    kind: PhantomData<fn() -> T>,
}

impl<T: JavaObject> FakeResolver<T> {
    /// `FakeResolver.of(clazz)`.
    pub fn of() -> Self {
        FakeResolver { kind: PhantomData }
    }

    /// `generate(schema)`.
    pub fn generate(&self, schema: &Schema<'_, T>) -> Result<T> {
        JavaObjectTransformer::<T>::new().apply(schema)
    }
}

impl<T: FakeForSchema> FakeResolver<T> {
    /// `generate(null)`: with the type's default schema.
    pub fn generate_default(&self) -> Result<T> {
        let id = TypeId::of::<T>();
        let cached = DEFAULT_SCHEMA_CACHE.lock().expect("schema cache").get(&id).cloned();
        let entry = match cached {
            Some(entry) => entry,
            None => {
                let schema: Arc<dyn Any + Send + Sync> = Arc::new(T::default_schema());
                DEFAULT_SCHEMA_CACHE.lock().expect("schema cache").entry(id).or_insert(schema).clone()
            }
        };
        let schema = entry.downcast_ref::<Schema<'static, T>>().expect("one schema type per type");
        JavaObjectTransformer::<T>::new().apply(schema)
    }
}

impl Faker {
    /// `Faker.populate(clazz)`: a `T` from its `@FakeForSchema` schema.
    pub fn populate<T: FakeForSchema>() -> Result<T> {
        FakeResolver::<T>::of().generate_default()
    }

    /// `Faker.populate(clazz, schema)`.
    pub fn populate_with<T: JavaObject>(schema: &Schema<'_, T>) -> Result<T> {
        FakeResolver::<T>::of().generate(schema)
    }
}
