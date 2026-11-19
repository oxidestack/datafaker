//! `net.datafaker.providers.base.Azure`.

use super::{ProviderKind, resolve, s};
use crate::dispatch::Method;
use crate::{Faker, Result};

/// Microsoft Azure identifiers: regions, subscription ids, and resource names (a prefix and 16 hex digits).
#[derive(Clone, Copy)]
pub struct Azure<'a> {
    faker: &'a Faker,
}

impl<'a> Azure<'a> {
    pub(crate) fn new(faker: &'a Faker) -> Self {
        Azure { faker }
    }

    /// `azure.regions`.
    pub fn region(&self) -> Result<String> {
        resolve(self.faker, ProviderKind::Azure, "azure.regions")
    }

    /// A GUID-shaped id of lower-case hex digits.
    pub fn subscription_id(&self) -> String {
        let r = self.faker.random();
        [8, 4, 4, 4, 12].map(|n| r.hex_with(n, false)).join("-")
    }

    /// `tenantId()`: [`Azure::subscription_id`].
    pub fn tenant_id(&self) -> String {
        self.subscription_id()
    }

    fn rand_hex(&self) -> String {
        self.faker.random().hex_with(16, false)
    }

    /// `rg-` and 16 hex digits.
    pub fn resource_group(&self) -> String {
        format!("rg-{}", self.rand_hex())
    }

    /// `mg-` and 16 hex digits.
    pub fn management_group(&self) -> String {
        format!("mg-{}", self.rand_hex())
    }

    /// `agw-` and 16 hex digits.
    pub fn application_gateway(&self) -> String {
        format!("agw-{}", self.rand_hex())
    }

    /// `bas-` and 16 hex digits.
    pub fn bastion_host(&self) -> String {
        format!("bas-{}", self.rand_hex())
    }

    /// `afw-` and 16 hex digits.
    pub fn firewall(&self) -> String {
        format!("afw-{}", self.rand_hex())
    }

    /// `lbi-` and 16 hex digits.
    pub fn load_balancer(&self) -> String {
        format!("lbi-{}", self.rand_hex())
    }

    /// `nsg-` and 16 hex digits.
    pub fn network_security_group(&self) -> String {
        format!("nsg-{}", self.rand_hex())
    }

    /// `vnet-` and 16 hex digits.
    pub fn virtual_network(&self) -> String {
        format!("vnet-{}", self.rand_hex())
    }

    /// `vwan-` and 16 hex digits.
    pub fn virtual_wan(&self) -> String {
        format!("vwan-{}", self.rand_hex())
    }

    /// `ase-` and 16 hex digits.
    pub fn app_service_environment(&self) -> String {
        format!("ase-{}", self.rand_hex())
    }

    /// `asp-` and 16 hex digits.
    pub fn app_service_plan(&self) -> String {
        format!("asp-{}", self.rand_hex())
    }

    /// `lt-` and 16 hex digits.
    pub fn load_testing(&self) -> String {
        format!("lt-{}", self.rand_hex())
    }

    /// `stapp-` and 16 hex digits.
    pub fn static_web_app(&self) -> String {
        format!("stapp-{}", self.rand_hex())
    }

    /// `vm-` and 16 hex digits.
    pub fn virtual_machine(&self) -> String {
        format!("vm-{}", self.rand_hex())
    }

    /// `st-` and 16 hex digits.
    pub fn storage_account(&self) -> String {
        format!("st-{}", self.rand_hex())
    }

    /// `cr-` and 16 hex digits.
    pub fn container_registry(&self) -> String {
        format!("cr-{}", self.rand_hex())
    }

    /// `ca-` and 16 hex digits.
    pub fn container_apps(&self) -> String {
        format!("ca-{}", self.rand_hex())
    }

    /// `cae-` and 16 hex digits.
    pub fn container_apps_environment(&self) -> String {
        format!("cae-{}", self.rand_hex())
    }

    /// `ci-` and 16 hex digits.
    pub fn container_instance(&self) -> String {
        format!("ci-{}", self.rand_hex())
    }

    /// `cosmos-` and 16 hex digits.
    pub fn cosmos_db_database(&self) -> String {
        format!("cosmos-{}", self.rand_hex())
    }

    /// `sql-` and 16 hex digits.
    pub fn sql_database(&self) -> String {
        format!("sql-{}", self.rand_hex())
    }

    /// `mysql-` and 16 hex digits.
    pub fn mysql_database(&self) -> String {
        format!("mysql-{}", self.rand_hex())
    }

    /// `psql-` and 16 hex digits.
    pub fn postgre_sql_database(&self) -> String {
        format!("psql-{}", self.rand_hex())
    }

    /// `sb-` and 16 hex digits.
    pub fn service_bus(&self) -> String {
        format!("sb-{}", self.rand_hex())
    }

    /// `sbq-` and 16 hex digits.
    pub fn service_bus_queue(&self) -> String {
        format!("sbq-{}", self.rand_hex())
    }

    /// `sbt-` and 16 hex digits.
    pub fn service_bus_topic(&self) -> String {
        format!("sbt-{}", self.rand_hex())
    }

    /// `kv-` and 16 hex digits.
    pub fn key_vault(&self) -> String {
        format!("kv-{}", self.rand_hex())
    }

    /// `log-` and 16 hex digits.
    pub fn log_analytics(&self) -> String {
        format!("log-{}", self.rand_hex())
    }

    /// `sa-` and 16 hex digits.
    pub fn spring_apps(&self) -> String {
        format!("sa-{}", self.rand_hex())
    }
}

pub(crate) static METHODS: &[Method] = &[
    Method::new("region", &[], |f, _| s(Azure::new(f).region())),
    Method::new("subscriptionId", &[], |f, _| s(Ok(Azure::new(f).subscription_id()))),
    Method::new("tenantId", &[], |f, _| s(Ok(Azure::new(f).tenant_id()))),
    Method::new("resourceGroup", &[], |f, _| s(Ok(Azure::new(f).resource_group()))),
    Method::new("managementGroup", &[], |f, _| s(Ok(Azure::new(f).management_group()))),
    Method::new("applicationGateway", &[], |f, _| s(Ok(Azure::new(f).application_gateway()))),
    Method::new("bastionHost", &[], |f, _| s(Ok(Azure::new(f).bastion_host()))),
    Method::new("firewall", &[], |f, _| s(Ok(Azure::new(f).firewall()))),
    Method::new("loadBalancer", &[], |f, _| s(Ok(Azure::new(f).load_balancer()))),
    Method::new("networkSecurityGroup", &[], |f, _| s(Ok(Azure::new(f).network_security_group()))),
    Method::new("virtualNetwork", &[], |f, _| s(Ok(Azure::new(f).virtual_network()))),
    Method::new("virtualWan", &[], |f, _| s(Ok(Azure::new(f).virtual_wan()))),
    Method::new("appServiceEnvironment", &[], |f, _| s(Ok(Azure::new(f).app_service_environment()))),
    Method::new("appServicePlan", &[], |f, _| s(Ok(Azure::new(f).app_service_plan()))),
    Method::new("loadTesting", &[], |f, _| s(Ok(Azure::new(f).load_testing()))),
    Method::new("staticWebApp", &[], |f, _| s(Ok(Azure::new(f).static_web_app()))),
    Method::new("virtualMachine", &[], |f, _| s(Ok(Azure::new(f).virtual_machine()))),
    Method::new("storageAccount", &[], |f, _| s(Ok(Azure::new(f).storage_account()))),
    Method::new("containerRegistry", &[], |f, _| s(Ok(Azure::new(f).container_registry()))),
    Method::new("containerApps", &[], |f, _| s(Ok(Azure::new(f).container_apps()))),
    Method::new("containerAppsEnvironment", &[], |f, _| s(Ok(Azure::new(f).container_apps_environment()))),
    Method::new("containerInstance", &[], |f, _| s(Ok(Azure::new(f).container_instance()))),
    Method::new("cosmosDBDatabase", &[], |f, _| s(Ok(Azure::new(f).cosmos_db_database()))),
    Method::new("sqlDatabase", &[], |f, _| s(Ok(Azure::new(f).sql_database()))),
    Method::new("mysqlDatabase", &[], |f, _| s(Ok(Azure::new(f).mysql_database()))),
    Method::new("postgreSQLDatabase", &[], |f, _| s(Ok(Azure::new(f).postgre_sql_database()))),
    Method::new("serviceBus", &[], |f, _| s(Ok(Azure::new(f).service_bus()))),
    Method::new("serviceBusQueue", &[], |f, _| s(Ok(Azure::new(f).service_bus_queue()))),
    Method::new("serviceBusTopic", &[], |f, _| s(Ok(Azure::new(f).service_bus_topic()))),
    Method::new("keyVault", &[], |f, _| s(Ok(Azure::new(f).key_vault()))),
    Method::new("logAnalytics", &[], |f, _| s(Ok(Azure::new(f).log_analytics()))),
    Method::new("springApps", &[], |f, _| s(Ok(Azure::new(f).spring_apps()))),
];
