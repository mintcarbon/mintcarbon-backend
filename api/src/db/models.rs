use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    #[serde(skip_serializing)]
    pub mfa_secret: Option<String>,
    pub kyc_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub registry: String,
    pub cert_id: String,
    pub project_name: String,
    pub project_type: Option<String>,
    pub location: Option<String>,
    pub vintage_year: Option<i32>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct TokenHolding {
    pub user_id: Uuid,
    pub token_id: String,
    pub quantity: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Listing {
    pub id: Uuid,
    pub seller_id: Uuid,
    pub token_id: String,
    pub quantity: i64,
    pub price_lumens: i64,
    pub status: String,
    pub contract_listing_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub buyer_id: Uuid,
    pub listing_id: Uuid,
    pub quantity: i64,
    pub total_lumens: i64,
    pub tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AuditEvent {
    pub id: Uuid,
    pub event_type: String,
    pub actor_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub on_chain_index: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthPayload {
    pub sub: Uuid,
    pub role: String,
    pub exp: usize,
}
