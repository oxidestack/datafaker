//! `net.datafaker.providers.base.Lorem`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method, P};
use crate::values::capitalize;
use crate::{Faker, Result};

/// Lorem ipsum words, sentences and paragraphs.
#[derive(Clone, Copy)]
pub struct Lorem<'a> {
    faker: &'a Faker,
}

impl<'a> Lorem<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Lorem { faker }
    }

    /// `character()`: one lower-case letter.
    pub fn character(&self) -> Result<char> {
        self.character_with(false)
    }

    /// `character(includeUppercase)`.
    pub fn character_with(&self, include_uppercase: bool) -> Result<char> {
        let n = if include_uppercase { 2 } else { 1 };
        Ok(self.characters_fixed(n, include_uppercase, false, false)?.chars().next().expect("one character"))
    }

    /// `characters()`: 255 lower-case letters.
    pub fn characters(&self) -> Result<String> {
        self.characters_fixed(255, false, false, false)
    }

    /// `characters(minimumLength, maximumLength)`: a length drawn first, then that many letters.
    pub fn characters_between(&self, minimum_length: i32, maximum_length: i32) -> Result<String> {
        let n = self.faker.random().next_int_between(minimum_length, maximum_length)?;
        self.characters_fixed(n, false, false, false)
    }

    /// `characters(minimumLength, maximumLength, includeUppercase, includeSpecial, includeDigit)`.
    pub fn characters_with(
        &self,
        minimum_length: i32,
        maximum_length: i32,
        include_uppercase: bool,
        include_special: bool,
        include_digit: bool,
    ) -> Result<String> {
        self.faker.text().text_with(minimum_length, maximum_length, include_uppercase, include_special, include_digit)
    }

    /// `characters(fixedNumberOfCharacters, includeUppercase, includeSpecial, includeDigit)`.
    pub fn characters_fixed(
        &self,
        fixed_number_of_characters: i32,
        include_uppercase: bool,
        include_special: bool,
        include_digit: bool,
    ) -> Result<String> {
        self.characters_with(
            fixed_number_of_characters,
            fixed_number_of_characters,
            include_uppercase,
            include_special,
            include_digit,
        )
    }

    /// `words(num)`.
    pub fn words(&self, num: i32) -> Result<Vec<String>> {
        (0..num.max(0)).map(|_| self.word()).collect()
    }

    /// `word()`: one of `lorem.words`.
    pub fn word(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Lorem, "lorem.words")
    }

    /// `lorem.supplemental`.
    pub fn supplemental(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Lorem, "lorem.supplemental")
    }

    /// `sentence()`: 3 words plus up to 5 more.
    pub fn sentence(&self) -> Result<String> {
        self.sentence_with(3, 6)
    }

    /// `sentence(wordCount)`.
    pub fn sentence_of(&self, word_count: i32) -> Result<String> {
        self.sentence_with(word_count, 6)
    }

    /// `sentence(wordCount, randomWordsToAdd)`.
    pub fn sentence_with(&self, word_count: i32, random_words_to_add: i32) -> Result<String> {
        let added = if random_words_to_add == 0 { 0 } else { self.faker.random().next_int_bounded(random_words_to_add)? };
        let total = word_count + added;
        let mut sb = String::new();
        if total > 0 {
            sb.push_str(&capitalize(&self.word()?));
        }
        for _ in 1..total.max(1) {
            sb.push(' ');
            sb.push_str(&self.word()?);
        }
        sb.push('.');
        Ok(sb)
    }

    /// `sentences(sentenceCount)`.
    pub fn sentences(&self, sentence_count: i32) -> Result<Vec<String>> {
        (0..sentence_count.max(0)).map(|_| self.sentence()).collect()
    }

    /// `paragraph(sentenceCount)`: that many sentences plus up to 2 more.
    pub fn paragraph_of(&self, sentence_count: i32) -> Result<String> {
        let n = sentence_count + self.faker.random().next_int_bounded(3)?;
        Ok(self.sentences(n)?.join(" "))
    }

    /// `paragraph()`: three sentences.
    pub fn paragraph(&self) -> Result<String> {
        self.paragraph_of(3)
    }

    /// `paragraphs(paragraphCount)`.
    pub fn paragraphs(&self, paragraph_count: i32) -> Result<Vec<String>> {
        (0..paragraph_count.max(0)).map(|_| self.paragraph()).collect()
    }

    /// `fixedString(numberOfLetters)`: sentences run together, cut to length.
    pub fn fixed_string(&self, number_of_letters: i32) -> Result<String> {
        if number_of_letters <= 0 {
            return Ok(String::new());
        }
        let mut builder: Vec<u16> = Vec::new();
        while builder.len() < number_of_letters as usize {
            builder.extend(self.sentence()?.encode_utf16());
        }
        Ok(String::from_utf16_lossy(&builder[..number_of_letters as usize]))
    }

    /// `maxLengthSentence(fixedLength)`.
    pub fn max_length_sentence(&self, fixed_length: i32) -> Result<String> {
        if fixed_length <= 0 {
            return Ok(String::new());
        }
        let n = fixed_length as usize;
        let mut sentence: Vec<u16> = self.sentence_of(fixed_length)?.encode_utf16().collect();
        while sentence[n - 1] == b' ' as u16 {
            sentence = self.sentence_of(fixed_length)?.encode_utf16().collect();
        }
        Ok(String::from_utf16_lossy(&sentence[..n]))
    }
}

fn l(f: &Faker) -> Lorem<'_> {
    f.lorem()
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

fn list(r: Result<Vec<String>>) -> Result<JValue> {
    r.map(JValue::from)
}

fn ch(r: Result<char>) -> Result<JValue> {
    r.map(|c| JValue::Char(c as u16))
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("character", &[], |f, _| ch(l(f).character())),
    Method::new("character", &[P::Boolean], |f, a| ch(l(f).character_with(a[0].as_bool()))),
    Method::new("characters", &[], |f, _| s(l(f).characters())),
    Method::new("characters", &[P::Boolean], |f, a| s(l(f).characters_fixed(255, a[0].as_bool(), false, false))),
    Method::new("characters", &[P::Int, P::Int], |f, a| s(l(f).characters_between(a[0].as_int(), a[1].as_int()))),
    Method::new("characters", &[P::Int, P::Int, P::Boolean], |f, a| {
        s(l(f).characters_with(a[0].as_int(), a[1].as_int(), a[2].as_bool(), false, false))
    }),
    Method::new("characters", &[P::Int, P::Int, P::Boolean, P::Boolean], |f, a| {
        s(l(f).characters_with(a[0].as_int(), a[1].as_int(), a[2].as_bool(), false, a[3].as_bool()))
    }),
    Method::new("characters", &[P::Int], |f, a| s(l(f).characters_fixed(a[0].as_int(), false, false, false))),
    Method::new("characters", &[P::Int, P::Boolean], |f, a| s(l(f).characters_fixed(a[0].as_int(), a[1].as_bool(), false, false))),
    Method::new("characters", &[P::Int, P::Int, P::Boolean, P::Boolean, P::Boolean], |f, a| {
        s(l(f).characters_with(a[0].as_int(), a[1].as_int(), a[2].as_bool(), a[3].as_bool(), a[4].as_bool()))
    }),
    Method::new("characters", &[P::Int, P::Boolean, P::Boolean], |f, a| {
        s(l(f).characters_fixed(a[0].as_int(), a[1].as_bool(), false, a[2].as_bool()))
    }),
    Method::new("characters", &[P::Int, P::Boolean, P::Boolean, P::Boolean], |f, a| {
        s(l(f).characters_fixed(a[0].as_int(), a[1].as_bool(), a[2].as_bool(), a[3].as_bool()))
    }),
    Method::new("words", &[P::Int], |f, a| list(l(f).words(a[0].as_int()))),
    Method::new("words", &[], |f, _| list(l(f).words(3))),
    Method::new("word", &[], |f, _| s(l(f).word())),
    Method::new("supplemental", &[], |f, _| s(l(f).supplemental())),
    Method::new("sentence", &[], |f, _| s(l(f).sentence())),
    Method::new("sentence", &[P::Int], |f, a| s(l(f).sentence_of(a[0].as_int()))),
    Method::new("sentence", &[P::Int, P::Int], |f, a| s(l(f).sentence_with(a[0].as_int(), a[1].as_int()))),
    Method::new("sentences", &[P::Int], |f, a| list(l(f).sentences(a[0].as_int()))),
    Method::new("paragraph", &[P::Int], |f, a| s(l(f).paragraph_of(a[0].as_int()))),
    Method::new("paragraph", &[], |f, _| s(l(f).paragraph())),
    Method::new("paragraphs", &[P::Int], |f, a| list(l(f).paragraphs(a[0].as_int()))),
    Method::new("fixedString", &[P::Int], |f, a| s(l(f).fixed_string(a[0].as_int()))),
    Method::new("maxLengthSentence", &[P::Int], |f, a| s(l(f).max_length_sentence(a[0].as_int()))),
];
