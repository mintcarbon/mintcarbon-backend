pub mod config;
pub mod db;
pub mod middleware;
pub mod routes;
pub mod soroban;

use axum::{
    routing::{get, post, delete},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub kyc_provider: Arc<dyn kyc::Provider>,
    pub sanctions_checker: Arc<kyc::SanctionsChecker>,
    pub document_store: Arc<kyc::DocumentStore>,
    pub soroban_client: Arc<dyn soroban::SorobanClient>,
}

pub fn create_router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/api/v1/auth/register", post(routes::auth::register))
        .route("/api/v1/auth/login", post(routes::auth::login));

    let protected_routes = Router::new()
        .route("/api/v1/auth/mfa/setup", post(routes::auth::mfa_setup))
        .route("/api/v1/auth/mfa/verify", post(routes::auth::mfa_verify))
        .route("/api/v1/projects", post(routes::projects::create_project))
        .route("/api/v1/projects", get(routes::projects::list_projects))
        .route("/api/v1/projects/:id", get(routes::projects::get_project))
        .route("/api/v1/kyc/webhook", post(routes::kyc::kyc_webhook))
        .route("/api/v1/tokens/mint", post(routes::tokens::mint_token))
        .route("/api/v1/tokens/retire", post(routes::tokens::retire_token))
        .route("/api/v1/tokens", get(routes::tokens::list_tokens))
        .route("/api/v1/tokens/:id", get(routes::tokens::get_token))
        .route("/api/v1/listings", post(routes::marketplace::create_listing))
        .route("/api/v1/listings", get(routes::marketplace::list_listings))
        .route("/api/v1/listings/:id", delete(routes::marketplace::cancel_listing))
        .route("/api/v1/orders", post(routes::marketplace::place_order))
        .route("/api/v1/market/data", get(routes::marketplace::get_market_data))
        .route("/api/v1/portfolio", get(routes::portfolio::get_portfolio))
        .route("/api/v1/portfolio/history", get(routes::portfolio::get_history))
        .route("/api/v1/portfolio/export", get(routes::portfolio::export_portfolio))
        .route("/api/v1/compliance/reports", get(routes::compliance::get_reports))
        .route("/api/v1/compliance/proofs/:id", get(routes::compliance::get_proof));

    public_routes.merge(protected_routes).with_state(state)
}
