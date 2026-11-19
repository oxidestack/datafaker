//! `net.datafaker.providers.entertainment.Tron`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

provider_enum! {
    /// `Tron.AlternateCharacterSpelling`: `tron.alternate_character_spellings.<key>`.
    AlternateCharacterSpelling, "TronAlternateCharacterSpelling" {
        AlanBradley = "ALAN_BRADLEY" => "alan_bradley",
        Clu = "CLU" => "clu",
        DrLoraBaines = "DR_LORA_BAINES" => "dr_lora_baines",
        DrWalterGibbs = "DR_WALTER_GIBBS" => "dr_walter_gibbs",
        EdDillinger = "ED_DILLINGER" => "ed_dillinger",
        KevinFlynn = "KEVIN_FLYNN" => "kevin_flynn",
        Mcp = "MCP" => "mcp",
        RoyKleinberg = "ROY_KLEINBERG" => "roy_kleinberg",
    }
}

provider_enum! {
    /// `Tron.Quote`: whose quote, `tron.quotes.<key>`.
    Quote, "TronQuote" {
        AlanBradley = "ALAN_BRADLEY" => "alan_bradley",
        Bit = "BIT" => "bit",
        Clu = "CLU" => "clu",
        Crom = "CROM" => "crom",
        DrLoraBaines = "DR_LORA_BAINES" => "dr_lora_baines",
        DrWalterGibbs = "DR_WALTER_GIBBS" => "dr_walter_gibbs",
        Dumont = "DUMONT" => "dumont",
        EdDillinger = "ED_DILLINGER" => "ed_dillinger",
        KevinFlynn = "KEVIN_FLYNN" => "kevin_flynn",
        Mcp = "MCP" => "mcp",
        Program = "PROGRAM" => "program",
        Ram = "RAM" => "ram",
        Sark = "SARK" => "sark",
        Tron = "TRON" => "tron",
        Yori = "YORI" => "yori",
    }
}

provider_enum! {
    /// `Tron.Character`: a kind of character, `tron.characters.<key>`.
    Character, "TronCharacter" {
        Other = "OTHER" => "other",
        Program = "PROGRAM" => "programs",
        User = "USER" => "users",
    }
}

/// Tron, from `tron.*`.
#[derive(Clone, Copy)]
pub struct Tron<'a> {
    faker: &'a Faker,
}

impl<'a> Tron<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Tron { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Tron, key)
    }

    fn any(&self, count: usize) -> Result<usize> {
        self.faker.random().next_enum(count)
    }

    /// `character()`: a character of a random [`Character`] kind.
    pub fn character(&self) -> Result<String> {
        self.character_of(Character::VALUES[self.any(Character::VALUES.len())?])
    }

    /// `character(character)`.
    pub fn character_of(&self, character: Character) -> Result<String> {
        self.resolve(&format!("tron.characters.{}", character.yaml_key()))
    }

    /// `tron.games`.
    pub fn game(&self) -> Result<String> {
        self.resolve("tron.games")
    }

    /// `tron.locations`.
    pub fn location(&self) -> Result<String> {
        self.resolve("tron.locations")
    }

    /// `quote()`: a quote of a random [`Quote`].
    pub fn quote(&self) -> Result<String> {
        self.quote_of(Quote::VALUES[self.any(Quote::VALUES.len())?])
    }

    /// `quote(quote)`.
    pub fn quote_of(&self, quote: Quote) -> Result<String> {
        self.resolve(&format!("tron.quotes.{}", quote.yaml_key()))
    }

    /// `tron.taglines`.
    pub fn tagline(&self) -> Result<String> {
        self.resolve("tron.taglines")
    }

    /// `tron.vehicles`.
    pub fn vehicle(&self) -> Result<String> {
        self.resolve("tron.vehicles")
    }

    /// `alternateCharacterSpelling()`: a spelling of a random [`AlternateCharacterSpelling`].
    pub fn alternate_character_spelling(&self) -> Result<String> {
        let spelling = AlternateCharacterSpelling::VALUES[self.any(AlternateCharacterSpelling::VALUES.len())?];
        self.alternate_character_spelling_of(spelling)
    }

    /// `alternateCharacterSpelling(alternateCharacterSpelling)`.
    pub fn alternate_character_spelling_of(&self, spelling: AlternateCharacterSpelling) -> Result<String> {
        self.resolve(&format!("tron.alternate_character_spellings.{}", spelling.yaml_key()))
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("character", &[], |f, _| s(Tron::new(f).character())),
    Method::new("character", &[Character::PARAM], |f, a| s(Tron::new(f).character_of(Character::of_yaml_key_param(&a[0], "character")?)))
        .accepting_null(),
    Method::new("game", &[], |f, _| s(Tron::new(f).game())),
    Method::new("location", &[], |f, _| s(Tron::new(f).location())),
    Method::new("quote", &[], |f, _| s(Tron::new(f).quote())),
    Method::new("quote", &[Quote::PARAM], |f, a| s(Tron::new(f).quote_of(Quote::of_yaml_key_param(&a[0], "quote")?))).accepting_null(),
    Method::new("tagline", &[], |f, _| s(Tron::new(f).tagline())),
    Method::new("vehicle", &[], |f, _| s(Tron::new(f).vehicle())),
    Method::new("alternateCharacterSpelling", &[], |f, _| s(Tron::new(f).alternate_character_spelling())),
    Method::new("alternateCharacterSpelling", &[AlternateCharacterSpelling::PARAM], |f, a| {
        let spelling = AlternateCharacterSpelling::of_yaml_key_param(&a[0], "alternateCharacterSpelling")?;
        s(Tron::new(f).alternate_character_spelling_of(spelling))
    })
    .accepting_null(),
];
