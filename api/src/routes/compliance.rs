use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

#[derive(Deserialize)]
pub struct ReportQuery {
    pub from: chrono::DateTime<chrono::Utc>,
    pub to: chrono::DateTime<chrono::Utc>,
    pub format: Option<String>,
}

pub async fn get_reports(
    State(_state): State<AppState>,
    auth: AuthenticatedUser,
    Query(query): Query<ReportQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    if auth.role != "compliance_officer" && auth.role != "administrator" {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    let report = compliance::ReportGenerator::generate_json(query.from, query.to);
    Ok(Json(report))
}

pub async fn get_proof(
    State(_state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, &'static str)> {
    if auth.role != "compliance_officer" && auth.role != "administrator" {
        return Err((StatusCode::FORBIDDEN, "Access denied"));
    }

    // In a real app, this would fetch Merkle proof from on-chain AuditLog
    Ok(Json(serde_json::json!({
        "entry_id": id,
        "proof": ["hash1", "hash2", "hash3"],
        "root": "merkle_root_hash"
    })))
}
