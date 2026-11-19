//! `net.datafaker.providers.videogame.Minecraft`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Minecraft, from `minecraft.*`.
#[derive(Clone, Copy)]
pub struct Minecraft<'a> {
    faker: &'a Faker,
}

impl<'a> Minecraft<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Minecraft { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Minecraft, key)
    }

    /// `minecraft.item_name`.
    pub fn item_name(&self) -> Result<String> {
        self.resolve("minecraft.item_name")
    }

    /// `minecraft.tile_name`.
    pub fn tile_name(&self) -> Result<String> {
        self.resolve("minecraft.tile_name")
    }

    /// `minecraft.entity_name`.
    pub fn entity_name(&self) -> Result<String> {
        self.resolve("minecraft.entity_name")
    }

    /// `minecraft.monster_name`.
    pub fn monster_name(&self) -> Result<String> {
        self.resolve("minecraft.monster_name")
    }

    /// `minecraft.animal_name`.
    pub fn animal_name(&self) -> Result<String> {
        self.resolve("minecraft.animal_name")
    }

    /// An item name or a tile name, as `random().nextBoolean()` says.
    pub fn tile_item_name(&self) -> Result<String> {
        if self.faker.random().next_boolean() { self.item_name() } else { self.tile_name() }
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("itemName", &[], |f, _| s(Minecraft::new(f).item_name())),
    Method::new("tileName", &[], |f, _| s(Minecraft::new(f).tile_name())),
    Method::new("entityName", &[], |f, _| s(Minecraft::new(f).entity_name())),
    Method::new("monsterName", &[], |f, _| s(Minecraft::new(f).monster_name())),
    Method::new("animalName", &[], |f, _| s(Minecraft::new(f).animal_name())),
    Method::new("tileItemName", &[], |f, _| s(Minecraft::new(f).tile_item_name())),
];
