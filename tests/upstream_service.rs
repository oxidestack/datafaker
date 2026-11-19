//! Datafaker 2.7.0's tests of `net.datafaker.service` helpers and other small core classes: `RangeTest`,
//! `RandomNumbersTest`, `EvenlyDistributedRandomLongGeneratorTest`, `WeightedRandomSelectorTest`,
//! `LazyEvaluatedTest`, `EnFileTest` and `FakerConcurrencyModificationExceptionTest`.

mod common;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use common::EvenlyDistributedRandomLongGenerator;
use datafaker::java::Random;
use datafaker::java::random::RandomOverrides;
use datafaker::lazy::LazyEvaluated;
use datafaker::weighted_random_selector::{preprocess_items, select_weighted_element};
use datafaker::yaml::{Map, Value};
use datafaker::{Faker, RandomService, Range, WeightedRandomSelector};

// RangeTest

#[test]
fn inclusive_min_should_be_less_than_max() {
    assert_eq!(Range::inclusive(45, 44).unwrap_err().simple_class(), "IllegalArgumentException");
    assert!(Range::inclusive(45, 45).is_ok());
}

#[test]
fn exclusive_min_should_be_less_than_max_minus_one() {
    let message = |from: i64, to: i64| Range::exclusive(from, to).unwrap_err().message().to_string();
    assert_eq!(message(44, 45), "Lower bound (44) >= upper bound-1 (44)");
    // `Range.exclusive(Integer.MIN_VALUE, Integer.MIN_VALUE)` widens to long before subtracting.
    assert_eq!(message(i32::MIN as i64, i32::MIN as i64), "Lower bound (-2147483648) >= upper bound-1 (-2147483649)");
    assert_eq!(message(i64::MAX, i64::MAX), "Lower bound (9223372036854775807) >= upper bound-1 (9223372036854775806)");
    assert_eq!(message(i64::MIN, i64::MIN), "Lower bound (-9223372036854775808) >= upper bound (-9223372036854775808)");
    assert_eq!(message(i64::MAX, i64::MIN), "Lower bound (9223372036854775807) >= upper bound (-9223372036854775808)");
    assert!(Range::exclusive(44, 46).is_ok());
}

#[test]
fn inclusive_exclusive_min_should_be_less_than_max() {
    assert_eq!(Range::inclusive_exclusive(44, 44).unwrap_err().simple_class(), "IllegalArgumentException");
    assert!(Range::inclusive_exclusive(44, 45).is_ok());
}

#[test]
fn exclusive_inclusive_min_should_be_less_than_max() {
    assert_eq!(Range::exclusive_inclusive(44, 44).unwrap_err().simple_class(), "IllegalArgumentException");
    assert!(Range::exclusive_inclusive(44, 45).is_ok());
}

// RandomNumbersTest

fn all<T: Ord>(mut lambda: impl FnMut() -> T) -> Vec<T> {
    (0..1000).map(|_| lambda()).collect::<BTreeSet<T>>().into_iter().collect()
}

#[test]
fn next_int_min_inclusive_max_exclusive() {
    let rs = RandomService::default();
    assert_eq!(all(|| rs.next_int_bounded(3).unwrap()), [0, 1, 2]);
    assert_eq!(all(|| rs.next_int_between(2, 4).unwrap()), [2, 3, 4]); // legacy
    assert_eq!(all(|| rs.next_long_bounded(3).unwrap()), [0, 1, 2]);
    assert_eq!(all(|| rs.next_long_between(2, 4).unwrap()), [2, 3]);
}

#[test]
fn range_inclusive() {
    let rs = RandomService::default();
    assert_eq!(all(|| rs.next_int_range(Range::inclusive(2, 4).unwrap()).unwrap()), [2, 3, 4]);
    assert_eq!(all(|| rs.next_long_range(Range::inclusive(2, 4).unwrap()).unwrap()), [2, 3, 4]);
}

#[test]
fn range_inclusive_exclusive() {
    let rs = RandomService::default();
    assert_eq!(all(|| rs.next_int_range(Range::inclusive_exclusive(2, 5).unwrap()).unwrap()), [2, 3, 4]);
    assert_eq!(all(|| rs.next_long_range(Range::inclusive_exclusive(2, 5).unwrap()).unwrap()), [2, 3, 4]);
}

#[test]
fn range_exclusive_inclusive() {
    let rs = RandomService::default();
    assert_eq!(all(|| rs.next_int_range(Range::exclusive_inclusive(2, 5).unwrap()).unwrap()), [3, 4, 5]);
    assert_eq!(all(|| rs.next_long_range(Range::exclusive_inclusive(2, 5).unwrap()).unwrap()), [3, 4, 5]);
}

#[test]
fn range_exclusive() {
    let rs = RandomService::default();
    assert_eq!(all(|| rs.next_long_range(Range::exclusive(2, 5).unwrap()).unwrap()), [3, 4]);
    assert_eq!(all(|| rs.next_int_range(Range::exclusive(2, 5).unwrap()).unwrap()), [3, 4]);
}

#[test]
fn next_double() {
    let rs = RandomService::default();
    for _ in 0..10 {
        assert!(rs.next_double_between(2.0, 3.0) >= 2.0);
        let v = rs.next_double_between(2.0, 3.0);
        assert!(v > 1.9999 && v < 3.0);
    }
}

// EvenlyDistributedRandomLongGeneratorTest

#[test]
fn single_number() {
    let generator = EvenlyDistributedRandomLongGenerator::new(8, 9);
    for _ in 0..4 {
        assert_eq!(generator.next_long(), 8);
    }
}

#[test]
fn short_interval() {
    let generator = EvenlyDistributedRandomLongGenerator::new(6, 8);
    let values: HashSet<i64> = [generator.next_long(), generator.next_long()].into();
    assert_eq!(values, [6, 7].into());
}

#[test]
fn big_numbers() {
    let i = 7_609_772_473_929_375_770i64;
    let generator = EvenlyDistributedRandomLongGenerator::new(i, i + 6);
    let mut values = Vec::new();
    for _ in 0..4 {
        values.push(generator.next_long());
    }
    values.push(generator.next_long_range(i, i + 6).unwrap());
    values.push(generator.next_long_range(i, i + 6).unwrap());
    values.sort();
    assert_eq!(values, (0..6).map(|k| i + k).collect::<Vec<_>>());
}

// WeightedRandomSelectorTest

const ITERATIONS: usize = 1000;

fn item(value: Option<&str>, weight: Option<Value>) -> Option<Map> {
    let mut m = Map::new();
    if let Some(v) = value {
        m.insert("value".into(), Value::Str(v.into()));
    }
    if let Some(w) = weight {
        m.insert("weight".into(), w);
    }
    Some(m)
}

fn w(d: f64) -> Option<Value> {
    Some(Value::Float(d))
}

#[test]
fn test_constructor_with_non_null_random() {
    let provided = Arc::new(Random::from_entropy());
    let selector = WeightedRandomSelector::new(Some(provided.clone()));
    assert!(Arc::ptr_eq(selector.random(), &provided));
}

#[test]
fn test_constructor_with_null_random() {
    let selector = WeightedRandomSelector::new(None);
    selector.random().next_int();
}

fn assert_select_fails(items: Option<Vec<Option<Map>>>, expected: &str) {
    let selector = WeightedRandomSelector::new(Some(Arc::new(Random::from_entropy())));
    let e = selector.select(items.as_deref()).unwrap_err();
    assert_eq!((e.simple_class(), e.message()), ("IllegalArgumentException", expected));
}

#[test]
fn test_weighted_array_element_failure_case() {
    let keys = "Each item must contain 'weight' and 'value' keys";
    assert_select_fails(None, "Input list cannot be null");
    assert_select_fails(Some(vec![]), "Input list cannot be empty");
    assert_select_fails(Some(vec![item(Some("Element1"), None), item(Some("Element2"), w(2.0))]), keys);
    assert_select_fails(Some(vec![item(None, w(1.0)), item(Some("Element2"), w(2.0))]), keys);
    assert_select_fails(
        Some(vec![item(Some("Element1"), Some(Value::Str("1.0".into()))), item(Some("Element2"), w(2.0))]),
        "Weight must be a non-null Double",
    );
    assert_select_fails(Some(vec![None, item(Some("Element1"), w(1.0))]), "Item cannot be null");
    assert_select_fails(Some(vec![Some(Map::new()), item(Some("Element1"), w(1.0))]), "Item cannot be empty");
    let mut null_value = Map::new();
    null_value.insert("value".into(), Value::Null);
    null_value.insert("weight".into(), Value::Float(1.0));
    assert_select_fails(Some(vec![Some(null_value)]), "Value cannot be null");
    assert_select_fails(
        Some(vec![item(Some("Element1"), w(f64::MAX - 1.0)), item(Some("Element2"), w(f64::MAX - 1.0))]),
        "Sum of the weights exceeds Double.MAX_VALUE",
    );
    assert_select_fails(
        Some(vec![
            item(Some("Element1"), w(1.0)),
            item(Some("Element2"), w(1.0)),
            item(Some("Element2"), w(1.0)),
            item(Some("Element3"), w(1.0)),
        ]),
        "Duplicate value found: Element2. Values must be unique.",
    );
}

#[test]
fn test_weighted_array_element_with_invalid_weights() {
    let invalid = "Weight must be a non-negative number and cannot be NaN or infinite";
    for bad in [f64::NAN, -1.0, f64::INFINITY] {
        assert_select_fails(Some(vec![item(Some("Element1"), w(bad)), item(Some("Element2"), w(2.0))]), invalid);
    }
    assert_select_fails(
        Some(vec![item(Some("Element1"), w(0.0)), item(Some("Element2"), w(0.0)), item(Some("Element3"), w(0.0))]),
        "The total weight must be greater than 0. At least one item must have a positive weight",
    );
}

fn three_elements() -> Vec<Map> {
    [("Element1", 1.0), ("Element2", 2.0), ("Element3", 3.0)].iter().map(|(v, weight)| item(Some(v), w(*weight)).unwrap()).collect()
}

#[test]
fn test_select_weighted_element() {
    let items = three_elements();
    let refs: Vec<&Map> = items.iter().collect();
    let (cumulative, values) = preprocess_items(&refs).unwrap();
    let cases = [(0.5, "Element1"), (2.0, "Element2"), (5.5, "Element3"), (5.9, "Element3"), (6.0, "Element3"), (6.1, "Element3")];
    for (random_value, expected) in cases {
        assert_eq!(select_weighted_element(random_value, &cumulative, &values), &Value::Str(expected.into()));
    }
}

#[test]
fn test_weighted_array_element_with_single_element() {
    let selector = WeightedRandomSelector::new(Some(Arc::new(Random::from_entropy())));
    let items = vec![item(Some("Element1"), w(1.0))];
    assert_eq!(selector.select(Some(&items)).unwrap(), Value::Str("Element1".into()));
}

/// `EvenlyDistributedRandomDoubleGenerator` (upstream test support): `count` doubles, one in each of `count` equal
/// steps of `[lower, upper)`, shuffled, returned by `nextDouble()`.
#[derive(Debug)]
struct EvenlyDistributedRandomDoubleGenerator {
    values: Vec<f64>,
    index: AtomicI32,
}

impl RandomOverrides for EvenlyDistributedRandomDoubleGenerator {
    fn next_double(&self, _: &Random) -> Option<f64> {
        Some(self.values[self.index.fetch_add(1, Ordering::SeqCst) as usize])
    }

    fn next_int(&self, _: &Random) -> Option<i32> {
        panic!("UnsupportedOperationException")
    }

    fn next_long(&self, _: &Random) -> Option<i64> {
        panic!("UnsupportedOperationException")
    }
}

fn evenly_distributed_doubles(lower: i32, upper: i32, count: usize) -> Random {
    let random = Random::from_entropy();
    let step = (upper - lower) as f64 / count as f64;
    let mut values: Vec<f64> = (0..count).map(|i| lower as f64 + i as f64 * step + random.next_double() * step).collect();
    for i in (2..=values.len()).rev() {
        let j = random.next_int_bounded(i as i32).unwrap() as usize;
        values.swap(i - 1, j);
    }
    Random::from_entropy().with_overrides(Arc::new(EvenlyDistributedRandomDoubleGenerator { values, index: AtomicI32::new(0) }))
}

#[test]
fn test_weighted_array_element_with_multiple_elements() {
    let selector = WeightedRandomSelector::new(Some(Arc::new(evenly_distributed_doubles(0, 1, ITERATIONS))));
    let items: Vec<Option<Map>> = three_elements().into_iter().map(Some).collect();
    let mut counts: HashMap<String, i32> = HashMap::new();
    for _ in 0..ITERATIONS {
        let Value::Str(s) = selector.select(Some(&items)).unwrap() else { panic!("a string") };
        *counts.entry(s).or_default() += 1;
    }
    let expected = [("Element1", 166), ("Element2", 333), ("Element3", 500)];
    assert_eq!(counts.len(), 3);
    for (element, count) in expected {
        let got = counts[element];
        assert!((got - count).abs() as f64 <= count as f64 * 0.01, "{element}: {got} vs {count}");
    }
}

// LazyEvaluatedTest

fn slow_loader(executions: Arc<AtomicI32>) -> LazyEvaluated<i32> {
    LazyEvaluated::new(move || {
        std::thread::sleep(std::time::Duration::from_millis(10));
        executions.fetch_add(1, Ordering::SeqCst) + 1
    })
}

#[test]
fn not_initialized_until_called() {
    let executions = Arc::new(AtomicI32::new(0));
    let _lazy = slow_loader(executions.clone());
    assert_eq!(executions.load(Ordering::SeqCst), 0);
}

#[test]
fn initialized_only_once() {
    let executions = Arc::new(AtomicI32::new(0));
    let lazy = slow_loader(executions.clone());
    std::thread::scope(|scope| {
        for _ in 0..5 {
            scope.spawn(|| assert_eq!(*lazy.get(), 1));
        }
    });
    assert_eq!(executions.load(Ordering::SeqCst), 1, "The slowLoader should be executed only once");
}

// EnFileTest

/// To adhere to conventions, file names are lower case.
#[test]
fn file_name_convention() {
    for (file, _) in datafaker::en_files::EN_FILES {
        assert_eq!(*file, file.to_lowercase());
    }
}

// FakerConcurrencyModificationExceptionTest: two tests drawing from the shared generator of `new BaseFaker()`.

#[test]
fn concurrency_test1() {
    Faker::new().random().next_long();
}

#[test]
fn concurrency_test2() {
    Faker::new().random().next_long();
}
