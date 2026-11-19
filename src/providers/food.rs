//! `net.datafaker.providers.food.Food`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Food, from `food.*`.
#[derive(Clone, Copy)]
pub struct Food<'a> {
    faker: &'a Faker,
}

impl<'a> Food<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Food { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Food, key)
    }

    /// `food.ingredients`.
    pub fn ingredient(&self) -> Result<String> {
        self.resolve("food.ingredients")
    }

    /// `food.allergens`.
    pub fn allergen(&self) -> Result<String> {
        self.resolve("food.allergens")
    }

    /// `food.spices`.
    pub fn spice(&self) -> Result<String> {
        self.resolve("food.spices")
    }

    /// `food.dish`.
    pub fn dish(&self) -> Result<String> {
        self.resolve("food.dish")
    }

    /// `food.fruits`.
    pub fn fruit(&self) -> Result<String> {
        self.resolve("food.fruits")
    }

    /// `food.vegetables`.
    pub fn vegetable(&self) -> Result<String> {
        self.resolve("food.vegetables")
    }

    /// `food.sushi`.
    pub fn sushi(&self) -> Result<String> {
        self.resolve("food.sushi")
    }

    /// `food.measurement_sizes`, a space and `food.measurements`.
    pub fn measurement(&self) -> Result<String> {
        let size = self.resolve("food.measurement_sizes")?;
        Ok(format!("{size} {}", self.resolve("food.measurements")?))
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("ingredient", &[], |f, _| s(Food::new(f).ingredient())),
    Method::new("allergen", &[], |f, _| s(Food::new(f).allergen())),
    Method::new("spice", &[], |f, _| s(Food::new(f).spice())),
    Method::new("dish", &[], |f, _| s(Food::new(f).dish())),
    Method::new("fruit", &[], |f, _| s(Food::new(f).fruit())),
    Method::new("vegetable", &[], |f, _| s(Food::new(f).vegetable())),
    Method::new("sushi", &[], |f, _| s(Food::new(f).sushi())),
    Method::new("measurement", &[], |f, _| s(Food::new(f).measurement())),
];
