use async_trait::async_trait;
use uuid::Uuid;

use crate::provider::{KycStatus, KycSubmission, Provider};

pub struct MockProvider;

#[async_trait]
impl Provider for MockProvider {
    async fn submit(&self, _submission: KycSubmission) -> anyhow::Result<KycStatus> {
        tracing::info!("[MOCK KYC] Submission received, auto-approving");
        Ok(KycStatus::Approved)
    }

    async fn check_status(&self, _user_id: Uuid) -> anyhow::Result<KycStatus> {
        Ok(KycStatus::Approved)
    }
}
