//! `NumberTest` (Datafaker 2.7.0, `providers/base/NumberTest.java`), all 29 tests.

use std::collections::{HashMap, HashSet};

mod common;

use common::EvenlyDistributedRandomLongGenerator;
use datafaker::{Faker, Locale, RandomService};
use datafaker::java::{Random, double_to_string};

fn faker() -> Faker {
    Faker::new()
}

#[test]
fn test_random_digit() {
    let f = faker();
    let mut nums = HashSet::new();
    for _ in 0..1000 {
        let v = f.number().random_digit();
        assert!((0..=9).contains(&v));
        nums.insert(v);
    }
    assert_eq!(nums.len(), 10);
}

#[test]
fn random_digits() {
    let f = faker();
    for length in [0, 1, 2, 3, 4, 8, 16, 25, 36] {
        let v = f.number().random_digits(length);
        assert_eq!(v.len(), length);
        assert!(v.iter().all(|d| (0..=9).contains(d)));
    }
}

#[test]
fn test_random_digit_not_zero() {
    let f = faker();
    let mut nums = HashSet::new();
    for _ in 0..1000 {
        let v = f.number().random_digit_not_zero();
        assert!((1..=9).contains(&v));
        nums.insert(v);
    }
    assert_eq!(nums.len(), 9);
}

#[test]
fn test_random_number() {
    assert!(faker().number().random_number() < i64::MAX);
}

#[test]
fn test_random_number_with_single_digit_strict() {
    let f = faker();
    for _ in 0..100 {
        assert!((0..10).contains(&f.number().random_number_digits(1).unwrap()));
    }
}

#[test]
fn random_number_with_zero_digits() {
    assert_eq!(faker().number().random_number_digits(0).unwrap_err().simple_class(), "IllegalArgumentException");
}

#[test]
fn test_random_number_with_given_digits_strict() {
    let f = faker();
    for i in 1..9 {
        for _ in 0..100 {
            assert_eq!(f.number().random_number_digits(i).unwrap().to_string().len(), i as usize);
        }
    }
}

#[test]
fn test_number_between() {
    let f = faker();
    for i in 1..100 {
        assert!((0..=i).contains(&f.number().number_between(0, i)));
    }
    for i in 1i64..100 {
        assert!((0..=i).contains(&f.number().number_between_long(0, i).unwrap()));
    }
    let v1 = f.number().number_between_long(1, 980_000_000).unwrap();
    assert!(v1 > 1 && v1 < 980_000_000);
}

#[test]
fn test_long_number_between_repeated() {
    for _ in 0..100 {
        assert!((1..10).contains(&faker().number().number_between_long(1, 10).unwrap()));
    }
}

#[test]
fn test_int_number_between_repeated() {
    for _ in 0..100 {
        assert!((1..10).contains(&faker().number().number_between(1, 10)));
    }
}

#[test]
fn test_double_number_between_repeated_low_high() {
    for _ in 0..100 {
        let v = faker().number().number_between_double(1.0, 10.0);
        assert!((1.0..10.0).contains(&v));
    }
}

#[test]
fn test_double_number_between_repeated_high_low() {
    for _ in 0..100 {
        let v = faker().number().number_between_double(10.0, 1.0);
        assert!(v <= 10.0 && v > 1.0, "{v}");
    }
}

#[test]
fn test_number_between_one_and_three() {
    let f = faker();
    let mut nums = HashSet::new();
    for _ in 0..1000 {
        let v = f.number().number_between(0, 3);
        assert!((0..3).contains(&v));
        nums.insert(v);
    }
    assert_eq!(nums.len(), 3);
}

#[test]
fn test_long_between_one_and_three() {
    let f = faker();
    let mut nums = HashSet::new();
    for _ in 0..1000 {
        let v = f.number().number_between_long(0, 3).unwrap();
        assert!((0..3).contains(&v));
        nums.insert(v);
    }
    assert_eq!(nums.len(), 3);
}

#[test]
fn number_between_int_int_zero_min_max() {
    let f = faker();
    assert_eq!(f.number().number_between(0, 0), 0);
    assert_eq!(f.number().number_between(2, 2), 2);
}

#[test]
fn number_between_long_long_zero_min_max() {
    let f = faker();
    assert_eq!(f.number().number_between_long(0, 0).unwrap(), 0);
    assert_eq!(f.number().number_between_long(2, 2).unwrap(), 2);
}

fn numbers_to_get(min: i64, max: i64) -> i64 {
    let n = (max / 4 - min / 4).min(1000);
    if n == 0 { 1000 } else { n }
}

fn unique_percentage<T: std::hash::Hash + Eq>(iterations: i64, mut f: impl FnMut() -> T) -> f64 {
    let values: HashSet<T> = (0..iterations).map(|_| f()).collect();
    values.len() as f64 / iterations as f64
}

/// `randomizationQualityTest`: the share of the ranges `[-l, l)` (`l` from 25 to 975 by 25) over which more than
/// 80% of the draws are unique.
fn randomization_quality(run: impl Fn(i64, i64) -> f64) -> f64 {
    let mut above = 0;
    let mut total = 0;
    let mut l = 25;
    while l < 1000 {
        if run(-l, l) > 0.8 {
            above += 1;
        }
        total += 1;
        l += 25;
    }
    above as f64 / total as f64
}

#[test]
fn test_random_double() {
    let number = faker();
    for i in 1..5 {
        for _ in 0..100 {
            let value = number.number().random_double_int(i, 1, 1000);
            // `BigDecimal.valueOf(value).stripTrailingZeros().toString()`; in [1, 1000] that is `Double.toString`
            // without trailing zeros, and never in `E+` form below 1e7.
            let s = double_to_string(value);
            let s = s.trim_end_matches('0').trim_end_matches('.');
            if let Some(dot) = s.find('.') {
                assert!(s.len() - dot - 1 <= i as usize, "{value} has more than {i} decimals");
            }
        }
    }
}

#[test]
fn random_double_randomization_quality() {
    let f = faker();
    let run = |min: i64, max: i64| {
        unique_percentage(numbers_to_get(min, max), || f.number().random_double_int(0, min as i32, max as i32).to_bits())
    };
    assert!(randomization_quality(run) >= 0.9);
    // Issue #121: the extreme range.
    assert!(run(i32::MIN as i64, i32::MAX as i64) >= 0.8);
}

#[test]
fn number_between_long_long_randomization_quality() {
    let f = faker();
    let run = |min: i64, max: i64| unique_percentage(numbers_to_get(min, max), || f.number().number_between_long(min, max).unwrap());
    assert!(randomization_quality(run) >= 0.9);
    assert!(run(i64::MIN, i64::MAX) >= 0.8);
}

#[test]
fn test_random_double_max_equals_min() {
    assert_eq!(faker().number().random_double(1, 42, 42), 42.0);
}

#[test]
fn number_between_int_int_randomization_quality() {
    let f = faker();
    let run = |min: i64, max: i64| unique_percentage(numbers_to_get(min, max), || f.number().number_between(min as i32, max as i32));
    let mut above = 0;
    let mut total = 0;
    let mut l = 25;
    while l < 1000 {
        if run(-l, l) > 0.8 {
            above += 1;
        }
        total += 1;
        l += 25;
    }
    assert!(above as f64 / total as f64 >= 0.9);
    assert!(run(i32::MIN as i64, i32::MAX as i64) >= 0.8);
}

#[test]
fn test_digit() {
    let d = faker().number().digit();
    assert!(d.len() == 1 && d.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_digits() {
    let d = faker().number().digits(5);
    assert!(d.len() == 5 && d.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_int_number_between_quality() {
    let random = Random::from_entropy();
    let f = faker();
    let test_case = 100_000;
    let min = random.next_int().wrapping_abs();
    let max = min.saturating_add(1.max(random.next_int_bounded(100).unwrap().abs()));
    let mean = test_case as f64 / (max - min) as f64;
    let mut map: HashMap<i32, i32> = HashMap::new();
    for _ in 0..test_case {
        *map.entry(f.number().number_between(min, max)).or_default() += 1;
    }
    for count in map.values() {
        assert!((mean - *count as f64) / mean < 0.2);
    }
}

#[test]
fn test_number_between_contain() {
    let random = Random::from_entropy();
    let f = faker();
    let size = random.next_int_bounded(100).unwrap();
    let min_int = random.next_int().wrapping_abs().min(i32::MAX - 100);
    let max_int = min_int + size;
    let mut ints = HashSet::new();
    for _ in 0..10_000 {
        let v = f.number().number_between(min_int, max_int);
        assert!((min_int..=max_int).contains(&v));
        ints.insert(v);
    }
    assert_eq!(ints.len(), 1.max(size) as usize);
    let min_long = random.next_long().wrapping_abs().min(i64::MAX - 100);
    let max_long = min_long + size as i64;
    let mut longs = HashSet::new();
    for _ in 0..10_000 {
        let v = f.number().number_between_long(min_long, max_long).unwrap();
        assert!((min_long..=max_long).contains(&v));
        longs.insert(v);
    }
    assert_eq!(longs.len(), 1.max(size) as usize);
}

#[test]
fn test_number_between_border() {
    let random = Random::from_entropy();
    let f = faker();
    for _ in 0..=100 {
        let (mut min, mut max) = (0i64, -1i64);
        while max < min {
            let size = random.next_long().wrapping_abs();
            min = random.next_long().wrapping_abs();
            max = min.wrapping_add(size);
        }
        for _ in 0..100 {
            let v = f.number().number_between_long(min, max).unwrap();
            assert!(v >= min && v < max);
        }
    }
}

#[test]
fn test_positive() {
    for _ in 0..10 {
        assert!(faker().number().positive() > 0);
    }
}

#[test]
fn test_negative() {
    for _ in 0..10 {
        assert!(faker().number().negative() < 0);
    }
}

#[test]
fn test_long_number_between_quality() {
    let random = Random::from_entropy();
    let test_case = 100_000;
    let min = random.next_long_range(i64::MIN, i64::MAX - 200).unwrap();
    let max = min + 1.max(random.next_int_bounded(200).unwrap().abs() as i64);
    let mean = test_case as f64 / (max - min) as f64;
    let faker = Faker::with_locale_and_random(Locale::english(), RandomService::new(EvenlyDistributedRandomLongGenerator::new(min, max)));
    let number = faker.number();
    let mut map: HashMap<i64, i32> = HashMap::new();
    for _ in 0..test_case {
        *map.entry(number.number_between_long(min, max).unwrap()).or_default() += 1;
    }
    for (value, count) in &map {
        assert!((mean - *count as f64).abs() / mean < 0.002, "entry {value}={count}, all numbers: {map:?}");
    }
}
