//! Test helper functions for database setup and teardown
//!
//! This module provides reusable utilities for setting up test databases
//! following DRY principles as specified in CLAUDE.md testing guidelines.

#![allow(dead_code)]

use evento::migrator::{Migrate, Plan};
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
/// Creates all evento tables and indices needed for event sourcing using evento::sql_migrator.
async fn setup_evento_schema(pool: &SqlitePool) -> anyhow::Result<()> {
    let migrator = evento::sql_migrator::new_migrator::<sqlx::Sqlite>()?;
    let mut conn = pool.acquire().await?;
    migrator.run(&mut *conn, &Plan::apply_all()).await?;

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
        auth: imkitchen::config::AuthConfig {
            jwt_secret: "test_secret_for_testing_only".to_string(),
            jwt_lifetime_seconds: 3600,
        },
    }
}
