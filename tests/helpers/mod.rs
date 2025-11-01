//! Test helper functions for database setup and teardown
//!
//! This module provides reusable utilities for setting up test databases
//! following DRY principles as specified in CLAUDE.md testing guidelines.

use sqlx::SqlitePool;

/// Test database configuration
pub struct TestDatabases {
    pub evento: evento::Sqlite,
    pub queries: SqlitePool,
    pub validation: SqlitePool,
}

// Allow dead code for evento field - it's used through the Executor trait
#[allow(dead_code)]
impl TestDatabases {
    /// Get a reference to the evento executor
    pub fn evento(&self) -> &evento::Sqlite {
        &self.evento
    }
}

/// Set up test databases with migrations
///
/// Creates in-memory SQLite databases for testing and runs all migrations.
/// This is the recommended way to set up databases in tests.
///
/// # Examples
///
/// ```no_run
/// use tests::helpers;
///
/// #[tokio::test]
/// async fn test_something() {
///     let dbs = helpers::setup_test_databases().await.unwrap();
///     // Your test code here
///     helpers::cleanup_test_databases(dbs).await.unwrap();
/// }
/// ```
pub async fn setup_test_databases() -> anyhow::Result<TestDatabases> {
    // Create in-memory evento database
    let evento_pool = SqlitePool::connect("sqlite::memory:").await?;
    setup_evento_schema(&evento_pool).await?;
    let evento: evento::Sqlite = evento_pool.into();

    // Create in-memory queries database
    let queries = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::migrate!("./migrations/queries").run(&queries).await?;

    // Create in-memory validation database
    let validation = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::migrate!("./migrations/validation")
        .run(&validation)
        .await?;

    Ok(TestDatabases {
        evento,
        queries,
        validation,
    })
}

/// Set up evento schema in a pool
///
/// Creates all evento tables and indices needed for event sourcing.
async fn setup_evento_schema(pool: &SqlitePool) -> anyhow::Result<()> {
    // Create event table
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
    .execute(pool)
    .await?;

    // Create indices
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_event_type ON event(aggregator_type);")
        .execute(pool)
        .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_event_type_id ON event(aggregator_type, aggregator_id);",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_event_routing_key_type ON event(routing_key, aggregator_type);")
        .execute(pool)
        .await?;

    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_event_type_id_version ON event(aggregator_type, aggregator_id, version);")
        .execute(pool)
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
    .execute(pool)
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
    .execute(pool)
    .await?;

    Ok(())
}

/// Clean up test databases
///
/// Closes all database connections. Since we use in-memory databases,
/// they will be automatically cleaned up when connections are closed.
pub async fn cleanup_test_databases(dbs: TestDatabases) -> anyhow::Result<()> {
    dbs.queries.close().await;
    dbs.validation.close().await;
    Ok(())
}

/// Create a test configuration
///
/// Returns a test configuration with in-memory database paths.
pub fn create_test_config() -> imkitchen::Config {
    imkitchen::Config {
        server: imkitchen::config::ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3001, // Different port to avoid conflicts
        },
        database: imkitchen::config::DatabaseConfig {
            evento_db: ":memory:".to_string(),
            queries_db: ":memory:".to_string(),
            validation_db: ":memory:".to_string(),
        },
        logging: imkitchen::config::LoggingConfig {
            level: "debug".to_string(),
            format: "pretty".to_string(),
        },
    }
}
