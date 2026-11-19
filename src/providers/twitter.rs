//! `net.datafaker.providers.base.Twitter`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{java_str, npe, JValue, Method, P};
use crate::java::{Date, DateTimeFormatter, Instant};
use crate::{Error, Faker, Result};

const BASIC_STRING: &[u8] = b"QabR0cYdZ1efSg2hi3jNOPkTUM4VLlmXK5nJo6WIpHGqF7rEs8tDuC9vwBxAyz";

/// Tweets, user names and ids, links and creation times.
#[derive(Clone, Copy)]
pub struct Twitter<'a> {
    faker: &'a Faker,
}

/// `UUID.randomUUID().toString()`: from the operating system's random source, not from the faker's `Random`.
fn random_uuid() -> String {
    use std::io::Read;
    let mut bytes = [0u8; 16];
    let read = std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut bytes)).is_ok();
    if !read {
        use std::hash::{BuildHasher, Hasher};
        for chunk in bytes.chunks_mut(8) {
            let mut h = std::collections::hash_map::RandomState::new().build_hasher();
            h.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos()));
            chunk.copy_from_slice(&h.finish().to_be_bytes());
        }
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let msb = u64::from_be_bytes(bytes[..8].try_into().expect("8 bytes"));
    let lsb = u64::from_be_bytes(bytes[8..].try_into().expect("8 bytes"));
    super::internet::uuid_string(msb, lsb)
}

impl<'a> Twitter<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Twitter { faker }
    }

    /// `createdTime(forward, base, constraints)`: a `Date` between `base` and `constraints`.
    pub fn created_time(&self, forward: bool, base: Date, constraints: Date) -> Date {
        let random = self.faker.random();
        let time = base.get_time();
        if forward {
            Date::new(time.wrapping_add((random.next_double() * constraints.get_time().wrapping_sub(time) as f64) as i64))
        } else {
            Date::new(time.wrapping_sub((random.next_double() * time.wrapping_sub(constraints.get_time()) as f64) as i64))
        }
    }

    /// `twitterId(expectedLength)`: digits of a random UUID's hash interleaved with random digits and the current
    /// time (`yyyyMMddHHmmss`); not reproducible, as upstream.
    pub fn twitter_id(&self, expected_length: i32) -> Result<String> {
        self.twitter_id_with(expected_length, &random_uuid(), Instant::now())
    }

    /// `twitterId(expectedLength)` with the UUID and the clock given.
    pub fn twitter_id_with(&self, expected_length: i32, uuid: &str, now: Instant) -> Result<String> {
        let mut hash = crate::java::string_hash(uuid);
        if hash < 0 {
            hash = hash.wrapping_neg();
        }
        let id1: Vec<char> = hash.to_string().chars().collect();
        let formatter = DateTimeFormatter::of_pattern("yyyyMMddHHmmss")?.with_default_zone();
        let new_date = formatter.format_instant(now)?;
        let capacity = (expected_length as i64 - id1.len() as i64 - new_date.len() as i64).max(0);
        let random = self.faker.random();
        let mut result = String::new();
        for _ in 0..capacity {
            result.push_str(&random.next_int_bounded(10)?.to_string());
        }
        result.push_str(&new_date);
        let result: Vec<char> = result.chars().collect();
        let mut id = String::new();
        let (mut i, mut j) = (0, 0);
        while i < id1.len() || j < result.len() {
            if i < id1.len() {
                id.push(id1[i]);
                i += 1;
            }
            if j < result.len() {
                id.push(result[j]);
                j += 1;
            }
        }
        let start = random.next_int_bounded((id.len() as i32).wrapping_sub(expected_length).wrapping_add(1))?;
        let end = start as i64 + expected_length as i64;
        if start < 0 || expected_length < 0 || end > id.len() as i64 {
            return Err(Error::new("StringIndexOutOfBoundsException", format!("begin {start}, end {end}, length {}", id.len())));
        }
        Ok(id[start as usize..end as usize].to_string())
    }

    /// `text(keywords, sentenceMaxLength, wordMaxLength)`: random words, with each keyword inserted at a random
    /// position.
    pub fn text(&self, keywords: Option<&[String]>, sentence_max_length: i32, word_max_length: i32) -> Result<String> {
        let random = self.faker.random();
        let sentence_length = random.next_int_between(1, sentence_max_length)?;
        let mut text: Vec<String> = Vec::new();
        for _ in 0..sentence_length {
            let word_length = random.next_int_between(3, word_max_length)?;
            let mut word = String::new();
            for _ in 0..word_length {
                word.push(BASIC_STRING[random.next_int_bounded(BASIC_STRING.len() as i32)? as usize] as char);
            }
            text.push(word);
        }
        if let Some(keywords) = keywords {
            for keyword in keywords {
                let position = random.next_int_bounded(text.len() as i32)?;
                text.insert(position as usize, keyword.clone());
            }
        }
        Ok(text.join(" "))
    }

    /// `twitter.user_name`.
    pub fn user_name(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Twitter, "twitter.user_name")
    }

    /// `twitter.user_id`.
    pub fn user_id(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Twitter, "twitter.user_id")
    }

    /// `getLink(username, extraLength)`: `https://twitter.com/<username>/<extraLength random characters>`.
    pub fn get_link(&self, username: &str, extra_length: i32) -> Result<String> {
        let size = extra_length.wrapping_add(1);
        if size < 0 {
            return Err(Error::new("NegativeArraySizeException", size.to_string()));
        }
        if size == 0 {
            return Err(Error::new("ArrayIndexOutOfBoundsException", "Index 0 out of bounds for length 0"));
        }
        let random = self.faker.random();
        let mut res = String::from("/");
        for _ in 1..size {
            res.push(BASIC_STRING[random.next_int_bounded(BASIC_STRING.len() as i32)? as usize] as char);
        }
        Ok(format!("https://twitter.com/{username}{res}"))
    }
}

// `createdTime` takes `Date`s and `text` a `String[]`: expressions cannot reach them (no `(String)` constructor
// that coercion would use, beyond the unported `Date.parse`).
pub(crate) static METHODS: &[Method] = &[
    Method::new("twitterId", &[P::Int], |f, a| s(Twitter::new(f).twitter_id(a[0].as_int()))),
    Method::new("userName", &[], |f, _| s(Twitter::new(f).user_name())),
    Method::new("userId", &[], |f, _| s(Twitter::new(f).user_id())),
    Method::new("getLink", &[P::String, P::Int], |f, a| s(Twitter::new(f).get_link(java_str(&a[0]), a[1].as_int()))).accepting_null(),
    // Expressions cannot reach these (`String[]` and, here, `Date` are not coerced); a template only with nulls.
    Method::new("createdTime", &[P::Boolean, P::Opaque("Date"), P::Opaque("Date")], |_, _| {
        Err(npe("Cannot invoke \"java.util.Date.getTime()\" because \"base\" is null"))
    })
    .accepting_null(),
    Method::new("text", &[P::Opaque("String[]"), P::Int, P::Int], |f, a| {
        let keywords = match &a[0] {
            JValue::Null => None,
            keywords => Some(crate::dispatch::java_strings(keywords)?),
        };
        s(Twitter::new(f).text(keywords.as_deref(), a[1].as_int(), a[2].as_int()))
    })
    .accepting_null(),
];
