use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::db::models::Order;
use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

#[derive(Serialize)]
pub struct PortfolioHolding {
    pub token_id: String,
    pub quantity: i64,
    pub project_name: String,
    pub registry: String,
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn get_portfolio(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<Json<Vec<PortfolioHolding>>, (StatusCode, &'static str)> {
    let holdings = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT th.token_id, th.quantity, p.project_name, p.registry
         FROM token_holdings th
         JOIN projects p ON th.token_id = p.id::text
         WHERE th.user_id = $1",
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    let response = holdings
        .into_iter()
        .map(
            |(token_id, quantity, project_name, registry)| PortfolioHolding {
                token_id,
                quantity,
                project_name,
                registry,
            },
        )
        .collect();

    Ok(Json(response))
}

pub async fn get_history(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<Order>>, (StatusCode, &'static str)> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let orders = sqlx::query_as::<_, Order>(
        "SELECT * FROM orders WHERE buyer_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(auth.user_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    Ok(Json(orders))
}

pub async fn export_portfolio(
    State(_state): State<AppState>,
    _auth: AuthenticatedUser,
) -> Result<Response, (StatusCode, &'static str)> {
    // In a real app, this would stream CSV from the DB
    let csv = "token_id,quantity,project_name,registry\n".to_string();

    Response::builder()
        .header("Content-Type", "text/csv")
        .header(
            "Content-Disposition",
            "attachment; filename=\"portfolio.csv\"",
        )
        .body(axum::body::Body::from(csv))
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to build response",
            )
        })
}
