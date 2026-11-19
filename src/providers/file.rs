//! `net.datafaker.providers.base.File`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{JValue, Method, P};
use crate::java::string_to_lower;
use crate::{Faker, Result};

/// File extensions, MIME types and file names, from `file.*`.
#[derive(Clone, Copy)]
pub struct File<'a> {
    faker: &'a Faker,
}

impl<'a> File<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        File { faker }
    }

    /// `file.extension`.
    pub fn extension(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::File, "file.extension")
    }

    /// `file.mime_type`.
    pub fn mime_type(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::File, "file.mime_type")
    }

    /// `fileName()`: `<slug>/<word>.<extension>`.
    pub fn file_name(&self) -> Result<String> {
        self.file_name_with(None, None, None, None)
    }

    /// `fileName(dirOrNull, nameOrNull, extensionOrNull, separatorOrNull)`: each `None` is generated (the
    /// separator is the platform's, `/` here as on Unix).
    pub fn file_name_with(&self, dir: Option<&str>, name: Option<&str>, extension: Option<&str>, separator: Option<&str>) -> Result<String> {
        let sep = separator.unwrap_or("/");
        let dir = match dir {
            Some(d) => d.to_string(),
            None => self.faker.internet().slug()?,
        };
        let name = match name {
            Some(n) => n.to_string(),
            None => string_to_lower(&self.faker.lorem().word()?, self.faker.locale().language()),
        };
        let ext = match extension {
            Some(e) => e.to_string(),
            None => self.extension()?,
        };
        Ok(format!("{dir}{sep}{name}.{ext}"))
    }
}

fn opt(v: &JValue) -> Option<&str> {
    match v {
        JValue::Str(s) => Some(s),
        _ => None,
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("extension", &[], |f, _| s(File::new(f).extension())),
    Method::new("mimeType", &[], |f, _| s(File::new(f).mime_type())),
    Method::new("fileName", &[], |f, _| s(File::new(f).file_name())),
    Method::new("fileName", &[P::String, P::String, P::String, P::String], |f, a| {
        s(File::new(f).file_name_with(opt(&a[0]), opt(&a[1]), opt(&a[2]), opt(&a[3])))
    })
    .accepting_null(),
];
