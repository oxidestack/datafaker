//! `net.datafaker.providers.entertainment.OscarMovie`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Oscar-winning movies, from `oscar_movie.<year>.<choice>.*`: each call draws a year and a choice first.
#[derive(Clone, Copy)]
pub struct OscarMovie<'a> {
    faker: &'a Faker,
}

impl<'a> OscarMovie<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        OscarMovie { faker }
    }

    /// `faker.resolve("oscar_movie.year.years")`.
    pub fn get_year(&self) -> Result<String> {
        self.faker.resolve("oscar_movie.year.years")
    }

    /// `faker.resolve("oscar_movie.year.choice")`.
    pub fn get_choice(&self) -> Result<String> {
        self.faker.resolve("oscar_movie.year.choice")
    }

    /// `resolve("oscar_movie." + year + "." + choice + field)`, the year drawn before the choice.
    fn field(&self, field: &str) -> Result<String> {
        let year = self.get_year()?;
        let choice = self.get_choice()?;
        resolve(self.faker, ProviderKind::OscarMovie, &format!("oscar_movie.{year}.{choice}{field}"))
    }

    /// `.actor`.
    pub fn actor(&self) -> Result<String> {
        self.field(".actor")
    }

    /// `.movieName`.
    pub fn movie_name(&self) -> Result<String> {
        self.field(".movieName")
    }

    /// `.quote`.
    pub fn quote(&self) -> Result<String> {
        self.field(".quote")
    }

    /// `.character`.
    pub fn character(&self) -> Result<String> {
        self.field(".character")
    }

    /// `.releaseDate`.
    pub fn release_date(&self) -> Result<String> {
        self.field(".releaseDate")
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("getYear", &[], |f, _| s(OscarMovie::new(f).get_year())),
    Method::new("getChoice", &[], |f, _| s(OscarMovie::new(f).get_choice())),
    Method::new("actor", &[], |f, _| s(OscarMovie::new(f).actor())),
    Method::new("movieName", &[], |f, _| s(OscarMovie::new(f).movie_name())),
    Method::new("quote", &[], |f, _| s(OscarMovie::new(f).quote())),
    Method::new("character", &[], |f, _| s(OscarMovie::new(f).character())),
    Method::new("releaseDate", &[], |f, _| s(OscarMovie::new(f).release_date())),
];
