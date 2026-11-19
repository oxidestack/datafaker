//! `net.datafaker.providers.base.Locality`.

use super::jdk_tables::{ENGLISH_DISPLAY_LANGUAGE, LOCALITY_DISPLAY_NAMES};
use super::{ProviderKind, s};
use crate::dispatch::{npe, JValue, Method};
use crate::java::RandomGenerator;
use crate::{Faker, Result};

/// `Locality.LOCALES`: the locales Datafaker supports, as tags.
pub const LOCALES: &[&str] = &[
    "_al", "_ar", "_au", "_at", "_bg", "_by", "_ca", "_ch", "_cn", "_cz", "_dk", "_ee", "_fi", "_fr", "_gb", "_ge",
    "_hu", "_jp", "_il", "_in", "_kr", "_lv", "_no", "_md", "_mk", "_nl", "_pl", "_py", "_ru", "_se", "_tr", "_tw",
    "_ua", "_us", "_uz", "_vn", "_es", "_sk", "_id", "_hr", "_pt", "_ie", "_it", "_de", "_am", "_mx", "_br", "_be",
    "_th", "ar", "be", "bg", "by", "ca", "ca-cat", "cs", "cs-cz", "da-dk", "de", "de-at", "de-ch", "el-gr", "en",
    "en-au", "en-au-ocker", "en-bork", "en-ca", "en-gb", "en-ie", "en-in", "en-md", "en-ms", "en-nep", "en-ng",
    "en-nz", "en-pak", "en-ph", "en-pk", "en-sg", "en-ug", "en-us", "en-za", "es", "es-ar", "es-mx", "es-py", "et",
    "fa", "fi-fi", "fr", "fr-ca", "fr-ch", "he", "hr", "hu", "hy", "id", "id-id", "it", "ja", "ka", "ko", "lv", "mk",
    "nb-no", "nl", "nl-be", "no-no", "pl", "pt", "pt-br", "ro-md", "ru", "ru-md", "sk", "sq", "sv", "sv-se", "ta",
    "ta-in", "th", "tr", "uk", "uz", "vi", "zh-cn", "zh-tw",
];

/// The state upstream keeps on the faker's cached `Locality`: the shuffled locales and the next index.
#[derive(Default)]
struct Shuffled {
    locales: Vec<&'static str>,
    index: usize,
}

/// Locales Datafaker supports.
#[derive(Clone, Copy)]
pub struct Locality<'a> {
    faker: &'a Faker,
}

impl<'a> Locality<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Locality { faker }
    }

    /// `allSupportedLocales()` (and the deprecated `allSupportedLocales(fileMasks)`).
    pub fn all_supported_locales(&self) -> &'static [&'static str] {
        LOCALES
    }

    /// `displayName()`: a random locale's display language and country in `Locale.ROOT` (the JDK's names, read by
    /// `scripts/jdk-tables.sh`), `English` for one with no language.
    pub fn display_name(&self) -> Result<String> {
        let index = self.faker.random().next_int_bounded(LOCALES.len() as i32)? as usize;
        let (_, language, country) = LOCALITY_DISPLAY_NAMES[index];
        let mut name = language.to_string();
        if !country.is_empty() {
            name = format!("{name} ({country})");
        }
        Ok(if name.is_empty() { ENGLISH_DISPLAY_LANGUAGE.to_string() } else { name })
    }

    /// `localeString()`: a random locale, with replacement.
    pub fn locale_string(&self) -> Result<String> {
        let random = self.faker.random();
        self.locale_string_with_random(random.random())
    }

    /// `localeStringWithRandom(random)`.
    pub fn locale_string_with_random(&self, random: &dyn RandomGenerator) -> Result<String> {
        Ok(LOCALES[random.next_int_bounded(LOCALES.len() as i32)? as usize].to_string())
    }

    /// `localeStringWithoutReplacement()`: rotates through a shuffled copy of the locales, kept per faker.
    pub fn locale_string_without_replacement(&self) -> Result<String> {
        let random = self.faker.random();
        self.locale_string_without_replacement_with(random.random())
    }

    /// `localeStringWithoutReplacement(random)`. As upstream, a shuffle starts over one locale before the end.
    pub fn locale_string_without_replacement_with(&self, random: &dyn RandomGenerator) -> Result<String> {
        self.without_replacement(Some(random))
    }

    /// `localeStringWithoutReplacement(null)`: a shuffle due starts over with the locales in order and throws at its
    /// first draw, leaving them unshuffled for the next call.
    fn without_replacement(&self, random: Option<&dyn RandomGenerator>) -> Result<String> {
        self.faker.with_provider_state(ProviderKind::Locality, |state: &mut Shuffled| {
            if state.locales.is_empty() || state.index >= state.locales.len() - 1 {
                state.locales = LOCALES.to_vec();
                state.index = 0;
                for i in (2..=state.locales.len()).rev() {
                    let Some(random) = random else { return Err(npe("Cannot invoke \"java.util.random.RandomGenerator.nextInt(int)\" because \"rnd\" is null")) };
                    let j = random.next_int_bounded(i as i32)? as usize;
                    state.locales.swap(i - 1, j);
                }
            }
            state.index += 1;
            Ok(state.locales[state.index - 1].to_string())
        })
    }
}

// The `RandomGenerator` and `Set` overloads take arguments no expression can give (`P::Opaque` never coerces): a
// template reaches them with `null` only.
fn locales() -> JValue {
    JValue::List(LOCALES.iter().map(|l| JValue::str(*l)).collect())
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("allSupportedLocales", &[crate::dispatch::P::Opaque("Set")], |_, _| Ok(locales())).accepting_null(),
    Method::new("allSupportedLocales", &[], |_, _| Ok(locales())),
    Method::new("displayName", &[], |f, _| s(Locality::new(f).display_name())),
    Method::new("localeString", &[], |f, _| s(Locality::new(f).locale_string())),
    Method::new("localeStringWithRandom", &[crate::dispatch::P::Opaque("RandomGenerator")], |_, _| {
        Err(npe("Cannot invoke \"java.util.random.RandomGenerator.nextInt(int)\" because \"random\" is null"))
    })
    .accepting_null(),
    Method::new("localeStringWithoutReplacement", &[], |f, _| s(Locality::new(f).locale_string_without_replacement())),
    Method::new(
        "localeStringWithoutReplacement",
        &[crate::dispatch::P::Opaque("RandomGenerator")],
        |f, _| s(Locality::new(f).without_replacement(None)),
    )
    .accepting_null(),
];
