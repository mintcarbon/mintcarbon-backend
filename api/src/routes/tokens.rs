use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::Value;

use crate::db::models::{Project, TokenHolding};
use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

#[derive(Deserialize)]
pub struct MintTokenRequest {
    pub project_id: Uuid,
    pub quantity: i64,
}

#[derive(Deserialize)]
pub struct RetireTokenRequest {
    pub token_id: String,
    pub quantity: i64,
    pub reason: String,
}

#[derive(Serialize)]
pub struct TokenHoldingResponse {
    pub token_id: String,
    pub quantity: i64,
    pub project_name: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn mint_token(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(req): Json<MintTokenRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, &'static str)> {
    if auth.role != "issuer" {
        return Err((StatusCode::FORBIDDEN, "Only issuers can mint tokens"));
    }

    let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1")
        .bind(req.project_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::NOT_FOUND, "Project not found"))?;

    if project.status != "verified" {
        return Err((StatusCode::BAD_REQUEST, "Project must be verified to mint tokens"));
    }

    let tx_hash = state.soroban_client.mint_token(&project.id.to_string(), req.quantity)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Soroban RPC error"))?;

    sqlx::query(
        "INSERT INTO token_holdings (user_id, token_id, quantity)
         VALUES ($1, $2, $3)
         ON CONFLICT (user_id, token_id) DO UPDATE SET quantity = token_holdings.quantity + $3",
    )
    .bind(auth.user_id)
    .bind(project.id.to_string())
    .bind(req.quantity)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    sqlx::query(
        "INSERT INTO audit_events (event_type, actor_id, payload)
         VALUES ($1, $2, $3)",
    )
    .bind("token_minted")
    .bind(auth.user_id)
    .bind(serde_json::json!({
        "project_id": project.id,
        "quantity": req.quantity,
        "tx_hash": tx_hash,
    }))
    .execute(&state.db)
    .await
    .ok();

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "tx_hash": tx_hash }))))
}

pub async fn retire_token(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(req): Json<RetireTokenRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, &'static str)> {
    let holding = sqlx::query_as::<_, TokenHolding>(
        "SELECT * FROM token_holdings WHERE user_id = $1 AND token_id = $2"
    )
    .bind(auth.user_id)
    .bind(&req.token_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
    .ok_or((StatusCode::BAD_REQUEST, "Insufficient balance"))?;

    if holding.quantity < req.quantity {
        return Err((StatusCode::BAD_REQUEST, "Insufficient balance"));
    }

    let tx_hash = state.soroban_client.retire_token(&req.token_id, req.quantity)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Soroban RPC error"))?;

    sqlx::query(
        "UPDATE token_holdings SET quantity = quantity - $1 WHERE user_id = $2 AND token_id = $3"
    )
    .bind(req.quantity)
    .bind(auth.user_id)
    .bind(&req.token_id)
    .execute(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    sqlx::query(
        "INSERT INTO audit_events (event_type, actor_id, payload)
         VALUES ($1, $2, $3)",
    )
    .bind("token_retired")
    .bind(auth.user_id)
    .bind(serde_json::json!({
        "token_id": req.token_id,
        "quantity": req.quantity,
        "reason": req.reason,
        "tx_hash": tx_hash,
    }))
    .execute(&state.db)
    .await
    .ok();

    Ok((StatusCode::OK, Json(serde_json::json!({ "tx_hash": tx_hash }))))
}

pub async fn list_tokens(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<Json<Vec<TokenHoldingResponse>>, (StatusCode, &'static str)> {
    let holdings = sqlx::query_as::<_, (String, i64, chrono::DateTime<chrono::Utc>, String)>(
        "SELECT th.token_id, th.quantity, th.updated_at, p.project_name
         FROM token_holdings th
         JOIN projects p ON th.token_id = p.id::text
         WHERE th.user_id = $1"
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    let response = holdings
        .into_iter()
        .map(|(token_id, quantity, updated_at, project_name)| TokenHoldingResponse {
            token_id,
            quantity,
            project_name,
            updated_at,
        })
        .collect();

    Ok(Json(response))
}

pub async fn get_token(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TokenHoldingResponse>, (StatusCode, &'static str)> {
    let token = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT p.id::text as token_id, p.project_name, COALESCE(SUM(th.quantity), 0) as total_supply
         FROM projects p
         LEFT JOIN token_holdings th ON p.id::text = th.token_id
         WHERE p.id::text = $1
         GROUP BY p.id, p.project_name"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
    .ok_or((StatusCode::NOT_FOUND, "Token not found"))?;

    Ok(Json(TokenHoldingResponse {
        token_id: token.0,
        quantity: token.2,
        project_name: token.1,
        updated_at: chrono::Utc::now(),
    }))
}
