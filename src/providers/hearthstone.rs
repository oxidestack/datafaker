//! `net.datafaker.providers.videogame.Hearthstone`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

/// Hearthstone, from `games.hearthstone.*`.
#[derive(Clone, Copy)]
pub struct Hearthstone<'a> {
    faker: &'a Faker,
}

impl<'a> Hearthstone<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Hearthstone { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Hearthstone, key)
    }

    /// `games.hearthstone.professions`.
    pub fn main_profession(&self) -> Result<String> {
        self.resolve("games.hearthstone.professions")
    }

    /// `games.hearthstone.characters`.
    pub fn main_character(&self) -> Result<String> {
        self.resolve("games.hearthstone.characters")
    }

    /// `games.hearthstone.patterns`.
    pub fn main_pattern(&self) -> Result<String> {
        self.resolve("games.hearthstone.patterns")
    }

    /// `random().nextInt(0, 16000)`.
    pub fn battlegrounds_score(&self) -> Result<i32> {
        self.faker.random().next_int_between(0, 16000)
    }

    /// A rank of `games.hearthstone.rank` and a number: 1 to 65000 for `Legend`, else 1 to 10.
    pub fn standard_rank(&self) -> Result<String> {
        let rank = self.resolve("games.hearthstone.rank")?;
        let max = if rank == "Legend" { 65000 } else { 10 };
        Ok(format!("{rank} {}", self.faker.random().next_int_between(1, max)?))
    }

    /// As [`Hearthstone::standard_rank`].
    pub fn wild_rank(&self) -> Result<String> {
        self.standard_rank()
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("mainProfession", &[], |f, _| s(Hearthstone::new(f).main_profession())),
    Method::new("mainCharacter", &[], |f, _| s(Hearthstone::new(f).main_character())),
    Method::new("mainPattern", &[], |f, _| s(Hearthstone::new(f).main_pattern())),
    Method::new("battlegroundsScore", &[], |f, _| Hearthstone::new(f).battlegrounds_score().map(JValue::Int)),
    Method::new("standardRank", &[], |f, _| s(Hearthstone::new(f).standard_rank())),
    Method::new("wildRank", &[], |f, _| s(Hearthstone::new(f).wild_rank())),
];
