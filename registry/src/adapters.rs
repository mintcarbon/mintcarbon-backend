use async_trait::async_trait;
use chrono::Utc;

use crate::provider::{RegistryProvider, VerificationRecord};

pub struct VerraAdapter;

#[async_trait]
impl RegistryProvider for VerraAdapter {
    async fn validate_cert(&self, cert_id: &str) -> anyhow::Result<VerificationRecord> {
        Ok(VerificationRecord {
            registry: "verra".into(),
            cert_id: cert_id.to_string(),
            project_id: format!("VCS-{}", cert_id),
            project_name: format!("Verra Project {}", cert_id),
            project_type: Some("renewable_energy".into()),
            location: Some("Global".into()),
            vintage_year: Some(2024),
            timestamp: Utc::now(),
            suspended: false,
        })
    }

    async fn check_revocation(&self, _cert_id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

pub struct GoldStandardAdapter;

#[async_trait]
impl RegistryProvider for GoldStandardAdapter {
    async fn validate_cert(&self, cert_id: &str) -> anyhow::Result<VerificationRecord> {
        Ok(VerificationRecord {
            registry: "gold_standard".into(),
            cert_id: cert_id.to_string(),
            project_id: format!("GS-{}", cert_id),
            project_name: format!("Gold Standard Project {}", cert_id),
            project_type: Some("clean_energy".into()),
            location: Some("Global".into()),
            vintage_year: Some(2024),
            timestamp: Utc::now(),
            suspended: false,
        })
    }

    async fn check_revocation(&self, _cert_id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}

pub struct AcrAdapter;

#[async_trait]
impl RegistryProvider for AcrAdapter {
    async fn validate_cert(&self, cert_id: &str) -> anyhow::Result<VerificationRecord> {
        Ok(VerificationRecord {
            registry: "acr".into(),
            cert_id: cert_id.to_string(),
            project_id: format!("ACR-{}", cert_id),
            project_name: format!("ACR Project {}", cert_id),
            project_type: Some("forestry".into()),
            location: Some("Global".into()),
            vintage_year: Some(2024),
            timestamp: Utc::now(),
            suspended: false,
        })
    }

    async fn check_revocation(&self, _cert_id: &str) -> anyhow::Result<bool> {
        Ok(false)
    }
}
