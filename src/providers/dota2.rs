//! `net.datafaker.providers.videogame.Dota2`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{java_str, Method, P};
use crate::{Faker, Result};

/// Dota 2, from `games.dota2.*`.
#[derive(Clone, Copy)]
pub struct Dota2<'a> {
    faker: &'a Faker,
}

impl<'a> Dota2<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Dota2 { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Dota2, key)
    }

    /// `games.dota2.faction`.
    pub fn faction(&self) -> Result<String> {
        self.resolve("games.dota2.faction")
    }

    /// `games.dota2.rank`.
    pub fn rank(&self) -> Result<String> {
        self.resolve("games.dota2.rank")
    }

    /// `games.dota2.attribute`.
    pub fn attribute(&self) -> Result<String> {
        self.resolve("games.dota2.attribute")
    }

    /// `games.dota2.building`.
    pub fn building(&self) -> Result<String> {
        self.resolve("games.dota2.building")
    }

    /// `games.dota2.hero`.
    pub fn hero(&self) -> Result<String> {
        self.resolve("games.dota2.hero")
    }

    /// `heroQuote(heroName)`: `games.dota2.<heroName>.quote` (the hero's snake_case name).
    pub fn hero_quote(&self, hero_name: &str) -> Result<String> {
        self.resolve(&format!("games.dota2.{hero_name}.quote"))
    }

    /// `games.dota2.item`.
    pub fn item(&self) -> Result<String> {
        self.resolve("games.dota2.item")
    }

    /// `games.dota2.neutral_item`.
    pub fn neutral_item(&self) -> Result<String> {
        self.resolve("games.dota2.neutral_item")
    }

    /// `games.dota2.neutral_enchantment`.
    pub fn neutral_enchantment(&self) -> Result<String> {
        self.resolve("games.dota2.neutral_enchantment")
    }

    /// `games.dota2.team`.
    pub fn team(&self) -> Result<String> {
        self.resolve("games.dota2.team")
    }

    /// `games.dota2.tier`.
    pub fn tier(&self) -> Result<String> {
        self.resolve("games.dota2.tier")
    }

    /// `games.dota2.player`.
    pub fn player(&self) -> Result<String> {
        self.resolve("games.dota2.player")
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("faction", &[], |f, _| s(Dota2::new(f).faction())),
    Method::new("rank", &[], |f, _| s(Dota2::new(f).rank())),
    Method::new("attribute", &[], |f, _| s(Dota2::new(f).attribute())),
    Method::new("building", &[], |f, _| s(Dota2::new(f).building())),
    Method::new("hero", &[], |f, _| s(Dota2::new(f).hero())),
    Method::new("heroQuote", &[P::String], |f, a| s(Dota2::new(f).hero_quote(java_str(&a[0])))).accepting_null(),
    Method::new("item", &[], |f, _| s(Dota2::new(f).item())),
    Method::new("neutralItem", &[], |f, _| s(Dota2::new(f).neutral_item())),
    Method::new("neutralEnchantment", &[], |f, _| s(Dota2::new(f).neutral_enchantment())),
    Method::new("team", &[], |f, _| s(Dota2::new(f).team())),
    Method::new("tier", &[], |f, _| s(Dota2::new(f).tier())),
    Method::new("player", &[], |f, _| s(Dota2::new(f).player())),
];
