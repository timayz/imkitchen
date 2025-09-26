use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::time::Duration;
use tracing::{error, info, warn};

/// Database connection configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub acquire_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:imkitchen.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600), // 10 minutes
            max_lifetime: Duration::from_secs(3600), // 1 hour
            acquire_timeout: Duration::from_secs(30),
        }
    }
}

impl DatabaseConfig {
    pub fn from_url(url: String) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }

    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }

    pub fn with_timeouts(mut self, connect_timeout: Duration, acquire_timeout: Duration) -> Self {
        self.connect_timeout = connect_timeout;
        self.acquire_timeout = acquire_timeout;
        self
    }
}

/// Create a database connection pool with health checks and retry logic
pub async fn create_database_pool(config: &DatabaseConfig) -> Result<SqlitePool, sqlx::Error> {
    info!("Creating database connection pool for: {}", config.url);

    // Ensure database file exists for SQLite
    if let Err(e) = ensure_database_exists(&config.url).await {
        error!("Failed to ensure database exists: {}", e);
        return Err(sqlx::Error::Configuration(
            format!("Database setup failed: {}", e).into(),
        ));
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .test_before_acquire(true) // Enable health checks on acquire
        .connect(&config.url)
        .await?;

    info!(
        "Database connection pool created successfully with {} max connections",
        config.max_connections
    );

    // Test the connection pool
    match test_database_connection(&pool).await {
        Ok(_) => {
            info!("Database connection pool health check passed");
        }
        Err(e) => {
            warn!("Database connection pool health check failed: {}", e);
            // Don't fail pool creation, but log the warning
        }
    }

    Ok(pool)
}

/// Create database pool with retry logic
pub async fn create_database_pool_with_retry(
    config: &DatabaseConfig,
    max_retries: u32,
    retry_delay: Duration,
) -> Result<SqlitePool, sqlx::Error> {
    let mut attempts = 0;
    let mut last_error = None;

    while attempts < max_retries {
        attempts += 1;

        match create_database_pool(config).await {
            Ok(pool) => {
                if attempts > 1 {
                    info!(
                        "Database connection established after {} attempts",
                        attempts
                    );
                }
                return Ok(pool);
            }
            Err(e) => {
                last_error = Some(e);
                if attempts < max_retries {
                    warn!(
                        "Database connection attempt {} failed, retrying in {:?}",
                        attempts, retry_delay
                    );
                    tokio::time::sleep(retry_delay).await;
                } else {
                    error!("Database connection failed after {} attempts", attempts);
                }
            }
        }
    }

    Err(last_error.unwrap())
}

/// Ensure database file exists (for SQLite)
async fn ensure_database_exists(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = database_url.strip_prefix("sqlite:") {
        let db_path = if path.starts_with('/') {
            path.to_string()
        } else {
            let current_dir = std::env::current_dir()?;
            current_dir.join(path).to_string_lossy().to_string()
        };

        let path = std::path::Path::new(&db_path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
                info!("Created database directory: {:?}", parent);
            }
        }

        if !path.exists() {
            std::fs::File::create(path)?;
            info!("Created database file: {}", db_path);
        }
    }

    Ok(())
}

/// Test database connection with a simple query
pub async fn test_database_connection(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let result = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await?;

    if result == 1 {
        Ok(())
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}

/// Get database connection pool stats
pub fn get_pool_stats(pool: &SqlitePool) -> DatabaseStats {
    let size = pool.size() as u32;
    let idle = pool.num_idle() as u32;

    DatabaseStats {
        size,
        idle,
        used: size.saturating_sub(idle),
        is_closed: pool.is_closed(),
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub size: u32,
    pub idle: u32,
    pub used: u32,
    pub is_closed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_config_creation() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
    }

    #[tokio::test]
    async fn test_database_config_builder() {
        let config = DatabaseConfig::from_url("sqlite:test.db".to_string())
            .with_max_connections(5)
            .with_timeouts(Duration::from_secs(10), Duration::from_secs(5));

        assert_eq!(config.url, "sqlite:test.db");
        assert_eq!(config.max_connections, 5);
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.acquire_timeout, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_in_memory_database_pool() {
        let config = DatabaseConfig::from_url("sqlite::memory:".to_string());
        let pool = create_database_pool(&config).await;
        assert!(pool.is_ok());

        if let Ok(pool) = pool {
            let test_result = test_database_connection(&pool).await;
            assert!(test_result.is_ok());

            let stats = get_pool_stats(&pool);
            assert!(stats.size > 0);
        }
    }

    #[tokio::test]
    async fn test_database_pool_retry_logic() {
        // Test with invalid URL to trigger retry logic
        let config = DatabaseConfig::from_url("sqlite:/invalid/path/test.db".to_string());
        let result = create_database_pool_with_retry(&config, 2, Duration::from_millis(10)).await;

        // Should fail after retries
        assert!(result.is_err());
    }
}
