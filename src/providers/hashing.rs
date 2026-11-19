//! `net.datafaker.providers.base.Hashing`.

use super::s;
use crate::digests;
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Hex digests of random `lorem().characters()`.
#[derive(Clone, Copy)]
pub struct Hashing<'a> {
    faker: &'a Faker,
}

impl<'a> Hashing<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Hashing { faker }
    }

    /// The digest of `lorem().characters()`, as upstream feeds it: the first `characters.length()` (UTF-16 units)
    /// bytes of its UTF-8 encoding.
    fn generate(&self, digest: impl FnOnce(&[u8]) -> Vec<u8>) -> Result<String> {
        let characters = self.faker.lorem().characters()?;
        let len = characters.encode_utf16().count();
        Ok(digest(&characters.as_bytes()[..len]).iter().map(|b| format!("{b:02x}")).collect())
    }

    /// MD2, 32 hex digits.
    pub fn md2(&self) -> Result<String> {
        self.generate(|b| digests::md2(b).to_vec())
    }

    /// MD5, 32 hex digits.
    pub fn md5(&self) -> Result<String> {
        self.generate(|b| crate::md5::digest(b).to_vec())
    }

    /// SHA-1, 40 hex digits.
    pub fn sha1(&self) -> Result<String> {
        self.generate(|b| digests::sha1(b).to_vec())
    }

    /// SHA-384, 96 hex digits.
    pub fn sha384(&self) -> Result<String> {
        self.generate(|b| digests::sha384(b).to_vec())
    }

    /// SHA-256, 64 hex digits.
    pub fn sha256(&self) -> Result<String> {
        self.generate(|b| digests::sha256(b).to_vec())
    }

    /// SHA-512, 128 hex digits.
    pub fn sha512(&self) -> Result<String> {
        self.generate(|b| digests::sha512(b).to_vec())
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("md2", &[], |f, _| s(Hashing::new(f).md2())),
    Method::new("md5", &[], |f, _| s(Hashing::new(f).md5())),
    Method::new("sha1", &[], |f, _| s(Hashing::new(f).sha1())),
    Method::new("sha384", &[], |f, _| s(Hashing::new(f).sha384())),
    Method::new("sha256", &[], |f, _| s(Hashing::new(f).sha256())),
    Method::new("sha512", &[], |f, _| s(Hashing::new(f).sha512())),
];
