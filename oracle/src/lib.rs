use sqlx::PgPool;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct PricePoint {
    pub token_id: String,
    pub price_lumens: i64,
    pub source: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[async_trait::async_trait]
pub trait OraclePublisher: Send + Sync {
    async fn fetch_price(&self, token_id: &str) -> anyhow::Result<PricePoint>;
}

pub struct ToucanOracle;

#[async_trait::async_trait]
impl OraclePublisher for ToucanOracle {
    async fn fetch_price(&self, token_id: &str) -> anyhow::Result<PricePoint> {
        Ok(PricePoint {
            token_id: token_id.to_string(),
            price_lumens: 1000,
            source: "toucan".into(),
            timestamp: chrono::Utc::now(),
        })
    }
}

pub struct CblOracle;

#[async_trait::async_trait]
impl OraclePublisher for CblOracle {
    async fn fetch_price(&self, token_id: &str) -> anyhow::Result<PricePoint> {
        Ok(PricePoint {
            token_id: token_id.to_string(),
            price_lumens: 1005,
            source: "cbl".into(),
            timestamp: chrono::Utc::now(),
        })
    }
}

pub struct IceOracle;

#[async_trait::async_trait]
impl OraclePublisher for IceOracle {
    async fn fetch_price(&self, token_id: &str) -> anyhow::Result<PricePoint> {
        Ok(PricePoint {
            token_id: token_id.to_string(),
            price_lumens: 998,
            source: "ice".into(),
            timestamp: chrono::Utc::now(),
        })
    }
}

pub struct PriceStore {
    pool: PgPool,
}

impl PriceStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, point: PricePoint) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO audit_events (event_type, payload) VALUES ($1, $2)")
            .bind("price_update")
            .bind(serde_json::to_value(&point)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn history(&self, token_id: &str) -> anyhow::Result<Vec<PricePoint>> {
        // Since we don't have a separate price_history table in migrations yet,
        // we'll query audit_events
        let events = sqlx::query_as::<_, (serde_json::Value,)>(
            "SELECT payload FROM audit_events WHERE event_type = 'price_update' AND payload->>'token_id' = $1"
        )
        .bind(token_id)
        .fetch_all(&self.pool)
        .await?;

        let mut prices = Vec::new();
        for (payload,) in events {
            if let Ok(price) = serde_json::from_value::<PricePoint>(payload) {
                prices.push(price);
            }
        }
        Ok(prices)
    }
}
