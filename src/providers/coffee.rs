//! `net.datafaker.providers.food.Coffee`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

provider_enum! {
    /// `Coffee.Country`: a growing country, `coffee.regions.<name in lower case>`.
    Country, "CoffeeCountry" {
        Brazil = "BRAZIL" => "brazil",
        Colombia = "COLOMBIA" => "colombia",
        Sumatra = "SUMATRA" => "sumatra",
        Ethiopia = "ETHIOPIA" => "ethiopia",
        Honduras = "HONDURAS" => "honduras",
        Kenya = "KENYA" => "kenya",
        Uganda = "UGANDA" => "uganda",
        Mexico = "MEXICO" => "mexico",
        Guatemala = "GUATEMALA" => "guatemala",
        Nicaragua = "NICARAGUA" => "nicaragua",
        CostaRica = "COSTA_RICA" => "costa_rica",
        Tanzania = "TANZANIA" => "tanzania",
        ElSalvador = "EL_SALVADOR" => "el_salvador",
        Rwanda = "RWANDA" => "rwanda",
        Burundi = "BURUNDI" => "burundi",
        Panama = "PANAMA" => "panama",
        Yemen = "YEMEN" => "yemen",
        India = "INDIA" => "india",
    }
}

/// Coffee, from `coffee.*`.
#[derive(Clone, Copy)]
pub struct Coffee<'a> {
    faker: &'a Faker,
}

impl<'a> Coffee<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Coffee { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Coffee, key)
    }

    /// `coffee.country`.
    pub fn country(&self) -> Result<String> {
        self.resolve("coffee.country")
    }

    /// `region()`: a region of a random [`Country`].
    pub fn region(&self) -> Result<String> {
        self.region_of(Country::VALUES[self.faker.random().next_enum(Country::VALUES.len())?])
    }

    /// `region(country)`.
    pub fn region_of(&self, country: Country) -> Result<String> {
        self.resolve(&format!("coffee.regions.{}", country.yaml_key()))
    }

    /// `coffee.variety`.
    pub fn variety(&self) -> Result<String> {
        self.resolve("coffee.variety")
    }

    /// `coffee.intensifier`.
    pub fn intensifier(&self) -> Result<String> {
        self.resolve("coffee.intensifier")
    }

    /// `coffee.body`.
    pub fn body(&self) -> Result<String> {
        self.resolve("coffee.body")
    }

    /// `coffee.descriptor`.
    pub fn descriptor(&self) -> Result<String> {
        self.resolve("coffee.descriptor")
    }

    /// `coffee.notes`.
    pub fn notes(&self) -> Result<String> {
        self.resolve("coffee.notes")
    }

    /// `coffee.name_1`.
    pub fn name1(&self) -> Result<String> {
        self.resolve("coffee.name_1")
    }

    /// `coffee.name_2`.
    pub fn name2(&self) -> Result<String> {
        self.resolve("coffee.name_2")
    }

    /// `coffee.blend_name`.
    pub fn blend_name(&self) -> Result<String> {
        self.resolve("coffee.blend_name")
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("country", &[], |f, _| s(Coffee::new(f).country())),
    Method::new("region", &[], |f, _| s(Coffee::new(f).region())),
    Method::new("region", &[Country::PARAM], |f, a| {
        let Some(country) = Country::of_nullable(&a[0]) else {
            return Err(crate::dispatch::npe("Cannot invoke \"net.datafaker.providers.food.Coffee$Country.name()\" because \"country\" is null"));
        };
        s(Coffee::new(f).region_of(country))
    })
    .accepting_null(),
    Method::new("variety", &[], |f, _| s(Coffee::new(f).variety())),
    Method::new("intensifier", &[], |f, _| s(Coffee::new(f).intensifier())),
    Method::new("body", &[], |f, _| s(Coffee::new(f).body())),
    Method::new("descriptor", &[], |f, _| s(Coffee::new(f).descriptor())),
    Method::new("notes", &[], |f, _| s(Coffee::new(f).notes())),
    Method::new("name1", &[], |f, _| s(Coffee::new(f).name1())),
    Method::new("name2", &[], |f, _| s(Coffee::new(f).name2())),
    Method::new("blendName", &[], |f, _| s(Coffee::new(f).blend_name())),
];
