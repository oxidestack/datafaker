//! `RandomServiceTest` (Datafaker 2.7.0, `service/RandomServiceTest.java`). Parameterized tests run on both
//! upstream providers' shapes (`new RandomService()` and `new RandomService(new Random())`, both unseeded here).

use datafaker::RandomService;
use datafaker::java::{Random, SplittableRandom};

fn services() -> [RandomService; 2] {
    [RandomService::default(), RandomService::new(Random::from_entropy())]
}

#[test]
fn test_positive_boundaries_only() {
    for rs in services() {
        assert_eq!(rs.next_long_bounded(0).unwrap_err().simple_class(), "IllegalArgumentException");
    }
}

#[test]
fn test_long_within_boundary() {
    for rs in services() {
        assert_eq!(rs.next_long_bounded(1).unwrap(), 0);
        for _ in 1..10 {
            assert!((0..2).contains(&rs.next_long_bounded(2).unwrap()));
        }
    }
}

#[test]
fn test_long_max_boundary() {
    for rs in services() {
        let v = rs.next_long_bounded(i64::MAX).unwrap();
        assert!(0 < v && v < i64::MAX);
    }
}

#[test]
fn test_int_in_range() {
    for rs in services() {
        for _ in 1..100 {
            assert!((-5..=5).contains(&rs.next_int_between(-5, 5).unwrap()));
        }
    }
}

#[test]
fn next_int_returns_value_within_given_range() {
    let rs = RandomService::default();
    for _ in 0..10_000 {
        assert!((2..=6).contains(&rs.next_int_between(2, 6).unwrap()));
    }
}

#[test]
fn predictable_random_range() {
    let rs = RandomService::with_seed(10);
    assert_eq!(rs.next_int(), -1157793070);
    assert_eq!(rs.next_int_bounded(100).unwrap(), 80);
    assert_eq!(rs.next_int_between(0, 100).unwrap(), 35);
    assert_eq!(rs.next_float(), 0.41291267f32);
    assert_eq!(rs.next_long(), 1092083446069765248);
    assert_eq!(rs.next_long_bounded(100).unwrap(), 0);
    assert_eq!(rs.next_long_between(100, 1000).unwrap(), 538);
    assert!(!rs.next_boolean());
}

#[test]
fn equal_min_and_max() {
    assert_eq!(RandomService::default().next_int_between(42, 42).unwrap(), 42);
}

#[test]
fn extreme_integer_values() {
    for _ in 0..10 {
        let rs = RandomService::default();
        assert!(rs.next_int_between(1, i32::MAX).unwrap() >= 1);
        assert!(rs.next_int_between(i32::MIN, 0).unwrap() <= 0);
        rs.next_int_between(i32::MIN, i32::MAX).unwrap();
    }
}

#[test]
fn test_double_in_range() {
    for rs in services() {
        for _ in 1..100 {
            let d = rs.next_double_between(-5.0, 5.0);
            assert!((-5.0..=5.0).contains(&d));
        }
    }
}

#[test]
fn test_long_in_range() {
    for rs in services() {
        for _ in 1..1_000 {
            assert!((-5_000_000_000..=5_000_000_000).contains(&rs.next_long_between(-5_000_000_000, 5_000_000_000).unwrap()));
        }
    }
}

#[test]
fn extreme_long_values() {
    for _ in 0..10 {
        let rs = RandomService::default();
        assert!((0..=i64::MAX - 1).contains(&rs.next_long_between(0, i64::MAX - 1).unwrap()));
        assert!(rs.next_long_between(i64::MIN, 0).unwrap() <= 0);
        rs.next_long_between(i64::MIN, i64::MAX).unwrap();
    }
}

fn is_upper_hex8(s: &str) -> bool {
    s.len() == 8 && s.chars().all(|c| c.is_ascii_digit() || ('A'..='F').contains(&c))
}

#[test]
fn test_hex() {
    for rs in services() {
        assert!(is_upper_hex8(&rs.hex_with(8, true)));
    }
}

#[test]
fn test_default_hex() {
    for rs in services() {
        assert!(is_upper_hex8(&rs.hex()));
    }
}

const RING: [&str; 4] = ["ELVES", "DWARVES", "MEN", "LORD"];

#[test]
fn next_enum() {
    let rs = RandomService::default();
    let mut all = std::collections::BTreeSet::new();
    for _ in 1..200 {
        all.insert(rs.next_enum(RING.len()).unwrap());
    }
    assert_eq!(all.len(), 4);
}

#[test]
fn predictable_enum() {
    for (seed, expected) in [(12345, ["DWARVES", "MEN", "LORD", "LORD", "LORD"]), (42, ["MEN", "ELVES", "MEN", "ELVES", "DWARVES"])] {
        let rs = RandomService::with_seed(seed);
        let got: Vec<&str> = (0..5).map(|_| RING[rs.next_enum(4).unwrap()]).collect();
        assert_eq!(got, expected);
    }
}

// `customRandomGenerator` (a `RandomService` over `SplittableRandom`) is not ported: this `RandomService` wraps
// `java.util.Random` only. See docs/test-coverage.md.

/// `customRandomGenerator` (a `@RepeatedTest(10)`): a `RandomService` over a `SplittableRandom`. Upstream's
/// `nextFloat` bound starts at `Float.MIN_VALUE`, which a draw of exactly 0 would fail; the range here is `[0, 1)`.
#[test]
fn custom_random_generator() {
    for _ in 0..10 {
        let rs = RandomService::new(SplittableRandom::from_entropy());
        rs.next_int();
        rs.next_long();
        assert!((0.0..1.0).contains(&rs.next_float()));
        assert!((0.0..1.0).contains(&rs.next_double()));
        rs.next_boolean();
        assert_eq!(rs.next_random_bytes(8).len(), 8);
    }
}
