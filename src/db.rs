use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{ConnectOptions, SqlitePool};
use std::str::FromStr;
use tracing::log::LevelFilter;

/// Configure SQLite PRAGMAs for optimal performance with WAL mode
///
/// Based on best practices:
/// - WAL mode enables concurrent reads and writes
/// - busy_timeout reduces SQLITE_BUSY errors
/// - synchronous=NORMAL is safe with WAL and improves performance
/// - cache_size increases memory cache for better performance
/// - foreign_keys must be explicitly enabled (disabled by default)
/// - temp_store=memory speeds up temporary table operations
async fn configure_pragmas(pool: &SqlitePool) -> Result<()> {
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(pool)
        .await?;
    sqlx::query("PRAGMA busy_timeout = 5000")
        .execute(pool)
        .await?;
    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(pool)
        .await?;
    sqlx::query("PRAGMA cache_size = -20000")
        .execute(pool)
        .await?;
    sqlx::query("PRAGMA foreign_keys = true")
        .execute(pool)
        .await?;
    sqlx::query("PRAGMA temp_store = memory")
        .execute(pool)
        .await?;

    Ok(())
}

/// Create a read-only connection pool optimized for concurrent reads
///
/// This pool is configured for read-only queries and can have multiple connections
/// to maximize read throughput. The connection limit should be set based on CPU cores.
pub async fn create_read_pool(database_url: &str, max_connections: u32) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .read_only(true)
        .log_statements(LevelFilter::Debug);

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;

    configure_pragmas(&pool).await?;

    tracing::info!(
        "Created read-only pool with {} max connections",
        max_connections
    );

    Ok(pool)
}

/// Create a read-write connection pool optimized for writes
///
/// This pool is limited to 1 connection to avoid SQLITE_BUSY errors on writes.
/// All write operations and transactions should use this pool.
/// For immediate transactions, use BEGIN IMMEDIATE to avoid SQLITE_BUSY.
pub async fn create_write_pool(database_url: &str) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?.log_statements(LevelFilter::Debug);

    let pool = SqlitePoolOptions::new()
        .max_connections(1) // CRITICAL: Single connection for writes to avoid SQLITE_BUSY
        .connect_with(options)
        .await?;

    configure_pragmas(&pool).await?;

    tracing::info!("Created read-write pool with 1 max connection");

    Ok(pool)
}

/// Create a standard pool with optimized PRAGMAs
///
/// This is used for simpler setups where read/write separation is not needed,
/// such as CLI commands (migrate, import, etc.) or test environments.
pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?.log_statements(LevelFilter::Debug);

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;

    configure_pragmas(&pool).await?;

    tracing::info!("Created pool with {} max connections", max_connections);

    Ok(pool)
}
