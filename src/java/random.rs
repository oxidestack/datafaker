//! `java.util.Random`, bit for bit, with the `java.util.random.RandomGenerator` defaults it inherits (JDK 17+):
//! `nextInt(origin, bound)`, `nextLong(bound)` and `nextLong(origin, bound)` go through `RandomSupport`'s bounded
//! algorithms, which draw differently from `nextInt(bound)`.
//!
//! A subclass of `java.util.Random` (`new Faker(new MyRandom())`) is a [`Random`] with [`RandomOverrides`]: every
//! method asks the overrides first, including the calls `Random` makes to itself (`nextLong(origin, bound)` draws
//! through an overridden `nextLong()`), as virtual dispatch does in Java.

use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI64, Ordering};

use crate::Error;

const MULTIPLIER: i64 = 0x5_DEEC_E66D;
const ADDEND: i64 = 0xB;
const MASK: i64 = (1 << 48) - 1;
const DOUBLE_UNIT: f64 = 1.0 / (1u64 << 53) as f64;

/// `RandomSupport.BAD_BOUND`.
const BAD_BOUND: &str = "bound must be positive";
/// `RandomSupport.BAD_RANGE`.
const BAD_RANGE: &str = "bound must be greater than origin";

/// The methods a subclass of `java.util.Random` overrides. Each returns `None` where the subclass keeps
/// `Random`'s own method; an override that throws in Java returns its `Err` (or panics where the Java method
/// returns a primitive that cannot carry one, such as `nextDouble()`). `random` is the object itself, so an
/// override can draw from the other methods (but not call its own super method).
#[allow(unused_variables)]
pub trait RandomOverrides: Debug + Send + Sync {
    /// `next(bits)`.
    fn next(&self, random: &Random, bits: u32) -> Option<i32> {
        None
    }
    /// `nextInt()`.
    fn next_int(&self, random: &Random) -> Option<i32> {
        None
    }
    /// `nextInt(bound)`.
    fn next_int_bounded(&self, random: &Random, bound: i32) -> Option<Result<i32, Error>> {
        None
    }
    /// `nextInt(origin, bound)`.
    fn next_int_range(&self, random: &Random, origin: i32, bound: i32) -> Option<Result<i32, Error>> {
        None
    }
    /// `nextLong()`.
    fn next_long(&self, random: &Random) -> Option<i64> {
        None
    }
    /// `nextLong(bound)`.
    fn next_long_bounded(&self, random: &Random, bound: i64) -> Option<Result<i64, Error>> {
        None
    }
    /// `nextLong(origin, bound)`.
    fn next_long_range(&self, random: &Random, origin: i64, bound: i64) -> Option<Result<i64, Error>> {
        None
    }
    /// `nextBoolean()`.
    fn next_boolean(&self, random: &Random) -> Option<bool> {
        None
    }
    /// `nextFloat()`.
    fn next_float(&self, random: &Random) -> Option<f32> {
        None
    }
    /// `nextDouble()`.
    fn next_double(&self, random: &Random) -> Option<f64> {
        None
    }
    /// `nextGaussian()`.
    fn next_gaussian(&self, random: &Random) -> Option<f64> {
        None
    }
}

/// `java.util.Random`. Thread-safe as the JDK's is: the seed is advanced with a compare-and-set.
#[derive(Debug)]
pub struct Random {
    seed: AtomicI64,
    next_next_gaussian: Mutex<Option<f64>>,
    overrides: Option<Arc<dyn RandomOverrides>>,
}

impl Clone for Random {
    fn clone(&self) -> Self {
        Random {
            seed: AtomicI64::new(self.seed.load(Ordering::SeqCst)),
            next_next_gaussian: Mutex::new(*self.next_next_gaussian.lock().expect("gaussian lock")),
            overrides: self.overrides.clone(),
        }
    }
}

impl Random {
    /// `new Random(seed)`.
    pub fn new(seed: i64) -> Self {
        Random { seed: AtomicI64::new((seed ^ MULTIPLIER) & MASK), next_next_gaussian: Mutex::new(None), overrides: None }
    }

    /// A subclass instance: this generator's state with `overrides` in front of its methods.
    pub fn with_overrides(self, overrides: Arc<dyn RandomOverrides>) -> Self {
        Random { overrides: Some(overrides), ..self }
    }

    fn overridden<T>(&self, f: impl FnOnce(&dyn RandomOverrides) -> Option<T>) -> Option<T> {
        self.overrides.as_deref().and_then(f)
    }

    /// `new Random()`: seeded from the clock and a per-process counter, as the JDK's `seedUniquifier() ^ nanoTime()`
    /// (the values differ from a JVM's, which never matters for an unseeded generator).
    pub fn from_entropy() -> Self {
        use std::sync::atomic::AtomicI64 as Counter;
        static UNIQUIFIER: Counter = Counter::new(8_682_522_807_148_012);
        // The JDK's compareAndSet loop: `fetch_update` is deprecated on current toolchains for `try_update`, which
        // older ones do not have.
        let mut u = UNIQUIFIER.load(Ordering::SeqCst);
        while let Err(actual) =
            UNIQUIFIER.compare_exchange_weak(u, u.wrapping_mul(1_181_783_497_276_652_981), Ordering::SeqCst, Ordering::SeqCst)
        {
            u = actual;
        }
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as i64).unwrap_or(0);
        Random::new(u.wrapping_mul(1_181_783_497_276_652_981) ^ nanos)
    }

    /// `setSeed(seed)`.
    pub fn set_seed(&self, seed: i64) {
        self.seed.store((seed ^ MULTIPLIER) & MASK, Ordering::SeqCst);
        *self.next_next_gaussian.lock().expect("gaussian lock") = None;
    }

    /// `next(bits)`.
    pub fn next(&self, bits: u32) -> i32 {
        if let Some(v) = self.overridden(|o| o.next(self, bits)) {
            return v;
        }
        let mut old = self.seed.load(Ordering::SeqCst);
        loop {
            let next = old.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND) & MASK;
            match self.seed.compare_exchange(old, next, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => return ((next as u64) >> (48 - bits)) as i32,
                Err(actual) => old = actual,
            }
        }
    }

    /// `nextInt()`.
    pub fn next_int(&self) -> i32 {
        if let Some(v) = self.overridden(|o| o.next_int(self)) {
            return v;
        }
        self.next(32)
    }

    /// `nextInt(bound)`: `IllegalArgumentException` unless `bound > 0`.
    pub fn next_int_bounded(&self, bound: i32) -> Result<i32, Error> {
        if let Some(v) = self.overridden(|o| o.next_int_bounded(self, bound)) {
            return v;
        }
        if bound <= 0 {
            return Err(Error::illegal_argument(BAD_BOUND));
        }
        let mut r = self.next(31);
        let m = bound - 1;
        if bound & m == 0 {
            return Ok(((bound as i64 * r as i64) >> 31) as i32);
        }
        let mut u = r;
        loop {
            r = u % bound;
            if u.wrapping_sub(r).wrapping_add(m) >= 0 {
                return Ok(r);
            }
            u = self.next(31);
        }
    }

    /// `nextInt(origin, bound)`: `RandomSupport.boundedNextInt`.
    pub fn next_int_range(&self, origin: i32, bound: i32) -> Result<i32, Error> {
        if let Some(v) = self.overridden(|o| o.next_int_range(self, origin, bound)) {
            return v;
        }
        bounded_next_int_range(self, origin, bound)
    }

    /// `nextLong()`.
    pub fn next_long(&self) -> i64 {
        if let Some(v) = self.overridden(|o| o.next_long(self)) {
            return v;
        }
        ((self.next(32) as i64) << 32).wrapping_add(self.next(32) as i64)
    }

    /// `nextLong(bound)`: `RandomSupport.boundedNextLong(rng, bound)`.
    pub fn next_long_bounded(&self, bound: i64) -> Result<i64, Error> {
        if let Some(v) = self.overridden(|o| o.next_long_bounded(self, bound)) {
            return v;
        }
        bounded_next_long(self, bound)
    }

    /// `nextLong(origin, bound)`: `RandomSupport.boundedNextLong(rng, origin, bound)`.
    pub fn next_long_range(&self, origin: i64, bound: i64) -> Result<i64, Error> {
        if let Some(v) = self.overridden(|o| o.next_long_range(self, origin, bound)) {
            return v;
        }
        bounded_next_long_range(self, origin, bound)
    }

    /// `nextBoolean()`.
    pub fn next_boolean(&self) -> bool {
        if let Some(v) = self.overridden(|o| o.next_boolean(self)) {
            return v;
        }
        self.next(1) != 0
    }

    /// `nextFloat()`.
    pub fn next_float(&self) -> f32 {
        if let Some(v) = self.overridden(|o| o.next_float(self)) {
            return v;
        }
        self.next(24) as f32 / (1 << 24) as f32
    }

    /// `nextDouble()`.
    pub fn next_double(&self) -> f64 {
        if let Some(v) = self.overridden(|o| o.next_double(self)) {
            return v;
        }
        (((self.next(26) as i64) << 27) + self.next(27) as i64) as f64 * DOUBLE_UNIT
    }

    /// `nextBytes(bytes)`.
    pub fn next_bytes(&self, bytes: &mut [i8]) {
        let mut i = 0;
        let len = bytes.len();
        while i < len {
            let mut rnd = self.next_int();
            let mut n = (len - i).min(4);
            while n > 0 {
                bytes[i] = rnd as i8;
                i += 1;
                rnd >>= 8;
                n -= 1;
            }
        }
    }

    /// `nextGaussian()`: the polar method with `StrictMath` (fdlibm) `log` and `sqrt`.
    pub fn next_gaussian(&self) -> f64 {
        if let Some(v) = self.overridden(|o| o.next_gaussian(self)) {
            return v;
        }
        let mut saved = self.next_next_gaussian.lock().expect("gaussian lock");
        if let Some(g) = saved.take() {
            return g;
        }
        loop {
            let v1 = 2.0 * self.next_double() - 1.0;
            let v2 = 2.0 * self.next_double() - 1.0;
            let s = v1 * v1 + v2 * v2;
            if s < 1.0 && s != 0.0 {
                let multiplier = (-2.0 * super::fdlibm::log(s) / s).sqrt();
                *saved = Some(v2 * multiplier);
                return v1 * multiplier;
            }
        }
    }
}

/// `java.util.random.RandomGenerator`: what `RandomService`, RgxGen and `Locality` draw from. Implementors supply
/// `nextInt()` and `nextLong()`; the other methods are the interface's defaults (`RandomSupport`'s bounded
/// algorithms), which `java.util.Random` overrides where it has its own (`nextInt(bound)`, `nextBoolean`,
/// `nextFloat`, `nextDouble`, `nextBytes`). `nextGaussian` and the streams are not part of it here: nothing in
/// Datafaker draws them from a `RandomGenerator` (`Random` keeps its own `next_gaussian`).
pub trait RandomGenerator: Debug + Send + Sync {
    /// `nextInt()`.
    fn next_int(&self) -> i32;

    /// `nextLong()`.
    fn next_long(&self) -> i64;

    /// `nextInt(bound)`: `IllegalArgumentException` unless `bound > 0`.
    fn next_int_bounded(&self, bound: i32) -> Result<i32, Error> {
        bounded_next_int(self, bound)
    }

    /// `nextInt(origin, bound)`.
    fn next_int_range(&self, origin: i32, bound: i32) -> Result<i32, Error> {
        bounded_next_int_range(self, origin, bound)
    }

    /// `nextLong(bound)`.
    fn next_long_bounded(&self, bound: i64) -> Result<i64, Error> {
        bounded_next_long(self, bound)
    }

    /// `nextLong(origin, bound)`.
    fn next_long_range(&self, origin: i64, bound: i64) -> Result<i64, Error> {
        bounded_next_long_range(self, origin, bound)
    }

    /// `nextBoolean()`: `nextInt() < 0`.
    fn next_boolean(&self) -> bool {
        self.next_int() < 0
    }

    /// `nextFloat()`: the top 24 bits of `nextInt()`.
    fn next_float(&self) -> f32 {
        ((self.next_int() as u32) >> 8) as f32 * (1.0 / (1u32 << 24) as f32)
    }

    /// `nextDouble()`: the top 53 bits of `nextLong()`.
    fn next_double(&self) -> f64 {
        ((self.next_long() as u64) >> 11) as f64 * DOUBLE_UNIT
    }

    /// `nextBytes(bytes)`: eight bytes per `nextLong()`, low byte first, the rest from one more.
    fn next_bytes(&self, bytes: &mut [i8]) {
        for chunk in bytes.chunks_mut(8) {
            let mut rnd = self.next_long();
            for b in chunk {
                *b = rnd as i8;
                rnd = ((rnd as u64) >> 8) as i64;
            }
        }
    }

    /// `(Random) generator`: this generator as a `java.util.Random`, if it is one.
    fn as_random(&self) -> Option<&Random> {
        None
    }
}

impl RandomGenerator for Random {
    fn next_int(&self) -> i32 {
        Random::next_int(self)
    }

    fn next_long(&self) -> i64 {
        Random::next_long(self)
    }

    fn next_int_bounded(&self, bound: i32) -> Result<i32, Error> {
        Random::next_int_bounded(self, bound)
    }

    fn next_int_range(&self, origin: i32, bound: i32) -> Result<i32, Error> {
        Random::next_int_range(self, origin, bound)
    }

    fn next_long_bounded(&self, bound: i64) -> Result<i64, Error> {
        Random::next_long_bounded(self, bound)
    }

    fn next_long_range(&self, origin: i64, bound: i64) -> Result<i64, Error> {
        Random::next_long_range(self, origin, bound)
    }

    fn next_boolean(&self) -> bool {
        Random::next_boolean(self)
    }

    fn next_float(&self) -> f32 {
        Random::next_float(self)
    }

    fn next_double(&self) -> f64 {
        Random::next_double(self)
    }

    fn next_bytes(&self, bytes: &mut [i8]) {
        Random::next_bytes(self, bytes)
    }

    fn as_random(&self) -> Option<&Random> {
        Some(self)
    }
}

/// `RandomSupport.boundedNextInt(rng, bound)` after `checkBound`.
fn bounded_next_int<R: RandomGenerator + ?Sized>(rng: &R, bound: i32) -> Result<i32, Error> {
    if bound <= 0 {
        return Err(Error::illegal_argument(BAD_BOUND));
    }
    let m = bound - 1;
    let mut r = rng.next_int();
    if bound & m == 0 {
        r &= m;
    } else {
        let mut u = ((r as u32) >> 1) as i32;
        loop {
            r = u % bound;
            if u.wrapping_add(m).wrapping_sub(r) >= 0 {
                break;
            }
            u = ((rng.next_int() as u32) >> 1) as i32;
        }
    }
    Ok(r)
}

/// `RandomSupport.boundedNextInt(rng, origin, bound)` after `checkRange`.
fn bounded_next_int_range<R: RandomGenerator + ?Sized>(rng: &R, origin: i32, bound: i32) -> Result<i32, Error> {
    if origin >= bound {
        return Err(Error::illegal_argument(BAD_RANGE));
    }
    let mut r = rng.next_int();
    let n = bound.wrapping_sub(origin);
    let m = n.wrapping_sub(1);
    if n & m == 0 {
        r = (r & m).wrapping_add(origin);
    } else if n > 0 {
        let mut u = ((r as u32) >> 1) as i32;
        loop {
            r = u % n;
            if u.wrapping_add(m).wrapping_sub(r) >= 0 {
                break;
            }
            u = ((rng.next_int() as u32) >> 1) as i32;
        }
        r = r.wrapping_add(origin);
    } else {
        while r < origin || r >= bound {
            r = rng.next_int();
        }
    }
    Ok(r)
}

/// `RandomSupport.boundedNextLong(rng, bound)` after `checkBound`.
fn bounded_next_long<R: RandomGenerator + ?Sized>(rng: &R, bound: i64) -> Result<i64, Error> {
    if bound <= 0 {
        return Err(Error::illegal_argument(BAD_BOUND));
    }
    let m = bound - 1;
    let mut r = rng.next_long();
    if bound & m == 0 {
        r &= m;
    } else {
        let mut u = ((r as u64) >> 1) as i64;
        loop {
            r = u % bound;
            if u.wrapping_add(m).wrapping_sub(r) >= 0 {
                break;
            }
            u = ((rng.next_long() as u64) >> 1) as i64;
        }
    }
    Ok(r)
}

/// `RandomSupport.boundedNextLong(rng, origin, bound)` after `checkRange`.
fn bounded_next_long_range<R: RandomGenerator + ?Sized>(rng: &R, origin: i64, bound: i64) -> Result<i64, Error> {
    if origin >= bound {
        return Err(Error::illegal_argument(BAD_RANGE));
    }
    let mut r = rng.next_long();
    let n = bound.wrapping_sub(origin);
    let m = n.wrapping_sub(1);
    if n & m == 0 {
        r = (r & m).wrapping_add(origin);
    } else if n > 0 {
        let mut u = ((r as u64) >> 1) as i64;
        loop {
            r = u % n;
            if u.wrapping_add(m).wrapping_sub(r) >= 0 {
                break;
            }
            u = ((rng.next_long() as u64) >> 1) as i64;
        }
        r = r.wrapping_add(origin);
    } else {
        while r < origin || r >= bound {
            r = rng.next_long();
        }
    }
    Ok(r)
}

/// `java.util.SplittableRandom`, bit for bit: SplitMix64 over a seed advanced by an odd gamma. Thread-safe here
/// (the JDK's is not): the seed advances with an atomic add, which is all `nextSeed()` does.
#[derive(Debug)]
pub struct SplittableRandom {
    seed: AtomicI64,
    gamma: i64,
}

/// `SplittableRandom.GOLDEN_GAMMA`.
const GOLDEN_GAMMA: i64 = 0x9e37_79b9_7f4a_7c15_u64 as i64;

impl SplittableRandom {
    /// `new SplittableRandom(seed)`.
    pub fn new(seed: i64) -> Self {
        SplittableRandom { seed: AtomicI64::new(seed), gamma: GOLDEN_GAMMA }
    }

    /// `new SplittableRandom()`: `defaultGen.split()`, where `defaultGen` starts at `RandomSupport.initialSeed()`
    /// (here from the clock, so the values differ from a JVM's, which never matters for an unseeded generator).
    pub fn from_entropy() -> Self {
        static DEFAULT_GEN: std::sync::LazyLock<AtomicI64> = std::sync::LazyLock::new(|| {
            let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as i64).unwrap_or(0);
            AtomicI64::new(mix64(nanos))
        });
        let s = DEFAULT_GEN.fetch_add(GOLDEN_GAMMA.wrapping_mul(2), Ordering::SeqCst);
        SplittableRandom { seed: AtomicI64::new(mix64(s)), gamma: mix_gamma(s.wrapping_add(GOLDEN_GAMMA)) }
    }

    /// `split()`: a new generator seeded from this one's `nextLong()` and the next seed's gamma.
    pub fn split(&self) -> SplittableRandom {
        let seed = RandomGenerator::next_long(self);
        SplittableRandom { seed: AtomicI64::new(seed), gamma: mix_gamma(self.next_seed()) }
    }

    /// `nextSeed()`: `seed += gamma`.
    fn next_seed(&self) -> i64 {
        self.seed.fetch_add(self.gamma, Ordering::SeqCst).wrapping_add(self.gamma)
    }
}

impl RandomGenerator for SplittableRandom {
    fn next_int(&self) -> i32 {
        mix32(self.next_seed())
    }

    fn next_long(&self) -> i64 {
        mix64(self.next_seed())
    }
}

/// `SplittableRandom.mix64`: Stafford's variant 13 of MurmurHash3's finaliser.
fn mix64(z: i64) -> i64 {
    let mut z = z as u64;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (z ^ (z >> 31)) as i64
}

/// `SplittableRandom.mix32`.
fn mix32(z: i64) -> i32 {
    let mut z = z as u64;
    z = (z ^ (z >> 33)).wrapping_mul(0x62a9_d9ed_7997_05f5);
    ((z ^ (z >> 28)).wrapping_mul(0xcb24_d0a5_c88c_35b3) >> 32) as i32
}

/// `SplittableRandom.mixGamma`: an odd gamma with enough bit transitions.
fn mix_gamma(z: i64) -> i64 {
    let mut z = z as u64;
    z = (z ^ (z >> 33)).wrapping_mul(0xff51_afd7_ed55_8ccd);
    z = (z ^ (z >> 33)).wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    z = (z ^ (z >> 33)) | 1;
    let n = (z ^ (z >> 1)).count_ones();
    (if n < 24 { z ^ 0xaaaa_aaaa_aaaa_aaaa } else { z }) as i64
}
