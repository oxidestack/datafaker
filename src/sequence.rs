//! `net.datafaker.sequence`: `FakeCollection` (a list of random length) and `FakeStream` (a stream, possibly
//! infinite) of values drawn from suppliers, with a share of `null`s.
//!
//! An element is `Some(value)` or `None` (Java's `null`, drawn with the null rate).

use std::marker::PhantomData;
use std::sync::Arc;

use crate::transformations::{Input, SequenceInfo};
use crate::{Error, Faker, RandomService, Result};

/// `Supplier<T>`.
pub type Supplier<'a, T> = Arc<dyn Fn() -> T + Send + Sync + 'a>;

/// `FakeCollection`'s default length when no maximum is set (or a negative one).
const FAKE_COLLECTION_DEFAULT_SIZE: i32 = 10;

/// What `FakeSequence` holds.
struct Sequence<'a, T> {
    suppliers: Vec<Supplier<'a, T>>,
    min_length: i32,
    max_length: i32,
    random: RandomService,
    null_rate: f64,
}

impl<T> Clone for Sequence<'_, T> {
    fn clone(&self) -> Self {
        Sequence {
            suppliers: self.suppliers.clone(),
            min_length: self.min_length,
            max_length: self.max_length,
            random: self.random.clone(),
            null_rate: self.null_rate,
        }
    }
}

impl<T> Sequence<'_, T> {
    /// `singleton()`: `null` with the null rate, else a value of a random supplier.
    fn singleton(&self) -> Result<Option<T>> {
        if self.null_rate == 0.0 || self.random.next_double() >= self.null_rate {
            let i = self.random.next_int_bounded(self.suppliers.len() as i32)?;
            return Ok(Some((self.suppliers[i as usize])()));
        }
        Ok(None)
    }

    fn size(&self) -> Result<i32> {
        self.random.next_int_between(self.min_length, self.max_length)
    }
}

/// `FakeCollection`: [`FakeCollection::get`] draws a length between the minimum and the maximum, then that many
/// elements.
pub struct FakeCollection<'a, T> {
    seq: Sequence<'a, T>,
}

impl<'a, T> FakeCollection<'a, T> {
    /// `new FakeCollection.Builder<T>()`: unseeded unless given a faker.
    pub fn builder() -> FakeCollectionBuilder<'a, T> {
        FakeSequenceBuilder::new()
    }

    /// `get()`: a new list.
    pub fn get(&self) -> Result<Vec<Option<T>>> {
        let size = self.seq.size()?;
        let mut result = Vec::with_capacity(size.max(0) as usize);
        while (result.len() as i64) < size as i64 {
            result.push(self.seq.singleton()?);
        }
        Ok(result)
    }

    /// `singleton()`.
    pub fn singleton(&self) -> Result<Option<T>> {
        self.seq.singleton()
    }

    /// `isInfinite()`: never.
    pub fn is_infinite(&self) -> bool {
        false
    }
}

/// `FakeStream`: [`FakeStream::get`] draws a length between the minimum and the maximum, or is infinite when no
/// maximum is set.
pub struct FakeStream<'a, T> {
    seq: Sequence<'a, T>,
}

impl<'a, T: 'a> FakeStream<'a, T> {
    /// `new FakeStream.Builder<T>()`: unseeded unless given a faker.
    pub fn builder() -> FakeStreamBuilder<'a, T> {
        FakeSequenceBuilder::new()
    }

    /// `get()`: a new stream; its elements are drawn as it is read (the length when this is called).
    pub fn get(&self) -> Result<Box<dyn Iterator<Item = Result<Option<T>>> + 'a>> {
        let seq = self.seq.clone();
        if self.is_infinite() {
            return Ok(Box::new(std::iter::repeat_with(move || seq.singleton())));
        }
        let size = seq.size()?;
        Ok(Box::new(std::iter::repeat_with(move || seq.singleton()).take(size.max(0) as usize)))
    }

    /// `singleton()`.
    pub fn singleton(&self) -> Result<Option<T>> {
        self.seq.singleton()
    }

    /// `isInfinite()`: when no maximum length is set.
    pub fn is_infinite(&self) -> bool {
        self.seq.max_length < 0
    }
}

impl<T> std::fmt::Display for FakeStream<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FakeStream{{minLength={}, maxLength={}, nullRate={}}}",
            self.seq.min_length,
            self.seq.max_length,
            crate::java::double_to_string(self.seq.null_rate)
        )
    }
}

impl<T> SequenceInfo for FakeCollection<'_, T> {
    fn is_infinite(&self) -> bool {
        false
    }

    /// `Object.toString()`; its identity hash reads `0`.
    fn describe(&self) -> String {
        "net.datafaker.sequence.FakeCollection@0".into()
    }
}

impl<'a, T: 'a> SequenceInfo for FakeStream<'a, T> {
    fn is_infinite(&self) -> bool {
        FakeStream::is_infinite(self)
    }

    fn describe(&self) -> String {
        self.to_string()
    }
}

impl<'a, T> Input<T> for FakeCollection<'a, T> {
    fn rows(&self) -> Box<dyn Iterator<Item = Result<Option<T>>> + '_> {
        match self.get() {
            Ok(items) => Box::new(items.into_iter().map(Ok)),
            Err(e) => Box::new(std::iter::once(Err(e))),
        }
    }

    fn as_sequence(&self) -> Option<&dyn SequenceInfo> {
        Some(self)
    }
}

impl<'a, T: 'a> Input<T> for FakeStream<'a, T> {
    fn rows(&self) -> Box<dyn Iterator<Item = Result<Option<T>>> + '_> {
        match self.get() {
            Ok(items) => items,
            Err(e) => Box::new(std::iter::once(Err(e))),
        }
    }

    fn as_sequence(&self) -> Option<&dyn SequenceInfo> {
        Some(self)
    }
}

/// Marks a [`FakeSequenceBuilder`] that builds a [`FakeCollection`].
pub struct CollectionKind;

/// Marks a [`FakeSequenceBuilder`] that builds a [`FakeStream`].
pub struct StreamKind;

/// `FakeCollection.Builder`.
pub type FakeCollectionBuilder<'a, T> = FakeSequenceBuilder<'a, T, CollectionKind>;

/// `FakeStream.Builder`.
pub type FakeStreamBuilder<'a, T> = FakeSequenceBuilder<'a, T, StreamKind>;

/// `FakeSequence.Builder`: suppliers, lengths (`-1` when not set), the null rate and the faker whose random
/// service draws.
pub struct FakeSequenceBuilder<'a, T, K> {
    suppliers: Vec<Supplier<'a, T>>,
    min_length: i32,
    max_length: i32,
    null_rate: f64,
    random: Option<RandomService>,
    kind: PhantomData<K>,
}

impl<'a, T, K> Default for FakeSequenceBuilder<'a, T, K> {
    fn default() -> Self {
        FakeSequenceBuilder::new()
    }
}

impl<'a, T, K> FakeSequenceBuilder<'a, T, K> {
    /// An empty builder: no suppliers, no length limits, no nulls.
    pub fn new() -> Self {
        FakeSequenceBuilder { suppliers: Vec::new(), min_length: -1, max_length: -1, null_rate: 0.0, random: None, kind: PhantomData }
    }

    /// `new Builder<>(suppliers)`.
    pub fn with_suppliers(suppliers: Vec<Supplier<'a, T>>) -> Self {
        FakeSequenceBuilder { suppliers, ..FakeSequenceBuilder::new() }
    }

    /// `faker(faker)`: draw with the faker's random service.
    pub fn faker(mut self, faker: &Faker) -> Self {
        self.random = Some(faker.random());
        self
    }

    /// `minLen(minLength)`.
    pub fn min_len(mut self, min_length: i32) -> Self {
        self.min_length = min_length;
        self
    }

    /// `maxLen(maxLength)`.
    pub fn max_len(mut self, max_length: i32) -> Self {
        self.max_length = max_length;
        self
    }

    /// `len(length)`.
    pub fn len(self, length: i32) -> Self {
        self.len_between(length, length)
    }

    /// `len(minLength, maxLength)`.
    pub fn len_between(mut self, min_length: i32, max_length: i32) -> Self {
        self.max_length = max_length;
        self.min_length = min_length;
        self
    }

    /// `nullRate(nullRate)`: the share of `null`s, between 0 and 1.
    pub fn null_rate(mut self, null_rate: f64) -> Result<Self> {
        if null_rate < 0.0 || null_rate > 1.0 {
            return Err(Error::illegal_argument(format!(
                "Null rate should be between 0 and 1 (received: {})",
                crate::java::double_to_string(null_rate)
            )));
        }
        self.null_rate = null_rate;
        Ok(self)
    }

    /// `suppliers(supplier)`, one at a time.
    pub fn supplier(mut self, supplier: impl Fn() -> T + Send + Sync + 'a) -> Self {
        self.suppliers.push(Arc::new(supplier));
        self
    }

    /// `suppliers(suppliers...)`.
    pub fn suppliers(mut self, suppliers: impl IntoIterator<Item = Supplier<'a, T>>) -> Self {
        self.suppliers.extend(suppliers);
        self
    }

    fn random_service(&self) -> RandomService {
        self.random.clone().unwrap_or_default()
    }

    fn length_error(&self) -> Error {
        Error::illegal_argument(format!(
            "Max length ({}) must be not less than min length ({}) and not negative",
            self.max_length, self.min_length
        ))
    }
}

impl<'a, T> FakeSequenceBuilder<'a, T, CollectionKind> {
    /// `build()`: ten elements unless a maximum is set.
    pub fn build(mut self) -> Result<FakeCollection<'a, T>> {
        if self.max_length < 0 {
            self.max_length = FAKE_COLLECTION_DEFAULT_SIZE;
        }
        if self.min_length > self.max_length {
            return Err(self.length_error());
        }
        let min_length = if self.min_length < 0 { self.max_length } else { self.min_length };
        let random = self.random_service();
        Ok(FakeCollection {
            seq: Sequence { suppliers: self.suppliers, min_length, max_length: self.max_length, random, null_rate: self.null_rate },
        })
    }

    /// `generate()`: `build().get()`.
    pub fn generate(self) -> Result<Vec<Option<T>>> {
        self.build()?.get()
    }
}

impl<'a, T: 'a> FakeSequenceBuilder<'a, T, StreamKind> {
    /// `build()`: infinite unless a maximum is set.
    pub fn build(self) -> Result<FakeStream<'a, T>> {
        if self.max_length >= 0 && self.min_length > self.max_length {
            return Err(self.length_error());
        }
        let min_length = if self.min_length < 0 { self.max_length } else { self.min_length };
        let random = self.random_service();
        Ok(FakeStream {
            seq: Sequence { suppliers: self.suppliers, min_length, max_length: self.max_length, random, null_rate: self.null_rate },
        })
    }

    /// `generate()`: `build().get()`.
    pub fn generate(self) -> Result<Box<dyn Iterator<Item = Result<Option<T>>> + 'a>> {
        self.build()?.get()
    }
}

impl Faker {
    /// `collection()`: a [`FakeCollection`] builder drawing with this faker's random service.
    pub fn collection<'a, T>(&self) -> FakeCollectionBuilder<'a, T> {
        FakeCollectionBuilder::new().faker(self)
    }

    /// `collection(suppliers)`.
    pub fn collection_of<'a, T>(&self, suppliers: Vec<Supplier<'a, T>>) -> FakeCollectionBuilder<'a, T> {
        FakeCollectionBuilder::with_suppliers(suppliers).faker(self)
    }

    /// `stream()`: a [`FakeStream`] builder drawing with this faker's random service.
    pub fn stream<'a, T>(&self) -> FakeStreamBuilder<'a, T> {
        FakeStreamBuilder::new().faker(self)
    }

    /// `stream(suppliers)`.
    pub fn stream_of<'a, T>(&self, suppliers: Vec<Supplier<'a, T>>) -> FakeStreamBuilder<'a, T> {
        FakeStreamBuilder::with_suppliers(suppliers).faker(self)
    }
}
