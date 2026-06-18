use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;

#[derive(Deserialize)]
pub struct KycWebhookPayload {
    pub user_id: Uuid,
    pub status: String,
    pub reason: Option<String>,
}

pub async fn kyc_webhook(
    State(state): State<AppState>,
    Json(payload): Json<KycWebhookPayload>,
) -> Result<(StatusCode, &'static str), (StatusCode, &'static str)> {
    let valid_statuses = ["approved", "rejected", "pending"];
    if !valid_statuses.contains(&payload.status.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid status"));
    }

    let result = sqlx::query("UPDATE users SET kyc_status = $1 WHERE id = $2")
        .bind(&payload.status)
        .bind(payload.user_id)
        .execute(&state.db)
        .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => Err((StatusCode::NOT_FOUND, "User not found")),
        Ok(_) => {
            sqlx::query(
                "INSERT INTO audit_events (event_type, actor_id, payload) VALUES ($1, $2, $3)",
            )
            .bind("kyc_status_changed")
            .bind(payload.user_id)
            .bind(serde_json::json!({
                "new_status": payload.status,
                "reason": payload.reason,
            }))
            .execute(&state.db)
            .await
            .ok();

            Ok((StatusCode::OK, "KYC status updated"))
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Database error")),
    }
}
