//! `net.datafaker.providers.base.Unique`.

use std::collections::HashMap;

use super::{ProviderKind, s};
use crate::dispatch::{Method, P};
use crate::yaml::Value;
use crate::{Error, Faker, Locale, Result};

/// The state of the faker's cached `Unique`: the values not drawn yet, per locale and key.
#[derive(Default)]
struct Remaining(HashMap<Locale, HashMap<String, Vec<String>>>);

/// Values of a YAML list without repetition, per faker.
#[derive(Clone, Copy)]
pub struct Unique<'a> {
    faker: &'a Faker,
}

impl<'a> Unique<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Unique { faker }
    }

    /// `fetchFromYaml(key)`: a value of the list at `key` (its non-list elements) that this faker has not returned
    /// for the key and locale yet; `NoSuchElementException` once all are used, or when there is no such list.
    pub fn fetch_from_yaml(&self, key: &str) -> Result<String> {
        let locale = self.faker.locale();
        let known = self.faker.with_provider_state(ProviderKind::Unique, |state: &mut Remaining| {
            state.0.get(&locale).and_then(|m| m.get(key)).cloned()
        });
        let mut values = match known {
            Some(values) => values,
            None => self.fetch_values(key)?,
        };
        if values.is_empty() {
            return Err(Error::new(
                "NoSuchElementException",
                format!("All possible values have been generated for key {key} under locale {locale}"),
            ));
        }
        let index = self.faker.random().next_int_between(0, values.len() as i32 - 1)? as usize;
        let value = values.swap_remove(index);
        self.faker.with_provider_state(ProviderKind::Unique, |state: &mut Remaining| {
            state.0.entry(locale).or_default().insert(key.to_string(), values);
        });
        Ok(value)
    }

    fn fetch_values(&self, key: &str) -> Result<Vec<String>> {
        let no_values = || Error::new("NoSuchElementException", format!("No values found for key {key}"));
        let Some(Value::List(list)) = self.faker.fake_values_service().fetch_object(key) else {
            return Err(no_values());
        };
        let values: Vec<String> = list.iter().filter(|v| !matches!(v, Value::List(_))).map(Value::to_java_string).collect();
        if values.is_empty() { Err(no_values()) } else { Ok(values) }
    }
}

pub(crate) static METHODS: &[Method] = &[Method::new("fetchFromYaml", &[P::String], |f, a| {
    s(Unique::new(f).fetch_from_yaml(crate::dispatch::non_null(a, 0, "Cannot invoke \"String.length()\" because \"string\" is null")?.as_str()))
})
.accepting_null()];
