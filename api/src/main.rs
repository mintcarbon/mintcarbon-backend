use std::sync::Arc;

use mintcarbon_api::{
    config::Config,
    db::{init_db, run_migrations},
    AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mintcarbon_api=debug".into()),
        )
        .init();

    let config = Config::from_env();

    let pool = init_db(&config.database_url).await;
    run_migrations(&pool).await;

    let kyc_provider = Arc::new(kyc::MockProvider);
    let sanctions_checker = Arc::new(kyc::SanctionsChecker::new(config.sanctions_blocklist));
    let document_store = Arc::new(kyc::DocumentStore::new(
        &config.minio_endpoint,
        &config.minio_access_key,
        &config.minio_secret_key,
        &config.encryption_key,
    ));

    let soroban_client = Arc::new(mintcarbon_api::soroban::RealSorobanClient {
        rpc_url: config.soroban_rpc_url,
    });

    let state = AppState {
        db: pool.clone(),
        kyc_provider,
        sanctions_checker,
        document_store,
        soroban_client,
    };

    // Day 8: Start Price Oracle Poller
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let price_store = oracle::PriceStore::new(pool_clone);
        let publishers: Vec<Box<dyn oracle::OraclePublisher>> = vec![
            Box::new(oracle::ToucanOracle),
            Box::new(oracle::CblOracle),
            Box::new(oracle::IceOracle),
        ];

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            for publisher in &publishers {
                if let Ok(price) = publisher.fetch_price("placeholder_token").await {
                    let _ = price_store.insert(price).await;
                }
            }
        }
    });

    let app = mintcarbon_api::create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("MintCarbon API listening on 0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
