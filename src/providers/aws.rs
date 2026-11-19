//! `net.datafaker.providers.base.Aws`.

use super::text::{EN_UPPERCASE, TextSymbolsBuilder};
use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Amazon Web Services identifiers: regions, services, account ids, ARNs, resource ids.
#[derive(Clone, Copy)]
pub struct Aws<'a> {
    faker: &'a Faker,
}

impl<'a> Aws<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Aws { faker }
    }

    /// `aws.regions`.
    pub fn region(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Aws, "aws.regions")
    }

    /// `aws.services`.
    pub fn service(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Aws, "aws.services")
    }

    /// Ten digits starting with `0`.
    pub fn account_id(&self) -> String {
        self.faker.numerify("0#########")
    }

    /// An ACM certificate ARN.
    pub fn acm_arn(&self) -> Result<String> {
        let region = self.region()?;
        let account = self.account_id();
        Ok(format!("arn:aws:acm:{region}:{account}:certificate/{}", self.faker.internet().uuid()))
    }

    /// An application load balancer ARN.
    pub fn alb_arn(&self) -> Result<String> {
        self.elb_arn("loadbalancer/app")
    }

    /// An ALB target group ARN.
    pub fn alb_target_group_arn(&self) -> Result<String> {
        self.elb_arn("targetgroup")
    }

    fn elb_arn(&self, kind: &str) -> Result<String> {
        let region = self.region()?;
        let account = self.account_id();
        let app = self.app_name()?;
        Ok(format!("arn:aws:elasticloadbalancing:{region}:{account}:{kind}/{app}/{}", self.rand_hex(16)))
    }

    /// 21 upper-case letters.
    pub fn route53_zone_id(&self) -> Result<String> {
        let config = TextSymbolsBuilder::builder().with(EN_UPPERCASE).len(21).build()?;
        self.faker.text().text_from(&config)
    }

    /// `sg-` and 17 hex digits.
    pub fn security_group_id(&self) -> String {
        format!("sg-{}", self.rand_hex(17))
    }

    /// `subnet-` and 17 hex digits.
    pub fn subnet_id(&self) -> String {
        format!("subnet-{}", self.rand_hex(17))
    }

    /// `vpc-` and 17 hex digits.
    pub fn vpc_id(&self) -> String {
        format!("vpc-{}", self.rand_hex(17))
    }

    /// `app().name()` lower-cased (the JVM's default locale, `en_US`) without non-word characters.
    fn app_name(&self) -> Result<String> {
        Ok(self.faker.app().name()?.to_lowercase().chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').collect())
    }

    fn rand_hex(&self, length: i32) -> String {
        self.faker.random().hex_with(length, false)
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("region", &[], |f, _| s(Aws::new(f).region())),
    Method::new("service", &[], |f, _| s(Aws::new(f).service())),
    Method::new("accountId", &[], |f, _| s(Ok(Aws::new(f).account_id()))),
    Method::new("acmARN", &[], |f, _| s(Aws::new(f).acm_arn())),
    Method::new("albARN", &[], |f, _| s(Aws::new(f).alb_arn())),
    Method::new("albTargetGroupARN", &[], |f, _| s(Aws::new(f).alb_target_group_arn())),
    Method::new("route53ZoneId", &[], |f, _| s(Aws::new(f).route53_zone_id())),
    Method::new("securityGroupId", &[], |f, _| s(Ok(Aws::new(f).security_group_id()))),
    Method::new("subnetId", &[], |f, _| s(Ok(Aws::new(f).subnet_id()))),
    Method::new("vpcId", &[], |f, _| s(Ok(Aws::new(f).vpc_id()))),
];
