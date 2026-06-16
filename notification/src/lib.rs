pub enum NotificationEvent {
    RegistrationConfirmed,
    KycStatusChanged,
    OrderConfirmed,
    ListingCreated,
    CertificateRevoked,
    UpgradeProposed,
}

#[async_trait::async_trait]
pub trait EmailSender: Send + Sync {
    async fn send(&self, to: &str, subject: &str, body: &str) -> anyhow::Result<()>;
}

pub struct StubEmailSender;

#[async_trait::async_trait]
impl EmailSender for StubEmailSender {
    async fn send(&self, to: &str, subject: &str, body: &str) -> anyhow::Result<()> {
        tracing::info!("[STUB EMAIL] To: {to} | Subject: {subject} | Body: {body}");
        Ok(())
    }
}
