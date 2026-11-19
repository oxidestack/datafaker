//! `net.datafaker.providers.base.IdNumber`.

use super::s;
use crate::dispatch::{Method, P};
use crate::idnumbers::{
    AmericanIdNumber, ChineseIdNumber, IdNumberGenerator, IdNumberRequest, MexicanIdNumber, PersonIdNumber, PolishGender, PolishIdNumber,
    PortugueseIdNumber, SingaporeIdNumber, SingaporeType, SouthAfricanIdNumber, SouthKoreanIdNumber, SwedenIdNumber, generator_for,
};
use crate::java::LocalDate;
use crate::{Error, Faker, Result};

/// National identity numbers: the generator of the faker's country (its locale's country, `SE` for `sv_SE`), else
/// the locale's `id_number` data; and the per-country methods upstream keeps (most of them deprecated there).
#[derive(Clone, Copy)]
pub struct IdNumber<'a> {
    faker: &'a Faker,
}

impl<'a> IdNumber<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        IdNumber { faker }
    }

    fn country_provider(&self) -> Option<&'static dyn IdNumberGenerator> {
        generator_for(self.faker.locale().country())
    }

    /// `valid()`: a valid id number of the faker's country, or `numerify` of the locale's `id_number.valid`.
    pub fn valid(&self) -> Result<String> {
        match self.country_provider() {
            Some(p) => p.generate_valid(self.faker),
            None => Ok(self.faker.numerify(&self.faker.resolve("id_number.valid")?)),
        }
    }

    /// `invalid()`: an invalid id number of the faker's country, or `numerify` of the locale's `id_number.invalid`.
    pub fn invalid(&self) -> Result<String> {
        match self.country_provider() {
            Some(p) => p.generate_invalid(self.faker),
            None => Ok(self.faker.numerify(&self.faker.resolve("id_number.invalid")?)),
        }
    }

    /// `valid(request)`: a valid id number with its birthday and gender, for the ages and gender asked for. A country
    /// without a generator is an `IllegalArgumentException`.
    pub fn valid_for(&self, request: &IdNumberRequest) -> Result<PersonIdNumber> {
        match self.country_provider() {
            Some(p) => p.generate_valid_for(self.faker, request),
            None => {
                Err(Error::illegal_argument(format!("ID Number generation not supported for country '{}'", self.faker.locale().country())))
            }
        }
    }

    /// `ssnValid()`: a valid United States social security number.
    pub fn ssn_valid(&self) -> Result<String> {
        AmericanIdNumber.generate_valid(self.faker)
    }

    /// `validSvSeSsn()` (deprecated upstream).
    pub fn valid_sv_se_ssn(&self) -> Result<String> {
        SwedenIdNumber.generate_valid(self.faker)
    }

    /// `invalidSvSeSsn()` (deprecated upstream).
    pub fn invalid_sv_se_ssn(&self) -> Result<String> {
        SwedenIdNumber.generate_invalid(self.faker)
    }

    /// `validEnZaSsn()` (deprecated upstream).
    pub fn valid_en_za_ssn(&self) -> Result<String> {
        SouthAfricanIdNumber.get_valid_ssn(self.faker)
    }

    /// `inValidEnZaSsn()` (deprecated upstream).
    pub fn in_valid_en_za_ssn(&self) -> Result<String> {
        SouthAfricanIdNumber.get_in_valid_ssn(self.faker)
    }

    /// `singaporeanFin()`: a foreigner's number of the 2000s (`G…`).
    pub fn singaporean_fin(&self) -> Result<String> {
        SingaporeIdNumber::get_valid_fin(self.faker, SingaporeType::ForeignerTwentyFirstCentury)
    }

    /// `singaporeanFinBefore2000()`: a foreigner's number of the 1900s (`F…`).
    pub fn singaporean_fin_before2000(&self) -> Result<String> {
        SingaporeIdNumber::get_valid_fin(self.faker, SingaporeType::ForeignerTwentiethCentury)
    }

    /// `singaporeanUin()`: a citizen's number of the 2000s (`T…`).
    pub fn singaporean_uin(&self) -> Result<String> {
        SingaporeIdNumber::get_valid_fin(self.faker, SingaporeType::SingaporeanTwentyFirstCentury)
    }

    /// `singaporeanUinBefore2000()`: a citizen's number of the 1900s (`S…`).
    pub fn singaporean_uin_before2000(&self) -> Result<String> {
        SingaporeIdNumber::get_valid_fin(self.faker, SingaporeType::SingaporeanTwentiethCentury)
    }

    /// `validZhCNSsn()` (deprecated upstream).
    pub fn valid_zh_cn_ssn(&self) -> Result<String> {
        ChineseIdNumber.generate_valid(self.faker)
    }

    /// `validPtNif()` (deprecated upstream).
    pub fn valid_pt_nif(&self) -> Result<String> {
        PortugueseIdNumber.generate_valid(self.faker)
    }

    /// `invalidPtNif()` (deprecated upstream).
    pub fn invalid_pt_nif(&self) -> Result<String> {
        PortugueseIdNumber.generate_invalid(self.faker)
    }

    /// `validEsMXSsn()` (deprecated upstream).
    pub fn valid_es_mx_ssn(&self) -> Result<String> {
        MexicanIdNumber.get(self.faker)
    }

    /// `invalidEsMXSsn()` (deprecated upstream).
    pub fn invalid_es_mx_ssn(&self) -> Result<String> {
        MexicanIdNumber.generate_invalid(self.faker)
    }

    /// `peselNumber()` (deprecated upstream): a PESEL of a person 0 to 100 years old of any gender.
    pub fn pesel_number(&self) -> Result<String> {
        let birth_date = self.faker.time_and_date().birthday_between(0, 100)?;
        self.pesel_number_for(birth_date, Some(PolishGender::Any))
    }

    /// `peselNumber(birthDate, gender)` (deprecated upstream); `None` is any gender.
    pub fn pesel_number_for(&self, birth_date: LocalDate, gender: Option<PolishGender>) -> Result<String> {
        PolishIdNumber.get(self.faker, birth_date, gender)
    }

    /// `validKoKrRrn()` (deprecated upstream).
    pub fn valid_ko_kr_rrn(&self) -> Result<String> {
        SouthKoreanIdNumber.get_valid_rrn(self.faker)
    }

    /// `validGeIDNumber()` (deprecated upstream): eleven random digits.
    pub fn valid_ge_id_number(&self) -> Result<String> {
        Ok(self.faker.numerify("###########"))
    }
}

/// The methods expressions reach. `valid(IdNumberRequest)` and `peselNumber(LocalDate, Gender)` take objects with no
/// `(String)` constructor, which expressions cannot build; they are left out.
pub(crate) static METHODS: &[Method] = &[
    Method::new("valid", &[], |f, _| s(IdNumber::new(f).valid())),
    // Expressions cannot build an `IdNumberRequest`, a `LocalDate` or a `PolishIdNumber.Gender`; a template passes nulls.
    Method::new("valid", &[P::Opaque("IdNumberRequest")], |f, _| {
        let id = IdNumber::new(f);
        match id.country_provider() {
            None => Err(Error::illegal_argument(format!("ID Number generation not supported for country '{}'", f.locale().country()))),
            // Not reached from MockServer, whose fakers are `en`: each generator dereferences the request its own way.
            Some(_) => Err(Error::without_message("NullPointerException")),
        }
    })
    .accepting_null(),
    // `pickGender(faker, null)` draws a gender before `birthDate.getYear()` throws.
    Method::new("peselNumber", &[P::Opaque("LocalDate"), P::Opaque("Gender")], |f, _| {
        crate::idnumbers::random_gender(f);
        Err(crate::dispatch::npe("Cannot invoke \"java.time.LocalDate.getYear()\" because \"birthDate\" is null"))
    })
    .accepting_null(),
    Method::new("invalid", &[], |f, _| s(IdNumber::new(f).invalid())),
    Method::new("ssnValid", &[], |f, _| s(IdNumber::new(f).ssn_valid())),
    Method::new("validSvSeSsn", &[], |f, _| s(IdNumber::new(f).valid_sv_se_ssn())),
    Method::new("invalidSvSeSsn", &[], |f, _| s(IdNumber::new(f).invalid_sv_se_ssn())),
    Method::new("validEnZaSsn", &[], |f, _| s(IdNumber::new(f).valid_en_za_ssn())),
    Method::new("inValidEnZaSsn", &[], |f, _| s(IdNumber::new(f).in_valid_en_za_ssn())),
    Method::new("singaporeanFin", &[], |f, _| s(IdNumber::new(f).singaporean_fin())),
    Method::new("singaporeanFinBefore2000", &[], |f, _| s(IdNumber::new(f).singaporean_fin_before2000())),
    Method::new("singaporeanUin", &[], |f, _| s(IdNumber::new(f).singaporean_uin())),
    Method::new("singaporeanUinBefore2000", &[], |f, _| s(IdNumber::new(f).singaporean_uin_before2000())),
    Method::new("validZhCNSsn", &[], |f, _| s(IdNumber::new(f).valid_zh_cn_ssn())),
    Method::new("validPtNif", &[], |f, _| s(IdNumber::new(f).valid_pt_nif())),
    Method::new("invalidPtNif", &[], |f, _| s(IdNumber::new(f).invalid_pt_nif())),
    Method::new("validEsMXSsn", &[], |f, _| s(IdNumber::new(f).valid_es_mx_ssn())),
    Method::new("invalidEsMXSsn", &[], |f, _| s(IdNumber::new(f).invalid_es_mx_ssn())),
    Method::new("peselNumber", &[], |f, _| s(IdNumber::new(f).pesel_number())),
    Method::new("validKoKrRrn", &[], |f, _| s(IdNumber::new(f).valid_ko_kr_rrn())),
    Method::new("validGeIDNumber", &[], |f, _| s(IdNumber::new(f).valid_ge_id_number())),
];
