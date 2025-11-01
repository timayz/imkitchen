//! Database migration utilities

use evento::migrator::{Migrate, Plan};
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

/// Migrate evento database using evento::sql_migrator
async fn migrate_evento(config: &Config) -> anyhow::Result<()> {
    // Create database if it doesn't exist
    let options =
        SqliteConnectOptions::from_str(&config.database.evento_db)?.create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;

    // Use evento's migrator to create tables
    let migrator = evento::sql_migrator::new_migrator::<sqlx::Sqlite>()?;
    let mut conn = pool.acquire().await?;
    migrator.run(&mut *conn, &Plan::apply_all()).await?;

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
