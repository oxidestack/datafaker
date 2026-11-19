//! `net.datafaker.service.RandomService` and `Range`.

use std::sync::Arc;

use crate::java::{Random, RandomGenerator};
use crate::{Error, Result};

const HEX_UP: &[u8; 16] = b"0123456789ABCDEF";
const HEX_LOWER: &[u8; 16] = b"0123456789abcdef";

/// Whether a [`Range`] bound is included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    /// The bound is in the range.
    Inclusive,
    /// The bound is not in the range.
    Exclusive,
}

/// `Range<Long>`: two bounds, each inclusive or exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// The lower bound.
    pub from: i64,
    /// Whether the lower bound is included.
    pub from_end: End,
    /// The upper bound.
    pub to: i64,
    /// Whether the upper bound is included.
    pub to_end: End,
}

impl Range {
    /// `Range.inclusive(from, to)`: `[from, to]`; an error when `from > to`.
    pub fn inclusive(from: i64, to: i64) -> Result<Range> {
        if from > to {
            return Err(Error::illegal_argument(format!("Lower bound ({from}) > upper bound ({to})")));
        }
        Ok(Range { from, from_end: End::Inclusive, to, to_end: End::Inclusive })
    }

    /// `Range.inclusiveExclusive(from, to)`: `[from, to)`; an error when `from >= to`.
    pub fn inclusive_exclusive(from: i64, to: i64) -> Result<Range> {
        if from >= to {
            return Err(Error::illegal_argument(format!("Lower bound ({from}) >= upper bound ({to})")));
        }
        Ok(Range { from, from_end: End::Inclusive, to, to_end: End::Exclusive })
    }

    /// `Range.exclusive(from, to)`: `(from, to)`; an error when the range holds no value.
    pub fn exclusive(from: i64, to: i64) -> Result<Range> {
        if to == i64::MIN {
            return Err(Error::illegal_argument(format!("Lower bound ({from}) >= upper bound ({to})")));
        }
        let upper = to - 1;
        if from >= upper {
            return Err(Error::illegal_argument(format!("Lower bound ({from}) >= upper bound-1 ({upper})")));
        }
        Ok(Range { from, from_end: End::Exclusive, to, to_end: End::Exclusive })
    }

    /// `Range.exclusiveInclusive(from, to)`: `(from, to]`; an error when `from >= to`.
    pub fn exclusive_inclusive(from: i64, to: i64) -> Result<Range> {
        if from >= to {
            return Err(Error::illegal_argument(format!("Lower bound ({from}) >= upper bound ({to})")));
        }
        Ok(Range { from, from_end: End::Exclusive, to, to_end: End::Inclusive })
    }
}

/// `RandomService`: Datafaker's facade over a `RandomGenerator` (a `java.util.Random` unless built with another).
#[derive(Debug, Clone)]
pub struct RandomService {
    random: Arc<dyn RandomGenerator>,
}

impl Default for RandomService {
    /// `new RandomService()`: the process-wide unseeded generator (`SHARED_RANDOM`), which every unseeded faker
    /// draws from.
    fn default() -> Self {
        static SHARED_RANDOM: std::sync::LazyLock<Arc<dyn RandomGenerator>> =
            std::sync::LazyLock::new(|| Arc::new(Random::from_entropy()));
        RandomService { random: SHARED_RANDOM.clone() }
    }
}

impl RandomService {
    /// `new RandomService(random)`: over a `java.util.Random`, a `SplittableRandom` or any other generator.
    pub fn new(random: impl RandomGenerator + 'static) -> Self {
        RandomService { random: Arc::new(random) }
    }

    /// `new RandomService(random)` over a generator shared with other owners.
    pub fn with_generator(random: Arc<dyn RandomGenerator>) -> Self {
        RandomService { random }
    }

    /// `new RandomService(new Random(seed))`.
    pub fn with_seed(seed: i64) -> Self {
        RandomService::new(Random::new(seed))
    }

    /// The underlying generator (`getRandomInternal()`); `as_random()` casts it to a `java.util.Random`.
    pub fn random(&self) -> &dyn RandomGenerator {
        &*self.random
    }

    /// `nextInt()`.
    pub fn next_int(&self) -> i32 {
        self.random.next_int()
    }

    /// `nextInt(maxExclusive)`.
    pub fn next_int_bounded(&self, max_exclusive: i32) -> Result<i32> {
        self.random.next_int_bounded(max_exclusive)
    }

    /// `nextInt(minInclusive, maxInclusive)`.
    pub fn next_int_between(&self, min_inclusive: i32, max_inclusive: i32) -> Result<i32> {
        if min_inclusive > max_inclusive {
            return Err(Error::illegal_argument(format!("Min ({min_inclusive}) > Max ({max_inclusive})")));
        }
        if max_inclusive.wrapping_add(1) < 0 {
            return Ok(self.next_long_between(min_inclusive as i64, max_inclusive as i64)? as i32);
        }
        self.random.next_int_range(min_inclusive, max_inclusive + 1)
    }

    /// `nextInt(Range<Integer>)`.
    pub fn next_int_range(&self, range: Range) -> Result<i32> {
        Ok(self.next_long_range(range)? as i32)
    }

    /// `nextFloat()`.
    pub fn next_float(&self) -> f32 {
        self.random.next_float()
    }

    /// `nextLong()`.
    pub fn next_long(&self) -> i64 {
        self.random.next_long()
    }

    /// `nextLong(maxExclusive)`.
    pub fn next_long_bounded(&self, max_exclusive: i64) -> Result<i64> {
        if max_exclusive <= 0 {
            return Err(Error::illegal_argument(format!("bound must be positive: {max_exclusive}")));
        }
        self.next_long_between(0, max_exclusive)
    }

    /// `nextLong(Range<Long>)`.
    pub fn next_long_range(&self, range: Range) -> Result<i64> {
        let plus_one = |v: i64| if v == i64::MAX { v } else { v + 1 };
        let from = match range.from_end {
            End::Exclusive => plus_one(range.from),
            End::Inclusive => range.from,
        };
        let to = match range.to_end {
            End::Exclusive => range.to,
            End::Inclusive => plus_one(range.to),
        };
        self.random.next_long_range(from, to)
    }

    /// `nextLong(min, max)`: `min` when they are equal, otherwise `max` is exclusive.
    pub fn next_long_between(&self, min: i64, max: i64) -> Result<i64> {
        if min == max { Ok(min) } else { self.random.next_long_range(min, max) }
    }

    /// `nextDouble()`.
    pub fn next_double(&self) -> f64 {
        self.random.next_double()
    }

    /// `nextDouble(min, max)`.
    pub fn next_double_between(&self, min: f64, max: f64) -> f64 {
        min + (self.next_double() * (max - min))
    }

    /// `nextBoolean()`.
    pub fn next_boolean(&self) -> bool {
        self.random.next_boolean()
    }

    /// `nextRandomBytes(n)`.
    pub fn next_random_bytes(&self, number_of_bytes: usize) -> Vec<i8> {
        let mut bytes = vec![0i8; number_of_bytes];
        self.random.next_bytes(&mut bytes);
        bytes
    }

    /// `nextEnum(klass)`: an index into the enum's constants.
    pub fn next_enum(&self, constants: usize) -> Result<usize> {
        Ok(self.next_int_bounded(constants as i32)? as usize)
    }

    /// `hex()`: eight upper-case hex digits.
    pub fn hex(&self) -> String {
        self.hex_with(8, true)
    }

    /// `hex(length, upper)`.
    pub fn hex_with(&self, length: i32, upper: bool) -> String {
        if length <= 0 {
            return String::new();
        }
        let table = if upper { HEX_UP } else { HEX_LOWER };
        let bytes = self.next_random_bytes(length as usize);
        // `((char) b) % 16`: a negative byte widens to 0xFFxx, whose low nibble is the byte's.
        bytes.iter().map(|b| table[(*b as u8 & 0x0F) as usize] as char).collect()
    }
}
