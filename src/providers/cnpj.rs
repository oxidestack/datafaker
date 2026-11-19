//! `net.datafaker.providers.base.CNPJ`.

use super::s;
use crate::dispatch::{Method, P};
use crate::idnumbers::pt_br;
use crate::{Faker, Result};

/// Brazilian company numbers (CNPJ).
#[derive(Clone, Copy)]
pub struct CNPJ<'a> {
    faker: &'a Faker,
}

impl<'a> CNPJ<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        CNPJ { faker }
    }

    /// A valid head-office CNPJ, formatted (`##.###.###/0001-##`).
    pub fn valid(&self) -> Result<String> {
        pt_br::cnpj(self.faker, true, true, false)
    }

    /// `valid(formatted)`.
    pub fn valid_with(&self, formatted: bool) -> Result<String> {
        pt_br::cnpj(self.faker, formatted, true, false)
    }

    /// `valid(formatted, multiBranch)`: any branch number when `multi_branch`.
    pub fn valid_branch(&self, formatted: bool, multi_branch: bool) -> Result<String> {
        pt_br::cnpj(self.faker, formatted, true, multi_branch)
    }

    /// An invalid CNPJ, formatted.
    pub fn invalid(&self) -> Result<String> {
        pt_br::cnpj(self.faker, true, false, false)
    }

    /// `invalid(formatted)`.
    pub fn invalid_with(&self, formatted: bool) -> Result<String> {
        pt_br::cnpj(self.faker, formatted, false, false)
    }

    /// `invalid(formatted, multiBranch)`.
    pub fn invalid_branch(&self, formatted: bool, multi_branch: bool) -> Result<String> {
        pt_br::cnpj(self.faker, formatted, false, multi_branch)
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("valid", &[], |f, _| s(CNPJ::new(f).valid())),
    Method::new("valid", &[P::Boolean], |f, a| s(CNPJ::new(f).valid_with(a[0].as_bool()))),
    Method::new("valid", &[P::Boolean, P::Boolean], |f, a| s(CNPJ::new(f).valid_branch(a[0].as_bool(), a[1].as_bool()))),
    Method::new("invalid", &[], |f, _| s(CNPJ::new(f).invalid())),
    Method::new("invalid", &[P::Boolean], |f, a| s(CNPJ::new(f).invalid_with(a[0].as_bool()))),
    Method::new("invalid", &[P::Boolean, P::Boolean], |f, a| s(CNPJ::new(f).invalid_branch(a[0].as_bool(), a[1].as_bool()))),
];
