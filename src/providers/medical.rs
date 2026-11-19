//! `net.datafaker.providers.base.Medical` (deprecated upstream since 2.3.0, still registered).

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Medicine, diseases, hospitals and ICD-10 codes, from `medical.*`.
#[derive(Clone, Copy)]
pub struct Medical<'a> {
    faker: &'a Faker,
}

impl<'a> Medical<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Medical { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Medical, key)
    }

    /// `medical.medicine_name`.
    pub fn medicine_name(&self) -> Result<String> {
        self.resolve("medical.medicine_name")
    }

    /// `medical.disease_name`.
    pub fn disease_name(&self) -> Result<String> {
        self.resolve("medical.disease_name")
    }

    /// `medical.hospital_name`.
    pub fn hospital_name(&self) -> Result<String> {
        self.resolve("medical.hospital_name")
    }

    /// `medical.symptoms`.
    pub fn symptoms(&self) -> Result<String> {
        self.resolve("medical.symptoms")
    }

    /// A string matching the regex at `medical.diagnosis_code.icd10`.
    pub fn diagnosis_code(&self) -> Result<String> {
        self.faker.regexify(&self.resolve("medical.diagnosis_code.icd10")?)
    }

    /// A string matching the regex at `medical.procedure_code.icd10`.
    pub fn procedure_code(&self) -> Result<String> {
        self.faker.regexify(&self.resolve("medical.procedure_code.icd10")?)
    }

    /// `medical.medical_professions`.
    pub fn medical_profession(&self) -> Result<String> {
        self.resolve("medical.medical_professions")
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("medicineName", &[], |f, _| s(Medical::new(f).medicine_name())),
    Method::new("diseaseName", &[], |f, _| s(Medical::new(f).disease_name())),
    Method::new("hospitalName", &[], |f, _| s(Medical::new(f).hospital_name())),
    Method::new("symptoms", &[], |f, _| s(Medical::new(f).symptoms())),
    Method::new("diagnosisCode", &[], |f, _| s(Medical::new(f).diagnosis_code())),
    Method::new("procedureCode", &[], |f, _| s(Medical::new(f).procedure_code())),
    Method::new("medicalProfession", &[], |f, _| s(Medical::new(f).medical_profession())),
];
