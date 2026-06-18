use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KycStatus {
    Pending,
    Approved,
    Rejected,
}

impl KycStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            KycStatus::Pending => "pending",
            KycStatus::Approved => "approved",
            KycStatus::Rejected => "rejected",
        }
    }
}

impl FromStr for KycStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(KycStatus::Pending),
            "approved" => Ok(KycStatus::Approved),
            "rejected" => Ok(KycStatus::Rejected),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycSubmission {
    pub user_id: Uuid,
    pub name: String,
    pub jurisdiction: String,
    pub documents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycResult {
    pub status: KycStatus,
    pub reason: Option<String>,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn submit(&self, submission: KycSubmission) -> anyhow::Result<KycStatus>;
    async fn check_status(&self, user_id: Uuid) -> anyhow::Result<KycStatus>;
}
