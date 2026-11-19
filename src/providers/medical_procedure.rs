//! `net.datafaker.providers.healthcare.MedicalProcedure`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Medical procedures, from `healthcare.medical_procedure.*`.
#[derive(Clone, Copy)]
pub struct MedicalProcedure<'a> {
    faker: &'a Faker,
}

impl<'a> MedicalProcedure<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        MedicalProcedure { faker }
    }

    /// An ICD-10-PCS code: `regexify` of `healthcare.medical_procedure.icd10`.
    pub fn icd10(&self) -> Result<String> {
        let regex = resolve(self.faker, ProviderKind::MedicalProcedure, "healthcare.medical_procedure.icd10")?;
        self.faker.regexify(&regex)
    }
}

pub(crate) static METHODS: &[Method] = &[Method::new("icd10", &[], |f, _| s(MedicalProcedure::new(f).icd10()))];
