//! `java.util.Locale` as far as Datafaker reads it (language and country), and `FakerContext`'s normalisation and
//! locale chain.

use std::fmt;

/// A locale: a lower-case language and an upper-case country, either possibly empty, as `new Locale(language,
/// country)` stores them.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale {
    language: String,
    country: String,
}

impl Locale {
    /// `new Locale(language, country)`.
    pub fn new(language: &str, country: &str) -> Locale {
        Locale { language: language.to_ascii_lowercase(), country: country.to_ascii_uppercase() }
    }

    /// `new Locale(language)`.
    pub fn language_only(language: &str) -> Locale {
        Locale::new(language, "")
    }

    /// `Locale.ENGLISH`, Datafaker's default.
    pub fn english() -> Locale {
        Locale::new("en", "")
    }

    /// `Locale.forLanguageTag(tag)` for the `language[-COUNTRY]` tags Datafaker meets.
    pub fn for_language_tag(tag: &str) -> Locale {
        let mut parts = tag.split(['-', '_']);
        let language = parts.next().unwrap_or("");
        let country = parts.next().filter(|c| c.len() == 2 || c.chars().all(|ch| ch.is_ascii_digit())).unwrap_or("");
        Locale::new(language, country)
    }

    /// `getLanguage()`: the lower-case language, possibly empty.
    pub fn language(&self) -> &str {
        &self.language
    }

    /// `getCountry()`: the upper-case country, possibly empty.
    pub fn country(&self) -> &str {
        &self.country
    }

    /// Whether this is `FakeValuesService.DEFAULT_LOCALE` (`Locale.ENGLISH`).
    pub fn is_default(&self) -> bool {
        self.language == "en" && self.country.is_empty()
    }

    /// `FakerContext.normalizeLocale`: `new Locale("pt-br")` becomes `pt_BR`.
    pub(crate) fn normalize(&self) -> Locale {
        if self.country.is_empty() {
            let parts: Vec<&str> = self.language.split(['-', '_']).collect();
            if parts.len() == 1 { Locale::for_language_tag(parts[0]) } else { Locale::new(parts[0], parts[1]) }
        } else {
            self.clone()
        }
    }

    /// `FakerContext.localeChain()`: `en_US` → `[en_US, _US, en]`, `de` → `[de, _DE, en]`,
    /// `pt_BR` → `[pt_BR, pt, _BR, en]`.
    pub fn chain(&self) -> Vec<Locale> {
        let locale = self.normalize();
        if locale.is_default() {
            return vec![locale];
        }
        let mut chain = vec![locale.clone()];
        if locale.country.is_empty() {
            chain.push(Locale::new("", default_country_for_language(&locale.language)));
        } else {
            if locale.language != "en" {
                chain.push(Locale::language_only(&locale.language));
            }
            chain.push(Locale::new("", &locale.country));
        }
        chain.push(Locale::english());
        chain
    }
}

fn default_country_for_language(language: &str) -> &str {
    match language {
        "be" => "BY",
        "cs" => "CZ",
        "da" => "DK",
        "el" => "GR",
        "et" => "EE",
        "he" => "IL",
        "hy" => "AM",
        "ja" => "JP",
        "ka" => "GE",
        "ko" => "KR",
        "nb" => "NO",
        "sq" => "AL",
        "sv" => "SE",
        "ta" => "IN",
        "uk" => "UA",
        "vi" => "VN",
        "zh" => "CN",
        other => other,
    }
}

impl fmt::Display for Locale {
    /// `Locale.toString()`: `en`, `en_US`, `_US`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.country.is_empty() { f.write_str(&self.language) } else { write!(f, "{}_{}", self.language, self.country) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chains() {
        let names = |l: Locale| l.chain().iter().map(ToString::to_string).collect::<Vec<_>>();
        assert_eq!(names(Locale::english()), ["en"]);
        assert_eq!(names(Locale::new("en", "US")), ["en_US", "_US", "en"]);
        assert_eq!(names(Locale::language_only("de")), ["de", "_DE", "en"]);
        assert_eq!(names(Locale::language_only("pt-br")), ["pt_BR", "pt", "_BR", "en"]);
        assert_eq!(names(Locale::language_only("uk")), ["uk", "_UA", "en"]);
    }
}
