pub mod models;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub type DbPool = PgPool;

pub async fn init_db(database_url: &str) -> DbPool {
    PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
        .expect("Failed to connect to database")
}

pub async fn run_migrations(pool: &DbPool) {
    sqlx::migrate!("../migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations");
}
