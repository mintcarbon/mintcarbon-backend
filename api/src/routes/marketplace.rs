use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::db::models::{Listing, Order, TokenHolding};
use crate::middleware::auth::AuthenticatedUser;
use crate::AppState;

#[derive(Deserialize)]
pub struct CreateListingRequest {
    pub token_id: String,
    pub quantity: i64,
    pub price_lumens: i64,
}

#[derive(Deserialize)]
pub struct PlaceOrderRequest {
    pub listing_id: Uuid,
    pub quantity: i64,
}

#[derive(Deserialize)]
pub struct ListingListQuery {
    pub token_id: Option<String>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Serialize)]
pub struct MarketDataResponse {
    pub best_ask: Option<i64>,
    pub volume_24h: i64,
    pub last_price: Option<i64>,
}

pub async fn create_listing(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(req): Json<CreateListingRequest>,
) -> Result<(StatusCode, Json<Listing>), (StatusCode, &'static str)> {
    let holding = sqlx::query_as::<_, TokenHolding>(
        "SELECT * FROM token_holdings WHERE user_id = $1 AND token_id = $2",
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

    let tx_hash = state
        .soroban_client
        .create_listing(&req.token_id, req.quantity, req.price_lumens)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Soroban RPC error"))?;

    let listing = sqlx::query_as::<_, Listing>(
        "INSERT INTO listings (seller_id, token_id, quantity, price_lumens, status, contract_listing_id)
         VALUES ($1, $2, $3, $4, 'active', $5) RETURNING *"
    )
    .bind(auth.user_id)
    .bind(&req.token_id)
    .bind(req.quantity)
    .bind(req.price_lumens)
    .bind(&tx_hash) // Using tx_hash as placeholder for contract_listing_id
    .fetch_one(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    sqlx::query(
        "INSERT INTO audit_events (event_type, actor_id, payload)
         VALUES ($1, $2, $3)",
    )
    .bind("listing_created")
    .bind(auth.user_id)
    .bind(serde_json::json!({
        "listing_id": listing.id,
        "token_id": req.token_id,
        "quantity": req.quantity,
        "price": req.price_lumens,
    }))
    .execute(&state.db)
    .await
    .ok();

    Ok((StatusCode::CREATED, Json(listing)))
}

pub async fn cancel_listing(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let listing = sqlx::query_as::<_, Listing>("SELECT * FROM listings WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::NOT_FOUND, "Listing not found"))?;

    if listing.seller_id != auth.user_id {
        return Err((StatusCode::FORBIDDEN, "Not your listing"));
    }

    if listing.status != "active" {
        return Err((StatusCode::BAD_REQUEST, "Listing is not active"));
    }

    state
        .soroban_client
        .cancel_listing(listing.contract_listing_id.as_deref().unwrap_or(""))
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Soroban RPC error"))?;

    sqlx::query("UPDATE listings SET status = 'cancelled' WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn place_order(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(req): Json<PlaceOrderRequest>,
) -> Result<(StatusCode, Json<Order>), (StatusCode, &'static str)> {
    let listing = sqlx::query_as::<_, Listing>("SELECT * FROM listings WHERE id = $1")
        .bind(req.listing_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::NOT_FOUND, "Listing not found"))?;

    if listing.status != "active" {
        return Err((StatusCode::BAD_REQUEST, "Listing is no longer active"));
    }

    if listing.quantity < req.quantity {
        return Err((
            StatusCode::BAD_REQUEST,
            "Requested quantity exceedes listing",
        ));
    }

    let tx_hash = state
        .soroban_client
        .place_order(
            listing.contract_listing_id.as_deref().unwrap_or(""),
            req.quantity,
        )
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Soroban RPC error"))?;

    let total_lumens = req.quantity * listing.price_lumens;

    let order = sqlx::query_as::<_, Order>(
        "INSERT INTO orders (buyer_id, listing_id, quantity, total_lumens, tx_hash)
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(auth.user_id)
    .bind(listing.id)
    .bind(req.quantity)
    .bind(total_lumens)
    .bind(&tx_hash)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    // Update listing quantity or close it
    if listing.quantity == req.quantity {
        sqlx::query("UPDATE listings SET quantity = 0, status = 'filled' WHERE id = $1")
            .bind(listing.id)
            .execute(&state.db)
            .await
            .ok();
    } else {
        sqlx::query("UPDATE listings SET quantity = quantity - $1 WHERE id = $2")
            .bind(req.quantity)
            .bind(listing.id)
            .execute(&state.db)
            .await
            .ok();
    }

    sqlx::query(
        "INSERT INTO audit_events (event_type, actor_id, payload)
         VALUES ($1, $2, $3)",
    )
    .bind("order_placed")
    .bind(auth.user_id)
    .bind(serde_json::json!({
        "order_id": order.id,
        "listing_id": listing.id,
        "quantity": req.quantity,
        "total_lumens": total_lumens,
        "tx_hash": tx_hash,
    }))
    .execute(&state.db)
    .await
    .ok();

    Ok((StatusCode::CREATED, Json(order)))
}

pub async fn list_listings(
    State(state): State<AppState>,
    Query(query): Query<ListingListQuery>,
) -> Result<Json<Vec<Listing>>, (StatusCode, &'static str)> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let mut sql = String::from("SELECT * FROM listings WHERE status = 'active'");
    if let Some(token_id) = &query.token_id {
        sql.push_str(&format!(" AND token_id = '{}'", token_id));
    }
    if let Some(min_price) = query.min_price {
        sql.push_str(&format!(" AND price_lumens >= {}", min_price));
    }
    if let Some(max_price) = query.max_price {
        sql.push_str(&format!(" AND price_lumens <= {}", max_price));
    }
    sql.push_str(&format!(
        " ORDER BY created_at DESC LIMIT {} OFFSET {}",
        per_page, offset
    ));

    let listings = sqlx::query_as::<_, Listing>(&sql)
        .fetch_all(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    Ok(Json(listings))
}

pub async fn get_market_data(
    State(state): State<AppState>,
    Query(query): Query<Value>,
) -> Result<Json<MarketDataResponse>, (StatusCode, &'static str)> {
    let token_id = query.get("token_id").and_then(|v| v.as_str());

    let mut best_ask_q =
        String::from("SELECT MIN(price_lumens) FROM listings WHERE status = 'active'");
    let mut volume_q = String::from(
        "SELECT SUM(total_lumens) FROM orders WHERE created_at > NOW() - INTERVAL '24 hours'",
    );
    let mut last_price_q = String::from("SELECT total_lumens / quantity FROM orders");

    if let Some(tid) = token_id {
        best_ask_q.push_str(&format!(" AND token_id = '{}'", tid));
        volume_q.push_str(&format!(
            " AND listing_id IN (SELECT id FROM listings WHERE token_id = '{}')",
            tid
        ));
        last_price_q.push_str(&format!(
            " WHERE listing_id IN (SELECT id FROM listings WHERE token_id = '{}')",
            tid
        ));
    }
    last_price_q.push_str(" ORDER BY created_at DESC LIMIT 1");

    let best_ask: Option<i64> = sqlx::query_scalar(&best_ask_q)
        .fetch_one(&state.db)
        .await
        .unwrap_or(None);
    let volume: Option<i64> = sqlx::query_scalar(&volume_q)
        .fetch_one(&state.db)
        .await
        .unwrap_or(None);
    let last_price: Option<i64> = sqlx::query_scalar(&last_price_q)
        .fetch_one(&state.db)
        .await
        .unwrap_or(None);

    Ok(Json(MarketDataResponse {
        best_ask,
        volume_24h: volume.unwrap_or(0),
        last_price,
    }))
}
