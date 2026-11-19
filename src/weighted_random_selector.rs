//! `net.datafaker.service.WeightedRandomSelector`: a random element of a list of `{weight, value}` maps, each
//! picked with a probability proportional to its weight.

use std::sync::Arc;

use crate::java::Random;
use crate::yaml::{Map, Value};
use crate::{Error, Result};

const WEIGHT_KEY: &str = "weight";
const VALUE_KEY: &str = "value";

/// `WeightedRandomSelector(Random random)`.
#[derive(Debug, Clone)]
pub struct WeightedRandomSelector {
    random: Arc<Random>,
}

impl WeightedRandomSelector {
    /// `new WeightedRandomSelector(random)`: a fresh unseeded `Random` for `None`.
    pub fn new(random: Option<Arc<Random>>) -> WeightedRandomSelector {
        WeightedRandomSelector { random: random.unwrap_or_else(|| Arc::new(Random::from_entropy())) }
    }

    /// `random()`.
    pub fn random(&self) -> &Arc<Random> {
        &self.random
    }

    /// `select(items)`: `None` stands for a Java `null`, of the list or of an item. A weight must be a
    /// `Value::Float` (a `Double`).
    pub fn select(&self, items: Option<&[Option<Map>]>) -> Result<Value> {
        let items = validate_items_list(items)?;
        let (cumulative_weights, values) = preprocess_items(&items)?;
        let random_value = self.random.next_double() * cumulative_weights[cumulative_weights.len() - 1];
        Ok(select_weighted_element(random_value, &cumulative_weights, &values).clone())
    }
}

fn illegal(message: &str) -> Error {
    Error::illegal_argument(message)
}

fn validate_items_list(items: Option<&[Option<Map>]>) -> Result<Vec<&Map>> {
    let Some(items) = items else { return Err(illegal("Input list cannot be null")) };
    if items.is_empty() {
        return Err(illegal("Input list cannot be empty"));
    }
    let mut unique_values: Vec<&Value> = Vec::new();
    let mut checked = Vec::with_capacity(items.len());
    for item in items {
        let item = validate_item(item.as_ref())?;
        let value = &item[VALUE_KEY];
        if unique_values.contains(&value) {
            return Err(illegal(&format!("Duplicate value found: {}. Values must be unique.", value.to_java_string())));
        }
        unique_values.push(value);
        checked.push(item);
    }
    Ok(checked)
}

fn validate_item(item: Option<&Map>) -> Result<&Map> {
    let Some(item) = item else { return Err(illegal("Item cannot be null")) };
    if item.is_empty() {
        return Err(illegal("Item cannot be empty"));
    }
    if !item.contains_key(WEIGHT_KEY) || !item.contains_key(VALUE_KEY) {
        return Err(illegal("Each item must contain 'weight' and 'value' keys"));
    }
    if item[VALUE_KEY] == Value::Null {
        return Err(illegal("Value cannot be null"));
    }
    let Value::Float(weight) = item[WEIGHT_KEY] else { return Err(illegal("Weight must be a non-null Double")) };
    if weight < 0.0 || weight.is_nan() || weight.is_infinite() {
        return Err(illegal("Weight must be a non-negative number and cannot be NaN or infinite"));
    }
    Ok(item)
}

/// `preprocessItems(items, values)`: the running sums of the weights, and the values in order.
pub fn preprocess_items<'a>(items: &[&'a Map]) -> Result<(Vec<f64>, Vec<&'a Value>)> {
    let mut cumulative_weights = Vec::with_capacity(items.len());
    let mut values = Vec::with_capacity(items.len());
    let mut total_weight = 0.0;
    for item in items {
        let Value::Float(weight) = item[WEIGHT_KEY] else { return Err(illegal("Weight must be a non-null Double")) };
        if f64::MAX - total_weight < weight {
            return Err(illegal("Sum of the weights exceeds Double.MAX_VALUE"));
        }
        total_weight += weight;
        cumulative_weights.push(total_weight);
        values.push(&item[VALUE_KEY]);
    }
    if total_weight <= 0.0 {
        return Err(illegal("The total weight must be greater than 0. At least one item must have a positive weight"));
    }
    Ok((cumulative_weights, values))
}

/// `selectWeightedElement(randomValue, cumulativeWeights, values)`: the first running sum not below
/// `randomValue`, found with `Arrays.binarySearch`.
pub fn select_weighted_element<'a>(random_value: f64, cumulative_weights: &[f64], values: &[&'a Value]) -> &'a Value {
    let index = match binary_search(cumulative_weights, random_value) {
        Ok(i) => i,
        Err(insertion) => insertion,
    };
    values[index.min(cumulative_weights.len() - 1)]
}

/// `Arrays.binarySearch(double[], double)`, probe for probe.
fn binary_search(a: &[f64], key: f64) -> std::result::Result<usize, usize> {
    let (mut low, mut high) = (0isize, a.len() as isize - 1);
    while low <= high {
        let mid = ((low + high) as usize) >> 1;
        let mid_val = a[mid];
        if mid_val < key {
            low = mid as isize + 1;
        } else if mid_val > key {
            high = mid as isize - 1;
        } else {
            let (mid_bits, key_bits) = (java_double_bits(mid_val), java_double_bits(key));
            if mid_bits == key_bits {
                return Ok(mid);
            } else if mid_bits < key_bits {
                low = mid as isize + 1;
            } else {
                high = mid as isize - 1;
            }
        }
    }
    Err(low as usize)
}

/// `Double.doubleToLongBits`: every NaN collapses to one.
fn java_double_bits(d: f64) -> i64 {
    if d.is_nan() { 0x7ff8_0000_0000_0000 } else { d.to_bits() as i64 }
}
