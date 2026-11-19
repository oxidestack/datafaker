//! `net.datafaker.providers.base.Music`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method};
use crate::{Faker, Result};

const KEYS: &[&str] = &[
    "C",
    "D",
    "E",
    "F",
    "G",
    "A",
    "B",
];

const KEY_VARIANTS: &[&str] = &[
    "b",
    "#",
    "",
];

const CHORD_TYPES: &[&str] = &[
    "",
    "maj",
    "6",
    "maj7",
    "m",
    "m7",
    "-7",
    "7",
    "dom7",
    "dim",
    "dim7",
    "m7b5",
];

/// Music, from `music.*` and fixed keys and chords.
#[derive(Clone, Copy)]
pub struct Music<'a> {
    faker: &'a Faker,
}

impl<'a> Music<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Music { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Music, key)
    }

    /// `music.instruments`.
    pub fn instrument(&self) -> Result<String> {
        self.resolve("music.instruments")
    }

    /// A key, `C` to `B`, with `b`, `#` or nothing.
    pub fn key(&self) -> Result<String> {
        let options = self.faker.options();
        Ok(format!("{}{}", options.option(KEYS)?, options.option(KEY_VARIANTS)?))
    }

    /// A key and a chord type.
    pub fn chord(&self) -> Result<String> {
        Ok(format!("{}{}", self.key()?, self.faker.options().option(CHORD_TYPES)?))
    }

    /// `music.genres`.
    pub fn genre(&self) -> Result<String> {
        self.resolve("music.genres")
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("instrument", &[], |f, _| s(Music::new(f).instrument())),
    Method::new("key", &[], |f, _| s(Music::new(f).key())),
    Method::new("chord", &[], |f, _| s(Music::new(f).chord())),
    Method::new("genre", &[], |f, _| s(Music::new(f).genre())),
];
