//! `net.datafaker.providers.entertainment.StarWars`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

/// The characters with quotes and alternate spellings, in upstream's order.
const CHARACTERS: [&str; 36] = [
    "admiral_ackbar", "ahsoka_tano", "anakin_skywalker", "asajj_ventress", "bendu", "boba_fett", "c_3po",
    "count_dooku", "darth_caedus", "darth_vader", "din_djarin", "emperor_palpatine", "finn", "general_hux",
    "grand_admiral_thrawn", "grand_moff_tarkin", "greedo", "grogu", "han_solo", "jabba_the_hutt", "jar_jar_binks",
    "k_2so", "kuiil", "kylo_ren", "lando_calrissian", "leia_organa", "luke_skywalker", "mace_windu", "maz_kanata",
    "moff_gideon", "obi_wan_kenobi", "padme_amidala", "qui_gon_jinn", "rey", "shmi_skywalker", "yoda",
];

/// Star Wars, from `star_wars.*`.
#[derive(Clone, Copy)]
pub struct StarWars<'a> {
    faker: &'a Faker,
}

impl<'a> StarWars<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        StarWars { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::StarWars, key)
    }

    /// `star_wars.characters`.
    pub fn character(&self) -> Result<String> {
        self.resolve("star_wars.characters")
    }

    /// `numerify` of `star_wars.call_sign`.
    pub fn call_sign(&self) -> Result<String> {
        Ok(self.faker.numerify(&self.resolve("star_wars.call_sign")?))
    }

    /// `star_wars.quotes.<character>`, the character drawn with `options().option(CHARACTERS)`.
    pub fn quotes(&self) -> Result<String> {
        let character = self.faker.options().option(&CHARACTERS)?;
        self.resolve(&format!("star_wars.quotes.{character}"))
    }

    /// `star_wars.vehicles`.
    pub fn vehicles(&self) -> Result<String> {
        self.resolve("star_wars.vehicles")
    }

    /// `star_wars.droids`.
    pub fn droids(&self) -> Result<String> {
        self.resolve("star_wars.droids")
    }

    /// `star_wars.planets`.
    pub fn planets(&self) -> Result<String> {
        self.resolve("star_wars.planets")
    }

    /// `star_wars.species`.
    pub fn species(&self) -> Result<String> {
        self.resolve("star_wars.species")
    }

    /// `star_wars.wookiee_words`.
    pub fn wookie_words(&self) -> Result<String> {
        self.resolve("star_wars.wookiee_words")
    }

    /// `star_wars.alternate_character_spellings.<character>`, the character drawn as in [`StarWars::quotes`].
    pub fn alternate_character_spelling(&self) -> Result<String> {
        let character = self.faker.options().option(&CHARACTERS)?;
        self.resolve(&format!("star_wars.alternate_character_spellings.{character}"))
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("character", &[], |f, _| s(StarWars::new(f).character())),
    Method::new("callSign", &[], |f, _| s(StarWars::new(f).call_sign())),
    Method::new("quotes", &[], |f, _| s(StarWars::new(f).quotes())),
    Method::new("vehicles", &[], |f, _| s(StarWars::new(f).vehicles())),
    Method::new("droids", &[], |f, _| s(StarWars::new(f).droids())),
    Method::new("planets", &[], |f, _| s(StarWars::new(f).planets())),
    Method::new("species", &[], |f, _| s(StarWars::new(f).species())),
    Method::new("wookieWords", &[], |f, _| s(StarWars::new(f).wookie_words())),
    Method::new("alternateCharacterSpelling", &[], |f, _| s(StarWars::new(f).alternate_character_spelling())),
];
