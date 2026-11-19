//! `net.datafaker.providers.entertainment.Dune`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

provider_enum! {
    /// `Dune.Quote`: whose quote, `dune.quotes.<key>`.
    Quote, "DuneQuote" {
        GuildNavigator = "GUILD_NAVIGATOR" => "guild_navigator",
        Emperor = "EMPEROR" => "emperor",
        Paul = "PAUL" => "paul",
        Thufir = "THUFIR" => "thufir",
        Jessica = "JESSICA" => "jessica",
        Irulan = "IRULAN" => "irulan",
        Mohiam = "MOHIAM" => "mohiam",
        Gurney = "GURNEY" => "gurney",
        Leto = "LETO" => "leto",
        Stilgar = "STILGAR" => "stilgar",
        LietKynes = "LIET_KYNES" => "liet_kynes",
        PardotKynes = "PARDOT_KYNES" => "pardot_kynes",
        BaronHarkonnen = "BARON_HARKONNEN" => "baron_harkonnen",
        Piter = "PITER" => "piter",
        Alia = "ALIA" => "alia",
        Mapes = "MAPES" => "mapes",
        Duncan = "DUNCAN" => "duncan",
        Yueh = "YUEH" => "yueh",
    }
}

provider_enum! {
    /// `Dune.Saying`: whose saying, `dune.sayings.<key>`.
    Saying, "DuneSaying" {
        BeneGesserit = "BENE_GESSERIT" => "bene_gesserit",
        Fremen = "FREMEN" => "fremen",
        Mentat = "MENTAT" => "mentat",
        Muaddib = "MUADDIB" => "muaddib",
        OrangeCatholicBible = "ORANGE_CATHOLIC_BIBLE" => "orange_catholic_bible",
    }
}

/// Dune, from `dune.*`.
#[derive(Clone, Copy)]
pub struct Dune<'a> {
    faker: &'a Faker,
}

impl<'a> Dune<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Dune { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Dune, key)
    }

    /// `dune.characters`.
    pub fn character(&self) -> Result<String> {
        self.resolve("dune.characters")
    }

    /// `dune.titles`.
    pub fn title(&self) -> Result<String> {
        self.resolve("dune.titles")
    }

    /// `dune.planets`.
    pub fn planet(&self) -> Result<String> {
        self.resolve("dune.planets")
    }

    /// `quote()`: a quote of a random [`Quote`] (`options().option(Quote.class)`).
    pub fn quote(&self) -> Result<String> {
        self.quote_of(Quote::VALUES[self.faker.random().next_enum(Quote::VALUES.len())?])
    }

    /// `quote(quote)`.
    pub fn quote_of(&self, quote: Quote) -> Result<String> {
        self.resolve(&format!("dune.quotes.{}", quote.yaml_key()))
    }

    /// `saying()`: a saying of a random [`Saying`].
    pub fn saying(&self) -> Result<String> {
        self.saying_of(Saying::VALUES[self.faker.random().next_enum(Saying::VALUES.len())?])
    }

    /// `saying(saying)`.
    pub fn saying_of(&self, saying: Saying) -> Result<String> {
        self.resolve(&format!("dune.sayings.{}", saying.yaml_key()))
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("character", &[], |f, _| s(Dune::new(f).character())),
    Method::new("title", &[], |f, _| s(Dune::new(f).title())),
    Method::new("planet", &[], |f, _| s(Dune::new(f).planet())),
    Method::new("quote", &[], |f, _| s(Dune::new(f).quote())),
    Method::new("quote", &[Quote::PARAM], |f, a| s(Dune::new(f).quote_of(Quote::of_yaml_key_param(&a[0], "quote")?))).accepting_null(),
    Method::new("saying", &[], |f, _| s(Dune::new(f).saying())),
    Method::new("saying", &[Saying::PARAM], |f, a| s(Dune::new(f).saying_of(Saying::of_yaml_key_param(&a[0], "saying")?))).accepting_null(),
];
