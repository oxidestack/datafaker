//! `net.datafaker.providers.base.Text`.

use crate::dispatch::{JValue, Method, P};
use crate::java;
use crate::{Error, Faker, Result};

/// The lowercase English letters.
pub const EN_LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
/// The uppercase English letters.
pub const EN_UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
/// The decimal digits.
pub const DIGITS: &str = "0123456789";
/// The special characters a password draws from by default.
pub const DEFAULT_SPECIAL: &str = "!@#$%^&*";

/// Random text from symbol sets, with a minimum number of symbols from each required set.
#[derive(Clone, Copy)]
pub struct Text<'a> {
    faker: &'a Faker,
}

/// `Text.TextRuleConfig`: the symbol sets (in `HashMap` order, as upstream builds them) and how many symbols each
/// must contribute.
#[derive(Debug, Clone)]
pub struct TextRuleConfig {
    text_keys: Vec<Vec<u16>>,
    required: Vec<i32>,
    fixed_number_of_characters: i32,
    number_of_required_symbols: i32,
}

/// `Text.TextSymbolsBuilder`.
#[derive(Debug, Clone, Default)]
pub struct TextSymbolsBuilder {
    length: i32,
    map: Vec<(String, i32)>,
    throw_if_length_small: bool,
}

impl TextSymbolsBuilder {
    /// `TextSymbolsBuilder.builder()`: an empty builder.
    pub fn builder() -> Self {
        Self::default()
    }

    /// `with(listOfSymbols, times)`: a symbol set that must appear at least `times` times.
    pub fn with_times(mut self, list_of_symbols: &str, times: i32) -> Result<Self> {
        if times < 0 {
            return Err(Error::illegal_argument(format!("times should be non-negative: {times}")));
        }
        match self.map.iter_mut().find(|(k, _)| k == list_of_symbols) {
            Some(entry) => entry.1 = times,
            None => self.map.push((list_of_symbols.to_string(), times)),
        }
        Ok(self)
    }

    /// `with(listOfSymbols)`.
    pub fn with(self, list_of_symbols: &str) -> Self {
        self.with_times(list_of_symbols, 0).expect("0 is non-negative")
    }

    /// `len(len)`: the length of the text.
    pub fn len(mut self, len: i32) -> Self {
        self.length = len;
        self
    }

    /// `throwIfLengthSmall(throw)`: whether [`Self::build`] fails when the required symbols do not fit the length.
    pub fn throw_if_length_small(mut self, throw: bool) -> Self {
        self.throw_if_length_small = throw;
        self
    }

    /// `build()`.
    pub fn build(self) -> Result<TextRuleConfig> {
        let min_size: i32 = self.map.iter().map(|(_, t)| *t).filter(|t| *t > 0).sum();
        if min_size > self.length && self.throw_if_length_small {
            return Err(Error::illegal_argument(format!(
                "Min length ({}) should be not smaller than number of required characters ({min_size})",
                self.length
            )));
        }
        let hashes: Vec<i32> = self.map.iter().map(|(k, _)| java::string_hash(k)).collect();
        let order = java::hash_map_order(&hashes);
        Ok(TextRuleConfig {
            text_keys: order.iter().map(|&i| self.map[i].0.encode_utf16().collect()).collect(),
            required: order.iter().map(|&i| self.map[i].1).collect(),
            fixed_number_of_characters: self.length,
            number_of_required_symbols: min_size,
        })
    }
}

impl<'a> Text<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Text { faker }
    }

    /// `character()`: a lower- or upper-case letter.
    pub fn character(&self) -> Result<char> {
        Ok(self.text_with(1, 1, true, false, false)?.chars().next().expect("one character"))
    }

    /// `uppercaseCharacter()`.
    pub fn uppercase_character(&self) -> Result<char> {
        let t = self.text_with(1, 1, false, false, false)?;
        let up = java::string_to_upper(&t, self.faker.locale().language());
        Ok(up.chars().next().expect("one character"))
    }

    /// `lowercaseCharacter()`.
    pub fn lowercase_character(&self) -> Result<char> {
        Ok(self.text_with(1, 1, false, false, false)?.chars().next().expect("one character"))
    }

    /// `text()`: 20 to 80 lower-case letters.
    pub fn text(&self) -> Result<String> {
        self.text_with(20, 80, false, false, false)
    }

    /// `text(includeDigit)`.
    pub fn text_digits(&self, include_digit: bool) -> Result<String> {
        self.text_with(20, 80, false, false, include_digit)
    }

    /// `text(length)`.
    pub fn text_of_length(&self, length: i32) -> Result<String> {
        self.text_with(length, length, false, false, false)
    }

    /// `text(minimumLength, maximumLength)`.
    pub fn text_between(&self, minimum_length: i32, maximum_length: i32) -> Result<String> {
        self.text_with(minimum_length, maximum_length, false, false, false)
    }

    /// `text(minimumLength, maximumLength, includeUppercase, includeSpecial, includeDigit)`: a length drawn from
    /// the range, then one symbol of each included kind at least.
    pub fn text_with(
        &self,
        minimum_length: i32,
        maximum_length: i32,
        include_uppercase: bool,
        include_special: bool,
        include_digit: bool,
    ) -> Result<String> {
        if minimum_length > maximum_length {
            return Err(Error::illegal_argument(format!(
                "Min length ({minimum_length}) should not be greater than max length ({maximum_length})"
            )));
        }
        let required = i32::from(include_uppercase) + i32::from(include_special) + i32::from(include_digit);
        let min_length = minimum_length.max(required);
        if min_length > maximum_length {
            return Err(Error::illegal_argument(format!(
                "Minimum number of required characters ({min_length}) should not be greater than max length ({maximum_length})"
            )));
        }
        let len = self.faker.number().number_between(min_length, maximum_length.wrapping_add(1));
        let mut builder = TextSymbolsBuilder::builder().with(EN_LOWERCASE);
        if include_uppercase {
            builder = builder.with_times(EN_UPPERCASE, 1)?;
        }
        if include_special {
            builder = builder.with_times(DEFAULT_SPECIAL, 1)?;
        }
        if include_digit {
            builder = builder.with_times(DIGITS, 1)?;
        }
        let config = builder.len(len).throw_if_length_small(true).build()?;
        self.text_from(&config)
    }

    /// `text(TextRuleConfig)`.
    pub fn text_from(&self, config: &TextRuleConfig) -> Result<String> {
        let random = self.faker.random();
        let fixed = config.fixed_number_of_characters.max(config.number_of_required_symbols).max(0) as usize;
        let required_symbols = config.number_of_required_symbols as usize;
        let keys = &config.text_keys;
        let max_diff_symbols: usize = keys.iter().map(Vec::len).sum();
        let mut buffer: Vec<u16> = vec![0; fixed];
        let mut required = config.required.clone();
        let mut number_of_required = 0usize;
        let mut idx = 0;
        if max_diff_symbols <= 256 {
            let bytes = random.next_random_bytes(2 * fixed);
            // `((char) b) % n`: the byte sign-extends, then narrows to 16 bits.
            let at = |b: i8, n: usize| (b as i32 as u16) as usize % n;
            let mut k = 0;
            while idx < buffer.len() {
                if required_symbols > number_of_required && required_symbols - number_of_required == buffer.len() - idx {
                    for j in 0..keys.len() {
                        while required[j] > 0 {
                            buffer[idx] = keys[j][at(bytes[k], keys[j].len())];
                            idx += 1;
                            k += 1;
                            number_of_required += 1;
                            required[j] -= 1;
                        }
                        if idx == buffer.len() {
                            break;
                        }
                    }
                } else {
                    let index = at(bytes[k], keys.len());
                    k += 1;
                    if required[index] > 0 {
                        number_of_required += 1;
                        required[index] -= 1;
                    }
                    buffer[idx] = keys[index][at(bytes[k], keys[index].len())];
                    idx += 1;
                    k += 1;
                }
            }
            return Ok(String::from_utf16_lossy(&buffer));
        }
        while idx < buffer.len() {
            if required_symbols > number_of_required && required_symbols - number_of_required == buffer.len() - idx {
                for j in 0..keys.len() {
                    while required[j] > 0 {
                        buffer[idx] = keys[j][random.next_int_bounded(keys[j].len() as i32)? as usize];
                        idx += 1;
                        number_of_required += 1;
                        required[j] -= 1;
                    }
                    if idx == buffer.len() {
                        break;
                    }
                }
            } else {
                let index = random.next_int_bounded(required.len() as i32)? as usize;
                if required[index] > 0 {
                    number_of_required += 1;
                    required[index] -= 1;
                }
                buffer[idx] = keys[index][random.next_int_bounded(keys[index].len() as i32)? as usize];
                idx += 1;
            }
        }
        Ok(String::from_utf16_lossy(&buffer))
    }
}

fn t(f: &Faker) -> Text<'_> {
    f.text()
}

fn ch(r: Result<char>) -> Result<JValue> {
    r.map(|c| JValue::Char(c as u16))
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("character", &[], |f, _| ch(t(f).character())),
    Method::new("uppercaseCharacter", &[], |f, _| ch(t(f).uppercase_character())),
    Method::new("lowercaseCharacter", &[], |f, _| ch(t(f).lowercase_character())),
    Method::new("text", &[], |f, _| t(f).text().map(JValue::Str)),
    Method::new("text", &[P::Boolean], |f, a| t(f).text_digits(a[0].as_bool()).map(JValue::Str)),
    Method::new("text", &[P::Int], |f, a| t(f).text_of_length(a[0].as_int()).map(JValue::Str)),
    Method::new("text", &[P::Int, P::Int], |f, a| t(f).text_between(a[0].as_int(), a[1].as_int()).map(JValue::Str)),
    Method::new("text", &[P::Int, P::Int, P::Boolean], |f, a| {
        t(f).text_with(a[0].as_int(), a[1].as_int(), a[2].as_bool(), false, false).map(JValue::Str)
    }),
    Method::new("text", &[P::Int, P::Int, P::Boolean, P::Boolean], |f, a| {
        t(f).text_with(a[0].as_int(), a[1].as_int(), a[2].as_bool(), a[3].as_bool(), false).map(JValue::Str)
    }),
    Method::new("text", &[P::Int, P::Int, P::Boolean, P::Boolean, P::Boolean], |f, a| {
        t(f).text_with(a[0].as_int(), a[1].as_int(), a[2].as_bool(), a[3].as_bool(), a[4].as_bool()).map(JValue::Str)
    }),
    // Reached only with a null config.
    Method::new("text", &[P::Opaque("TextRuleConfig")], |_, _| {
        Err(crate::dispatch::npe(
            "Cannot invoke \"net.datafaker.providers.base.Text$TextRuleConfig.getFixedNumberOfCharacters()\" because \"textRuleConfig\" is null",
        ))
    })
    .accepting_null(),
];
