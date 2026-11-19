//! `net.datafaker.providers.healthcare.Disease`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

provider_enum! {
    /// `Disease.DiseaseType`: a speciality and its YAML key. No public method takes one.
    DiseaseType, "DiseaseType" {
        InternalDiseaseKey = "INTERNAL_DISEASE_KEY" => "healthcare.disease.internal_disease",
        NeurologicalDiseaseKey = "NEUROLOGICAL_DISEASE_KEY" => "healthcare.disease.neurology",
        SurgicalDiseaseKey = "SURGICAL_DISEASE_KEY" => "healthcare.disease.surgery",
        PaediatricDiseaseKey = "PAEDIATRIC_DISEASE_KEY" => "healthcare.disease.paediatrics",
        GynecologyAndObstetricsDiseaseKey = "GYNECOLOGY_AND_OBSTETRICS_DISEASE_KEY" => "healthcare.disease.gynecology_and_obstetrics",
        OphthalmologyAndOtorhinolaryngologyDiseaseKey = "OPHTHALMOLOGY_AND_OTORHINOLARYNGOLOGY_DISEASE_KEY" => "healthcare.disease.ophthalmology_and_otorhinolaryngology",
        DermatologyDiseaseKey = "DERMATOLOGY_DISEASE_KEY" => "healthcare.disease.dermatology",
    }
}

/// Diseases, from `healthcare.disease.*`.
#[derive(Clone, Copy)]
pub struct Disease<'a> {
    faker: &'a Faker,
}

impl<'a> Disease<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Disease { faker }
    }

    fn of(&self, kind: DiseaseType) -> Result<String> {
        resolve(self.faker, ProviderKind::Disease, kind.yaml_key())
    }

    /// An ICD-10 code: `regexify` of `healthcare.disease.icd10`.
    pub fn icd10(&self) -> Result<String> {
        let regex = resolve(self.faker, ProviderKind::Disease, "healthcare.disease.icd10")?;
        self.faker.regexify(&regex)
    }

    /// A disease of a random [`DiseaseType`] (`options().option(DiseaseType.class)`).
    pub fn any_disease(&self) -> Result<String> {
        self.of(DiseaseType::VALUES[self.faker.random().next_enum(DiseaseType::VALUES.len())?])
    }

    /// `healthcare.disease.internal_disease`.
    pub fn internal_disease(&self) -> Result<String> {
        self.of(DiseaseType::InternalDiseaseKey)
    }

    /// `healthcare.disease.neurology`.
    pub fn neurology(&self) -> Result<String> {
        self.of(DiseaseType::NeurologicalDiseaseKey)
    }

    /// `healthcare.disease.surgery`.
    pub fn surgery(&self) -> Result<String> {
        self.of(DiseaseType::SurgicalDiseaseKey)
    }

    /// `healthcare.disease.paediatrics`.
    pub fn paediatrics(&self) -> Result<String> {
        self.of(DiseaseType::PaediatricDiseaseKey)
    }

    /// `healthcare.disease.gynecology_and_obstetrics`.
    pub fn gynecology_and_obstetrics(&self) -> Result<String> {
        self.of(DiseaseType::GynecologyAndObstetricsDiseaseKey)
    }

    /// `healthcare.disease.ophthalmology_and_otorhinolaryngology`.
    pub fn ophthalmology_and_otorhinolaryngology(&self) -> Result<String> {
        self.of(DiseaseType::OphthalmologyAndOtorhinolaryngologyDiseaseKey)
    }

    /// `healthcare.disease.dermatology`.
    pub fn dermatology(&self) -> Result<String> {
        self.of(DiseaseType::DermatologyDiseaseKey)
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("icd10", &[], |f, _| s(Disease::new(f).icd10())),
    Method::new("anyDisease", &[], |f, _| s(Disease::new(f).any_disease())),
    Method::new("internalDisease", &[], |f, _| s(Disease::new(f).internal_disease())),
    Method::new("neurology", &[], |f, _| s(Disease::new(f).neurology())),
    Method::new("surgery", &[], |f, _| s(Disease::new(f).surgery())),
    Method::new("paediatrics", &[], |f, _| s(Disease::new(f).paediatrics())),
    Method::new("gynecologyAndObstetrics", &[], |f, _| s(Disease::new(f).gynecology_and_obstetrics())),
    Method::new("ophthalmologyAndOtorhinolaryngology", &[], |f, _| {
        s(Disease::new(f).ophthalmology_and_otorhinolaryngology())
    }),
    Method::new("dermatology", &[], |f, _| s(Disease::new(f).dermatology())),
];
