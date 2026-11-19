//! `net.datafaker.providers.base.FinancialTerms`.

use super::{ProviderKind, resolve};
use crate::dispatch::{JValue, Method, P};
use crate::{Faker, Result};

/// `FinancialTerms.Category`, in declaration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// `ACCOUNT_MANAGEMENT`.
    AccountManagement,
    /// `TRANSACTIONS`.
    Transactions,
    /// `PAYMENTS`.
    Payments,
    /// `TRANSFERS`.
    Transfers,
    /// `INVESTMENTS`.
    Investments,
    /// `LOANS`.
    Loans,
    /// `ALERTS_AND_NOTIFICATIONS`.
    AlertsAndNotifications,
    /// `CUSTOMER_SUPPORT`.
    CustomerSupport,
    /// `SECURITY`.
    Security,
    /// `REGULATORY_COMPLIANCE`.
    RegulatoryCompliance,
    /// `MARKET_DATA`.
    MarketData,
    /// `USER_ACTIVITY`.
    UserActivity,
    /// `BANKING`.
    Banking,
    /// `CARDS`.
    Cards,
    /// `CRYPTOCURRENCY`.
    Cryptocurrency,
    /// `INSURANCE`.
    Insurance,
    /// `REWARDS_AND_LOYALTY`.
    RewardsAndLoyalty,
    /// `MISCELLANEOUS`.
    Miscellaneous,
}

/// The constants' names, for the method tables.
const CONSTANTS: &[&str] = &["ACCOUNT_MANAGEMENT", "TRANSACTIONS", "PAYMENTS", "TRANSFERS", "INVESTMENTS", "LOANS", "ALERTS_AND_NOTIFICATIONS", "CUSTOMER_SUPPORT", "SECURITY", "REGULATORY_COMPLIANCE", "MARKET_DATA", "USER_ACTIVITY", "BANKING", "CARDS", "CRYPTOCURRENCY", "INSURANCE", "REWARDS_AND_LOYALTY", "MISCELLANEOUS"];

const CATEGORY: P = P::Enum("Category", CONSTANTS);

impl Category {
    /// `Category.values()`.
    pub const VALUES: [Category; 18] = [
        Category::AccountManagement,
        Category::Transactions,
        Category::Payments,
        Category::Transfers,
        Category::Investments,
        Category::Loans,
        Category::AlertsAndNotifications,
        Category::CustomerSupport,
        Category::Security,
        Category::RegulatoryCompliance,
        Category::MarketData,
        Category::UserActivity,
        Category::Banking,
        Category::Cards,
        Category::Cryptocurrency,
        Category::Insurance,
        Category::RewardsAndLoyalty,
        Category::Miscellaneous,
    ];

    /// `getName()`: the YAML key of the category.
    pub fn name(self) -> &'static str {
        match self {
            Category::AccountManagement => "account_management",
            Category::Transactions => "transactions",
            Category::Payments => "payments",
            Category::Transfers => "transfers",
            Category::Investments => "investments",
            Category::Loans => "loans",
            Category::AlertsAndNotifications => "alerts_and_notifications",
            Category::CustomerSupport => "customer_support",
            Category::Security => "security",
            Category::RegulatoryCompliance => "regulatory_compliance",
            Category::MarketData => "market_data",
            Category::UserActivity => "user_activity",
            Category::Banking => "banking",
            Category::Cards => "cards",
            Category::Cryptocurrency => "cryptocurrency",
            Category::Insurance => "insurance",
            Category::RewardsAndLoyalty => "rewards_and_loyalty",
            Category::Miscellaneous => "miscellaneous",
        }
    }

    /// The Java constant's name (`ACCOUNT_MANAGEMENT`).
    pub fn constant(self) -> &'static str {
        match self {
            Category::AccountManagement => "ACCOUNT_MANAGEMENT",
            Category::Transactions => "TRANSACTIONS",
            Category::Payments => "PAYMENTS",
            Category::Transfers => "TRANSFERS",
            Category::Investments => "INVESTMENTS",
            Category::Loans => "LOANS",
            Category::AlertsAndNotifications => "ALERTS_AND_NOTIFICATIONS",
            Category::CustomerSupport => "CUSTOMER_SUPPORT",
            Category::Security => "SECURITY",
            Category::RegulatoryCompliance => "REGULATORY_COMPLIANCE",
            Category::MarketData => "MARKET_DATA",
            Category::UserActivity => "USER_ACTIVITY",
            Category::Banking => "BANKING",
            Category::Cards => "CARDS",
            Category::Cryptocurrency => "CRYPTOCURRENCY",
            Category::Insurance => "INSURANCE",
            Category::RewardsAndLoyalty => "REWARDS_AND_LOYALTY",
            Category::Miscellaneous => "MISCELLANEOUS",
        }
    }

    fn of(value: &JValue) -> Category {
        let JValue::Enum(name) = value else { unreachable!("coerced to a Category") };
        Category::VALUES.into_iter().find(|c| c.constant() == *name).expect("a Category constant")
    }
}

/// Financial terms by category, from `financial_terms.<category>.*`.
#[derive(Clone, Copy)]
pub struct FinancialTerms<'a> {
    faker: &'a Faker,
}

impl<'a> FinancialTerms<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        FinancialTerms { faker }
    }

    fn term(&self, category: Category, kind: &str) -> Result<String> {
        resolve(self.faker, ProviderKind::FinancialTerms, &format!("financial_terms.{}.{kind}", category.name()))
    }

    /// A category drawn as `options().option(Category.class)` does (`random().nextEnum`).
    fn any_category(&self) -> Result<Category> {
        Ok(Category::VALUES[self.faker.random().next_enum(Category::VALUES.len())?])
    }

    /// `noun()`: a noun of a random category.
    pub fn noun(&self) -> Result<String> {
        self.noun_of(self.any_category()?)
    }

    /// `noun(category)`.
    pub fn noun_of(&self, category: Category) -> Result<String> {
        self.term(category, "nouns")
    }

    /// `verb()`: a verb of a random category.
    pub fn verb(&self) -> Result<String> {
        self.verb_of(self.any_category()?)
    }

    /// `verb(category)`.
    pub fn verb_of(&self, category: Category) -> Result<String> {
        self.term(category, "verbs")
    }

    /// `adjective()`: an adjective of a random category.
    pub fn adjective(&self) -> Result<String> {
        self.adjective_of(self.any_category()?)
    }

    /// `adjective(category)`.
    pub fn adjective_of(&self, category: Category) -> Result<String> {
        self.term(category, "adjectives")
    }
}

fn s(r: Result<String>) -> Result<JValue> {
    r.map(JValue::Str)
}

fn category(v: &JValue) -> Result<Category> {
    if *v == JValue::Null {
        return Err(crate::dispatch::npe("Cannot invoke \"net.datafaker.providers.base.FinancialTerms$Category.getName()\" because \"category\" is null"));
    }
    Ok(Category::of(v))
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("noun", &[], |f, _| s(FinancialTerms::new(f).noun())),
    Method::new("noun", &[CATEGORY], |f, a| s(FinancialTerms::new(f).noun_of(category(&a[0])?))).accepting_null(),
    Method::new("verb", &[], |f, _| s(FinancialTerms::new(f).verb())),
    Method::new("verb", &[CATEGORY], |f, a| s(FinancialTerms::new(f).verb_of(category(&a[0])?))).accepting_null(),
    Method::new("adjective", &[], |f, _| s(FinancialTerms::new(f).adjective())),
    Method::new("adjective", &[CATEGORY], |f, a| s(FinancialTerms::new(f).adjective_of(category(&a[0])?))).accepting_null(),
];
