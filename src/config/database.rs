use sqlx::{Pool, Postgres, PgPool};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

pub async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").fetch_one(pool).await?;
    Ok(())
}