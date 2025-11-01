//! Database migration utilities

use imkitchen::Config;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::path::Path;
use std::str::FromStr;

/// Run all database migrations
pub async fn migrate(config: &Config) -> anyhow::Result<()> {
    tracing::info!("Migrating evento database");
    migrate_evento(config).await?;

    tracing::info!("Migrating queries database");
    migrate_queries(config).await?;

    tracing::info!("Migrating validation database");
    migrate_validation(config).await?;

    Ok(())
}

/// Drop all databases and run migrations
pub async fn reset(config: &Config) -> anyhow::Result<()> {
    tracing::info!("Dropping databases");

    // Remove database files if they exist
    for db in [
        &config.database.evento_db,
        &config.database.queries_db,
        &config.database.validation_db,
    ] {
        if Path::new(db).exists() {
            std::fs::remove_file(db)?;
            tracing::info!("Dropped database: {}", db);
        }
    }

    // Run migrations to recreate
    migrate(config).await?;

    Ok(())
}

/// Migrate evento database
async fn migrate_evento(config: &Config) -> anyhow::Result<()> {
    // Create database if it doesn't exist
    let options =
        SqliteConnectOptions::from_str(&config.database.evento_db)?.create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Manually create evento tables since the API doesn't expose migrations easily
    // These are the standard evento tables from sql_migrator.rs
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS event (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            aggregator_type TEXT NOT NULL,
            aggregator_id TEXT NOT NULL,
            version INTEGER NOT NULL,
            data BLOB NOT NULL,
            metadata BLOB NOT NULL,
            routing_key TEXT,
            timestamp INTEGER NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Create indices
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_event_type ON event(aggregator_type);")
        .execute(&pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_event_type_id ON event(aggregator_type, aggregator_id);",
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_event_routing_key_type ON event(routing_key, aggregator_type);")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_event_type_id_version ON event(aggregator_type, aggregator_id, version);")
        .execute(&pool)
        .await?;

    // Create snapshot table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS snapshot (
            id TEXT NOT NULL,
            type TEXT NOT NULL,
            cursor TEXT NOT NULL,
            revision TEXT NOT NULL,
            data BLOB NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMP,
            PRIMARY KEY (type, id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Create subscriber table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS subscriber (
            key TEXT PRIMARY KEY NOT NULL,
            worker_id TEXT NOT NULL,
            cursor TEXT,
            lag INTEGER NOT NULL,
            enabled BOOLEAN DEFAULT 1 NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await?;

    pool.close().await;

    tracing::info!("Evento database initialized");

    Ok(())
}

/// Migrate queries database using sqlx::migrate!
async fn migrate_queries(config: &Config) -> anyhow::Result<()> {
    // Create database if it doesn't exist
    let options =
        SqliteConnectOptions::from_str(&config.database.queries_db)?.create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run sqlx migrations from migrations/queries directory
    sqlx::migrate!("./migrations/queries").run(&pool).await?;

    pool.close().await;

    Ok(())
}

/// Migrate validation database using sqlx::migrate!
async fn migrate_validation(config: &Config) -> anyhow::Result<()> {
    // Create database if it doesn't exist
    let options =
        SqliteConnectOptions::from_str(&config.database.validation_db)?.create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Run sqlx migrations from migrations/validation directory
    sqlx::migrate!("./migrations/validation").run(&pool).await?;

    pool.close().await;

    Ok(())
}
