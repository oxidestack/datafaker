//! `net.datafaker.providers.base.CPF`.

use super::s;
use crate::dispatch::{Method, P};
use crate::idnumbers::pt_br;
use crate::{Faker, Result};

/// Brazilian individual taxpayer numbers (CPF).
#[derive(Clone, Copy)]
pub struct CPF<'a> {
    faker: &'a Faker,
}

impl<'a> CPF<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        CPF { faker }
    }

    /// A valid CPF, formatted (`###.###.###-##`).
    pub fn valid(&self) -> Result<String> {
        pt_br::cpf(self.faker, true, true)
    }

    /// `valid(formatted)`.
    pub fn valid_with(&self, formatted: bool) -> Result<String> {
        pt_br::cpf(self.faker, formatted, true)
    }

    /// An invalid CPF, formatted.
    pub fn invalid(&self) -> Result<String> {
        pt_br::cpf(self.faker, true, false)
    }

    /// `invalid(formatted)`.
    pub fn invalid_with(&self, formatted: bool) -> Result<String> {
        pt_br::cpf(self.faker, formatted, false)
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("valid", &[], |f, _| s(CPF::new(f).valid())),
    Method::new("valid", &[P::Boolean], |f, a| s(CPF::new(f).valid_with(a[0].as_bool()))),
    Method::new("invalid", &[], |f, _| s(CPF::new(f).invalid())),
    Method::new("invalid", &[P::Boolean], |f, a| s(CPF::new(f).invalid_with(a[0].as_bool()))),
];
