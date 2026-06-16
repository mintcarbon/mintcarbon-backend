pub mod config;
pub mod db;
pub mod middleware;
pub mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub kyc_provider: std::sync::Arc<dyn kyc::Provider>,
    pub sanctions_checker: std::sync::Arc<kyc::SanctionsChecker>,
    pub document_store: std::sync::Arc<kyc::DocumentStore>,
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
        .route("/api/v1/kyc/webhook", post(routes::kyc::kyc_webhook));

    public_routes.merge(protected_routes).with_state(state)
}
