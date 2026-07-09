use anyhow::Result;
// use sqlx::ConnectOptions;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};
use std::str::FromStr;
use std::time::Duration;
// use tracing::log::LevelFilter;

/// Base connect options shared by every pool.
///
/// Returns options with all per-connection pragmas configured. sqlx re-applies
/// these every time it opens a new connection, including replacement connections
/// after idle timeout — which is the behavior you want.
fn base_options(database_url: &str, busy_timeout: Duration) -> Result<SqliteConnectOptions> {
    Ok(
        SqliteConnectOptions::from_str(database_url)?
            .busy_timeout(busy_timeout)
            .foreign_keys(true)
            .pragma("wal_autocheckpoint", "1000") // explicit
            .pragma("journal_size_limit", "67108864")
            .pragma("cache_size", "-20000")
            .pragma("temp_store", "memory"), // .log_statements(LevelFilter::Debug)
    )
}

/// Read-only pool, optimized for concurrent reads.
///
/// Sized to CPU cores. Does NOT set `journal_mode` or `synchronous` — those are
/// write-side concerns and `PRAGMA journal_mode = WAL` would fail on a read-only
/// connection anyway. The DB file's journal mode is set by the write pool.
pub async fn create_read_pool(database_url: &str, max_connections: u32) -> Result<SqlitePool> {
    let options = base_options(database_url, Duration::from_millis(5000))?.read_only(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;

    tracing::info!(
        "Created read-only pool with {} max connections",
        max_connections
    );
    Ok(pool)
}

/// Read-write pool, single connection to serialize writes and avoid SQLITE_BUSY.
///
/// All write transactions go through this pool. Use `BEGIN IMMEDIATE` for any
/// transaction that will write, so it grabs the reserved lock up front instead
/// of upgrading mid-transaction (which is what causes most BUSY errors).
pub async fn create_write_pool(database_url: &str) -> Result<SqlitePool> {
    let options = base_options(database_url, Duration::from_millis(5000))?
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?;

    tracing::info!("Created read-write pool with 1 max connection");
    Ok(pool)
}

/// Standard pool for CLI commands (migrate, import, tests).
///
/// Single-pool setup, so it owns the WAL/synchronous settings.
///
/// Uses a long `busy_timeout` because migrate/reset can open a database with a large
/// leftover `-wal` from a previous unclean shutdown; recovering/checkpointing it on slow
/// network storage (e.g. Longhorn) can take well over the 5s the serve pools use.
pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<SqlitePool> {
    let options = base_options(database_url, Duration::from_secs(60))?
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;

    tracing::info!("Created pool with {} max connections", max_connections);
    Ok(pool)
}
