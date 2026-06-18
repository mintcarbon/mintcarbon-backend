use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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

pub struct NotificationService {
    pool: PgPool,
    email_sender: std::sync::Arc<dyn EmailSender>,
}

impl NotificationService {
    pub fn new(pool: PgPool, email_sender: std::sync::Arc<dyn EmailSender>) -> Self {
        Self { pool, email_sender }
    }

    pub async fn notify(
        &self,
        user_id: Uuid,
        event: NotificationEvent,
        payload: serde_json::Value,
    ) -> anyhow::Result<()> {
        // Store in DB
        sqlx::query("INSERT INTO audit_events (event_type, actor_id, payload) VALUES ($1, $2, $3)")
            .bind("notification_sent")
            .bind(user_id)
            .bind(serde_json::json!({
                "event": format!("{:?}", event),
                "payload": payload,
            }))
            .execute(&self.pool)
            .await?;

        // Try send email with retry
        let mut attempts = 0;
        while attempts < 3 {
            match self
                .email_sender
                .send("user@example.com", "Notification", &format!("{:?}", event))
                .await
            {
                Ok(_) => break,
                Err(e) => {
                    attempts += 1;
                    if attempts == 3 {
                        tracing::error!("Failed to send email after 3 attempts: {:?}", e);
                    } else {
                        tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempts)))
                            .await;
                    }
                }
            }
        }

        Ok(())
    }
}
