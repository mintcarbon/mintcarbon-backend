use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting MintCarbon Indexer");

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://mintcarbon:mintcarbon_dev@localhost:5432/mintcarbon".into()
    });
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let _soroban_rpc_url = std::env::var("SOROBAN_RPC_URL")
        .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let client = redis::Client::open(redis_url)?;
    let mut redis_conn = client.get_async_connection().await?;

    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        // Get last ledger from redis
        let last_ledger: u32 = redis::cmd("GET")
            .arg("last_indexed_ledger")
            .query_async(&mut redis_conn)
            .await
            .unwrap_or(0);

        info!("Polling Soroban events from ledger {}", last_ledger);

        // In a real implementation, we would call Soroban RPC getEvents here
        // For this task, we'll simulate finding some events
        let events = simulate_events(last_ledger);

        for event in events {
            info!("Processing event: {:?}", event);

            sqlx::query(
                "INSERT INTO audit_events (event_type, payload, on_chain_index)
                 VALUES ($1, $2, $3)
                 ON CONFLICT DO NOTHING",
            )
            .bind(&event.event_type)
            .bind(&event.payload)
            .bind(event.ledger as i64)
            .execute(&pool)
            .await?;
        }

        // Update last ledger in redis
        let next_ledger = last_ledger + 1;
        let _: () = redis::cmd("SET")
            .arg("last_indexed_ledger")
            .arg(next_ledger)
            .query_async(&mut redis_conn)
            .await?;
    }
}

#[derive(Debug)]
struct SorobanEvent {
    event_type: String,
    payload: serde_json::Value,
    ledger: u32,
}

fn simulate_events(ledger: u32) -> Vec<SorobanEvent> {
    // Simulate one event every 10 ledgers
    if ledger % 10 == 0 {
        vec![SorobanEvent {
            event_type: "on_chain_audit".to_string(),
            payload: serde_json::json!({
                "ledger": ledger,
                "msg": "Simulated on-chain event"
            }),
            ledger,
        }]
    } else {
        vec![]
    }
}
