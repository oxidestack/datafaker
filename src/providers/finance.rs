//! `net.datafaker.providers.base.Finance`, with its `CreditCardType` enum.

use super::{ProviderKind, resolve, s};
use crate::dispatch::{JValue, Method, P};
use crate::{Error, Faker, Result};

/// `Finance.CreditCardType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditCardType {
    /// `VISA`.
    Visa,
    /// `MASTERCARD`.
    Mastercard,
    /// `DISCOVER`.
    Discover,
    /// `AMERICAN_EXPRESS`.
    AmericanExpress,
    /// `DINERS_CLUB`.
    DinersClub,
    /// `JCB`.
    Jcb,
    /// `DANKORT`.
    Dankort,
    /// `FORBRUGSFORENINGEN`.
    Forbrugsforeningen,
    /// `LASER`.
    Laser,
    /// `UNIONPAY`.
    Unionpay,
}

impl CreditCardType {
    /// The constants in declaration order.
    pub const ALL: [CreditCardType; 10] = [
        CreditCardType::Visa,
        CreditCardType::Mastercard,
        CreditCardType::Discover,
        CreditCardType::AmericanExpress,
        CreditCardType::DinersClub,
        CreditCardType::Jcb,
        CreditCardType::Dankort,
        CreditCardType::Forbrugsforeningen,
        CreditCardType::Laser,
        CreditCardType::Unionpay,
    ];

    /// The constant's Java name (`name()`).
    pub fn name(self) -> &'static str {
        CONSTANTS[self as usize]
    }

    /// `valueOf(name)`.
    pub fn value_of(name: &str) -> Option<CreditCardType> {
        CONSTANTS.iter().position(|c| *c == name).map(|i| CreditCardType::ALL[i])
    }
}

const CONSTANTS: &[&str] =
    &["VISA", "MASTERCARD", "DISCOVER", "AMERICAN_EXPRESS", "DINERS_CLUB", "JCB", "DANKORT", "FORBRUGSFORENINGEN", "LASER", "UNIONPAY"];

const CREDIT_CARD_TYPE: P = P::Enum("CreditCardType", CONSTANTS);

/// `countryCodeToBasicBankAccountNumberPattern` (a `LinkedHashMap`, in its order): each country's BBAN regex.
const BBAN_PATTERNS: &[(&str, &str)] = &[
    ("AD", r"[0-9]{8}[0-9A-Z]{12}"),
    ("AE", r"[0-9]{19}"),
    ("AL", r"[0-9]{8}[0-9A-Z]{16}"),
    ("AO", r"[0-9]{21}"),
    ("AT", r"[0-9]{16}"),
    ("AZ", r"[A-Z]{4}[0-9A-Z]{20}"),
    ("BA", r"[0-9]{16}"),
    ("BE", r"[0-9]{12}"),
    ("BF", r"[0-9A-Z]{5}[0-9]{18}"),
    ("BG", r"[A-Z]{4}[0-9]{6}[0-9A-Z]{8}"),
    ("BH", r"[A-Z]{4}[0-9A-Z]{14}"),
    ("BI", r"[0-9]{23}"),
    ("BJ", r"[0-9A-Z]{5}[0-9]{19}"),
    ("BR", r"[0-9]{23}[A-Z][0-9A-Z]"),
    ("BY", r"[0-9A-Z]{4}[0-9]{4}[0-9A-Z]{16}"),
    ("CF", r"[0-9]{23}"),
    ("CH", r"[0-9]{5}[0-9A-Z]{12}"),
    ("CM", r"[0-9]{23}"),
    ("CR", r"[0-9]{18}"),
    ("CV", r"[0-9]{8}[0-9A-Z]{13}"),
    ("CY", r"[0-9]{8}[0-9A-Z]{16}"),
    ("CZ", r"[0-9]{20}"),
    ("DE", r"[0-9]{18}"),
    ("DJ", r"[0-9]{23}"),
    ("DK", r"[0-9]{14}"),
    ("DO", r"[0-9A-Z]{4}[0-9]{20}"),
    ("DZ", r"[0-9]{20}"),
    ("EE", r"[0-9]{16}"),
    ("EG", r"[0-9]{25}"),
    ("ES", r"[0-9]{20}"),
    ("FI", r"[0-9]{14}"),
    ("FK", r"[A-Z]{2}[0-9]{12}"),
    ("FO", r"[0-9]{14}"),
    ("FR", r"[0-9]{10}[0-9A-Z]{11}[0-9]{2}"),
    ("GA", r"[0-9]{10}[0-9A-Z]{13}"),
    ("GB", r"[A-Z]{4}[0-9]{14}"),
    ("GE", r"[A-Z]{2}[0-9]{16}"),
    ("GI", r"[A-Z]{4}[0-9A-Z]{15}"),
    ("GL", r"[0-9]{14}"),
    ("GQ", r"[0-9]{23}"),
    ("GR", r"[0-9]{7}[0-9A-Z]{16}"),
    ("GT", r"[0-9A-Z]{24}"),
    ("HN", r"[A-Z]{4}[0-9]{20}"),
    ("HR", r"[0-9]{17}"),
    ("HU", r"[0-9]{24}"),
    ("IE", r"[A-Z]{4}[0-9]{14}"),
    ("IL", r"[0-9]{19}"),
    ("IQ", r"[A-Z]{4}[0-9]{15}"),
    ("IR", r"[0-9]{22}"),
    ("IS", r"[0-9]{22}"),
    ("IT", r"[A-Z][0-9]{10}[0-9A-Z]{12}"),
    ("JO", r"[A-Z]{4}[0-9]{4}[0-9A-Z]{18}"),
    ("KM", r"[0-9]{23}"),
    ("KW", r"[A-Z]{4}[0-9A-Z]{22}"),
    ("KZ", r"[0-9]{3}[0-9A-Z]{13}"),
    ("LB", r"[0-9]{4}[0-9A-Z]{20}"),
    ("LC", r"[A-Z]{4}[0-9A-Z]{24}"),
    ("LI", r"[0-9]{5}[0-9A-Z]{12}"),
    ("LT", r"[0-9]{16}"),
    ("LU", r"[0-9]{3}[0-9A-Z]{13}"),
    ("LV", r"[A-Z]{4}[0-9A-Z]{13}"),
    ("LY", r"[0-9]{21}"),
    ("MA", r"[0-9]{24}"),
    ("MC", r"[0-9]{10}[0-9A-Z]{11}[0-9]{2}"),
    ("MD", r"[0-9A-Z]{20}"),
    ("ME", r"[0-9]{18}"),
    ("MK", r"[0-9]{3}[0-9A-Z]{10}[0-9]{2}"),
    ("MN", r"[0-9]{16}"),
    ("MR", r"[0-9]{23}"),
    ("MT", r"[A-Z]{4}[0-9]{5}[0-9A-Z]{18}"),
    ("MU", r"[A-Z]{4}[0-9]{19}[A-Z]{3}"),
    ("MZ", r"[0-9]{21}"),
    ("NI", r"[A-Z]{4}[0-9]{20}"),
    ("NL", r"[A-Z]{4}[0-9]{10}"),
    ("NO", r"[0-9]{11}"),
    ("OM", r"[0-9]{3}[0-9A-Z]{16}"),
    ("PK", r"[A-Z]{4}[0-9A-Z]{16}"),
    ("PL", r"[0-9]{24}"),
    ("PS", r"[A-Z]{4}[0-9A-Z]{21}"),
    ("PT", r"[0-9]{21}"),
    ("QA", r"[A-Z]{4}[0-9A-Z]{21}"),
    ("RO", r"[A-Z]{4}[0-9A-Z]{16}"),
    ("RS", r"[0-9]{18}"),
    ("RU", r"[0-9]{14}[0-9A-Z]{15}"),
    ("SA", r"[0-9]{2}[0-9A-Z]{18}"),
    ("SC", r"[A-Z]{4}[0-9]{20}[A-Z]{3}"),
    ("SD", r"[0-9]{14}"),
    ("SE", r"[0-9]{20}"),
    ("SI", r"[0-9]{15}"),
    ("SK", r"[0-9]{20}"),
    ("SM", r"[A-Z][0-9]{10}[0-9A-Z]{12}"),
    ("SN", r"[0-9A-Z]{5}[0-9]{19}"),
    ("SO", r"[0-9]{19}"),
    ("ST", r"[0-9]{21}"),
    ("SV", r"[A-Z]{4}[0-9]{20}"),
    ("TG", r"[0-9A-Z]{5}[0-9]{19}"),
    ("TL", r"[0-9]{19}"),
    ("TN", r"[0-9]{20}"),
    ("TR", r"[0-9]{6}[0-9A-Z]{16}"),
    ("UA", r"[0-9]{6}[0-9A-Z]{19}"),
    ("VA", r"[0-9]{18}"),
    ("VG", r"[A-Z]{4}[0-9]{16}"),
    ("XK", r"[0-9]{16}"),
    ("YE", r"[A-Z]{4}[0-9]{4}[0-9A-Z]{18}"),
];

/// Finance: credit card numbers (Luhn), IBANs (ISO 7064 mod 97), BICs, US routing numbers, tickers.
#[derive(Clone, Copy)]
pub struct Finance<'a> {
    faker: &'a Faker,
}

impl<'a> Finance<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Finance { faker }
    }

    fn resolve(&self, key: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::Finance, key)
    }

    /// `finance.ticker.nasdaq` (deprecated upstream).
    pub fn nasdaq_ticker(&self) -> Result<String> {
        self.resolve("finance.ticker.nasdaq")
    }

    /// `finance.ticker.nyse` (deprecated upstream).
    pub fn nyse_ticker(&self) -> Result<String> {
        self.resolve("finance.ticker.nyse")
    }

    /// `finance.stock_market` (deprecated upstream).
    pub fn stock_market(&self) -> Result<String> {
        self.resolve("finance.stock_market")
    }

    /// The countries [`Finance::iban_for`] knows, in upstream's order (static upstream).
    pub fn iban_supported_countries() -> Vec<&'static str> {
        BBAN_PATTERNS.iter().map(|(c, _)| *c).collect()
    }

    /// `creditCard(type)`: the type's template (`finance.credit_card.<type>`) numerified, `L` replaced by the
    /// Luhn check digit, `/` and `\` by spaces.
    pub fn credit_card_of(&self, credit_card_type: CreditCardType) -> Result<String> {
        let key = format!("finance.credit_card.{}", credit_card_type.name().to_ascii_lowercase());
        let template: Vec<char> = self.faker.numerify(&self.resolve(&key)?).chars().collect();
        let digits: Vec<u32> = template.iter().filter_map(|c| c.to_digit(10)).collect();
        let mut luhn_sum = 0;
        let mut multiplier = 1;
        for d in digits.iter().rev() {
            multiplier = if multiplier == 2 { 1 } else { 2 };
            let v = d * multiplier;
            luhn_sum += v / 10 + v % 10;
        }
        let luhn_digit = (10 - luhn_sum % 10) % 10;
        let mut res = String::with_capacity(template.len());
        for (i, c) in template.iter().enumerate() {
            match c {
                '/' | '\\' => {
                    if !res.is_empty() && i != template.len() - 1 {
                        res.push(' ');
                    }
                }
                'L' => res.push_str(&luhn_digit.to_string()),
                c => res.push(*c),
            }
        }
        Ok(res.trim_matches(|c: char| c <= ' ').to_string())
    }

    /// `creditCard()`: a card of a random type.
    pub fn credit_card(&self) -> Result<String> {
        let t = CreditCardType::ALL[self.faker.random().next_enum(CreditCardType::ALL.len())?];
        self.credit_card_of(t)
    }

    /// A BIC: four letters, the upper-cased `country.code2`, two and optionally three more characters.
    pub fn bic(&self) -> Result<String> {
        let country = self.faker.country().country_code2()?.to_uppercase();
        self.faker.regexify(&format!("[A-Z]{{4}}{country}[0-9A-Z]{{2}}(?:[0-9A-Z]{{3}})?"))
    }

    /// `iban()`: an IBAN of a random supported country.
    pub fn iban(&self) -> Result<String> {
        let (country, _) = self.faker.options().next_element(BBAN_PATTERNS)?;
        self.iban_for(country)
    }

    /// `iban(countryCode)`: the country, the mod-97 check digits and a BBAN from the country's pattern. An
    /// unsupported country is `regexify(null)`'s `NullPointerException`.
    pub fn iban_for(&self, country_code: &str) -> Result<String> {
        let Some((_, pattern)) = BBAN_PATTERNS.iter().find(|(c, _)| *c == country_code) else {
            return Err(Error::new(
                "NullPointerException",
                "Cannot invoke \"String.length()\" because \"this.aValue\" is null",
            ));
        };
        let bban = self.faker.regexify(pattern)?;
        let check = iban_checksum(country_code, &bban);
        Ok(format!("{country_code}{check}{bban}"))
    }

    /// A US bank routing number: a Federal Reserve district `01`–`12`, six digits and the ABA check digit.
    pub fn us_routing_number(&self) -> Result<String> {
        let random = self.faker.random().next_int_bounded(12)? + 1;
        let base = format!("{}{random}{}", if random < 10 { "0" } else { "" }, self.faker.numerify("######"));
        let d: Vec<i32> = base.chars().map(|c| c.to_digit(10).unwrap_or(0) as i32).collect();
        let check = d[0] * 3 + d[1] * 7 + d[2] + d[3] * 3 + d[4] * 7 + d[5] + d[6] * 3 + d[7] * 7;
        let check = (check % 10 - 10).abs() % 10;
        Ok(format!("{base}{check}"))
    }
}

/// `calculateIbanChecksum`: `98 - (BBAN + country + "00", letters as 10–35) mod 97`, two digits.
fn iban_checksum(country_code: &str, bban: &str) -> String {
    let basis = format!("{bban}{country_code}").to_lowercase() + "00";
    let mut rem: u32 = 0;
    for c in basis.chars() {
        let digits = if c.is_alphabetic() { (c as u32).wrapping_sub('a' as u32) + 10 } else { c.to_digit(10).unwrap_or(0) };
        for d in digits.to_string().chars() {
            rem = (rem * 10 + d.to_digit(10).unwrap_or(0)) % 97;
        }
    }
    format!("{:02}", 98 - rem)
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("nasdaqTicker", &[], |f, _| s(Finance::new(f).nasdaq_ticker())),
    Method::new("nyseTicker", &[], |f, _| s(Finance::new(f).nyse_ticker())),
    Method::new("stockMarket", &[], |f, _| s(Finance::new(f).stock_market())),
    Method::new("ibanSupportedCountries", &[], |_, _| {
        Ok(JValue::List(Finance::iban_supported_countries().into_iter().map(JValue::str).collect()))
    }),
    Method::new("creditCard", &[CREDIT_CARD_TYPE], |f, a| {
        let t = match &a[0] {
            JValue::Enum(name) => CreditCardType::value_of(name),
            _ => None,
        };
        if a[0] == JValue::Null {
            return Err(crate::dispatch::npe(
                "Cannot invoke \"net.datafaker.providers.base.Finance$CreditCardType.toString()\" because \"creditCardType\" is null",
            ));
        }
        s(Finance::new(f).credit_card_of(t.unwrap_or(CreditCardType::Visa)))
    })
    .accepting_null(),
    Method::new("creditCard", &[], |f, _| s(Finance::new(f).credit_card())),
    Method::new("bic", &[], |f, _| s(Finance::new(f).bic())),
    Method::new("iban", &[], |f, _| s(Finance::new(f).iban())),
    // `countryCodeToBasicBankAccountNumberPattern.get(null)` is null, which `regexify` dereferences.
    Method::new("iban", &[P::String], |f, a| {
        s(Finance::new(f).iban_for(crate::dispatch::non_null(a, 0, "Cannot invoke \"String.length()\" because \"this.aValue\" is null")?.as_str()))
    })
    .accepting_null(),
    Method::new("usRoutingNumber", &[], |f, _| s(Finance::new(f).us_routing_number())),
];
