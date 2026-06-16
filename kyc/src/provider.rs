use async_trait::async_trait;
use serde::{Deserialize, Serialize};
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

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(KycStatus::Pending),
            "approved" => Some(KycStatus::Approved),
            "rejected" => Some(KycStatus::Rejected),
            _ => None,
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
