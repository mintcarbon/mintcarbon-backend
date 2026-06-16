use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    prices: HashMap<String, Vec<PricePoint>>,
}

impl PriceStore {
    pub fn new() -> Self {
        Self {
            prices: HashMap::new(),
        }
    }

    pub fn insert(&mut self, point: PricePoint) {
        self.prices
            .entry(point.token_id.clone())
            .or_default()
            .push(point);
    }

    pub fn history(&self, token_id: &str) -> Vec<&PricePoint> {
        self.prices
            .get(token_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
}
