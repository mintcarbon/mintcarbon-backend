use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationRecord {
    pub registry: String,
    pub cert_id: String,
    pub project_id: String,
    pub project_name: String,
    pub project_type: Option<String>,
    pub location: Option<String>,
    pub vintage_year: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub suspended: bool,
}

#[async_trait]
pub trait RegistryProvider: Send + Sync {
    async fn validate_cert(&self, cert_id: &str) -> anyhow::Result<VerificationRecord>;
    async fn check_revocation(&self, cert_id: &str) -> anyhow::Result<bool>;
}
