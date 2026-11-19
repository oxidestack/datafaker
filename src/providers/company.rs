//! `net.datafaker.providers.base.Company`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::idn::faker_to_ascii;
use crate::java::string_to_lower;
use crate::yaml::Value;
use crate::{Faker, Result};

/// The state of the faker's cached `Company`: `allBuzzwords`, loaded on first use.
#[derive(Default)]
struct Buzzwords(Option<Vec<String>>);

/// Companies: names, buzzwords, logos and URLs.
#[derive(Clone, Copy)]
pub struct Company<'a> {
    faker: &'a Faker,
}

impl<'a> Company<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Company { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Company, key)
    }

    /// `company.name`.
    pub fn name(&self) -> Result<String> {
        self.resolve("company.name")
    }

    /// `company.suffix`.
    pub fn suffix(&self) -> Result<String> {
        self.resolve("company.suffix")
    }

    /// `company.industry`.
    pub fn industry(&self) -> Result<String> {
        self.resolve("company.industry")
    }

    /// `company.profession`.
    pub fn profession(&self) -> Result<String> {
        self.resolve("company.profession")
    }

    /// One of all buzzwords (the lists of `company.buzzwords` flattened, loaded once per faker as upstream's
    /// `LazyEvaluated` field).
    pub fn buzzword(&self) -> Result<String> {
        let all = self.faker.with_provider_state(ProviderKind::Company, |state: &mut Buzzwords| state.0.clone());
        let all = match all {
            Some(all) => all,
            None => {
                let all: Vec<String> = self.lists("company.buzzwords")?.into_iter().flatten().collect();
                self.faker.with_provider_state(ProviderKind::Company, |state: &mut Buzzwords| state.0.get_or_insert(all).clone())
            }
        };
        self.faker.options().next_element(&all)
    }

    /// One word of each list of `company.buzzwords`, joined by spaces.
    pub fn catch_phrase(&self) -> Result<String> {
        self.join_sample_of_each_list(self.lists("company.buzzwords")?)
    }

    /// One word of each list of `company.bs`, joined by spaces.
    pub fn bs(&self) -> Result<String> {
        self.join_sample_of_each_list(self.lists("company.bs")?)
    }

    /// A logo URL (one of 13).
    pub fn logo(&self) -> Result<String> {
        let number = self.faker.random().next_int_bounded(13)? + 1;
        Ok(format!("https://pigment.github.io/fake-logos/logos/medium/color/{number}.png"))
    }

    /// `www.<name without punctuation, IDN-encoded>.<domain suffix>`.
    pub fn url(&self) -> Result<String> {
        let name = string_to_lower(&self.name()?, self.faker.locale().language());
        let domain: String = name.chars().filter(|c| !matches!(c, '.' | ',' | '\'' | ' ' | ']' | '&')).collect();
        let domain = faker_to_ascii(&domain)?;
        Ok(format!("www.{domain}.{}", self.resolve("internet.domain_suffix")?))
    }

    /// `fetchObject(key)` as a list of string lists (an `unchecked` cast upstream).
    fn lists(&self, key: &str) -> Result<Vec<Vec<String>>> {
        let value = self.faker.fake_values_service().fetch_object(key);
        Ok(value
            .and_then(Value::as_list)
            .unwrap_or(&[])
            .iter()
            .map(|l| l.as_list().unwrap_or(&[]).iter().map(Value::to_java_string).collect())
            .collect())
    }

    fn join_sample_of_each_list(&self, lists: Vec<Vec<String>>) -> Result<String> {
        let words: Result<Vec<String>> = lists.iter().map(|l| self.faker.options().next_element(l)).collect();
        Ok(words?.join(" "))
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("name", &[], |f, _| s(Company::new(f).name())),
    Method::new("suffix", &[], |f, _| s(Company::new(f).suffix())),
    Method::new("industry", &[], |f, _| s(Company::new(f).industry())),
    Method::new("profession", &[], |f, _| s(Company::new(f).profession())),
    Method::new("buzzword", &[], |f, _| s(Company::new(f).buzzword())),
    Method::new("catchPhrase", &[], |f, _| s(Company::new(f).catch_phrase())),
    Method::new("bs", &[], |f, _| s(Company::new(f).bs())),
    Method::new("logo", &[], |f, _| s(Company::new(f).logo())),
    Method::new("url", &[], |f, _| s(Company::new(f).url())),
];
