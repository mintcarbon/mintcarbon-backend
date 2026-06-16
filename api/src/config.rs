use std::env;

pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub encryption_key: String,
    pub minio_endpoint: String,
    pub minio_access_key: String,
    pub minio_secret_key: String,
    pub soroban_rpc_url: String,
    pub sanctions_blocklist: Vec<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://mintcarbon:mintcarbon_dev@localhost:5432/mintcarbon".into()),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into()),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-in-production".into()),
            encryption_key: env::var("ENCRYPTION_KEY")
                .unwrap_or_else(|_| "01234567890123456789012345678901".into()),
            minio_endpoint: env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".into()),
            minio_access_key: env::var("MINIO_ACCESS_KEY").unwrap_or_else(|_| "mintcarbon".into()),
            minio_secret_key: env::var("MINIO_SECRET_KEY").unwrap_or_else(|_| "mintcarbon_dev".into()),
            soroban_rpc_url: env::var("SOROBAN_RPC_URL")
                .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".into()),
            sanctions_blocklist: env::var("SANCTIONS_BLOCKLIST")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect(),
        }
    }
}
