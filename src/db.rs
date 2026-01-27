use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub env: crate::config::Config,
}

pub async fn connection_pool(config: &crate::config::Config) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await
        .expect("Failed to create pool")
}
