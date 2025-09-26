use std::time::Duration;
use tokio::signal;
use tracing::{info, warn};

/// Graceful shutdown manager that handles system signals and resource cleanup
pub struct GracefulShutdown {
    shutdown_timeout: Duration,
}

impl GracefulShutdown {
    /// Create a new graceful shutdown manager with the specified timeout
    pub fn new(shutdown_timeout: Duration) -> Self {
        Self { shutdown_timeout }
    }

    /// Wait for shutdown signal (SIGTERM, SIGINT, or Ctrl+C)
    /// Returns when a shutdown signal is received
    pub async fn wait_for_signal(&self) {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                info!("Received Ctrl+C (SIGINT) signal");
            },
            _ = terminate => {
                info!("Received SIGTERM signal");
            },
        }

        info!("Shutdown signal received, initiating graceful shutdown");
    }

    /// Execute graceful shutdown with resource cleanup
    pub async fn shutdown_with_cleanup<F, Fut>(&self, cleanup_fn: F)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        info!(
            "Starting graceful shutdown with {}s timeout",
            self.shutdown_timeout.as_secs()
        );

        // Execute cleanup with timeout
        let cleanup_result = tokio::time::timeout(self.shutdown_timeout, cleanup_fn()).await;

        match cleanup_result {
            Ok(_) => {
                info!("Graceful shutdown completed successfully");
            }
            Err(_) => {
                warn!(
                    "Cleanup took longer than {}s timeout, forcing shutdown",
                    self.shutdown_timeout.as_secs()
                );
            }
        }
    }

    /// Complete shutdown sequence with signal handling and cleanup
    pub async fn shutdown_sequence<F, Fut>(&self, cleanup_fn: F)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        // Wait for shutdown signal
        self.wait_for_signal().await;

        // Execute cleanup
        self.shutdown_with_cleanup(cleanup_fn).await;
    }
}

/// Resource cleanup coordinator for database connections and other resources
pub struct ResourceCleanup {
    db_pool: Option<sqlx::SqlitePool>,
}

impl ResourceCleanup {
    /// Create a new resource cleanup coordinator
    pub fn new() -> Self {
        Self { db_pool: None }
    }

    /// Register a database pool for cleanup
    pub fn with_db_pool(mut self, pool: sqlx::SqlitePool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    /// Execute all resource cleanup operations
    pub async fn cleanup(&self) {
        info!("Starting resource cleanup");

        // Clean up database connections
        if let Some(pool) = &self.db_pool {
            self.cleanup_database(pool).await;
        }

        info!("Resource cleanup completed");
    }

    /// Clean up database connections with connection draining
    async fn cleanup_database(&self, pool: &sqlx::SqlitePool) {
        info!("Draining database connections");

        // Close the connection pool gracefully
        pool.close().await;

        info!("Database connections drained successfully");
    }
}

impl Default for ResourceCleanup {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::{sleep, Instant};

    #[tokio::test]
    async fn test_graceful_shutdown_creation() {
        let shutdown = GracefulShutdown::new(Duration::from_secs(30));
        assert_eq!(shutdown.shutdown_timeout, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_resource_cleanup_creation() {
        let cleanup = ResourceCleanup::new();
        assert!(cleanup.db_pool.is_none());
    }

    #[tokio::test]
    async fn test_shutdown_with_cleanup_success() {
        let shutdown = GracefulShutdown::new(Duration::from_secs(1));
        let start = Instant::now();

        let cleanup_fn = || async {
            sleep(Duration::from_millis(100)).await;
        };

        shutdown.shutdown_with_cleanup(cleanup_fn).await;

        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_shutdown_with_cleanup_timeout() {
        let shutdown = GracefulShutdown::new(Duration::from_millis(100));
        let start = Instant::now();

        let cleanup_fn = || async {
            sleep(Duration::from_millis(200)).await;
        };

        shutdown.shutdown_with_cleanup(cleanup_fn).await;

        let elapsed = start.elapsed();
        // Should timeout after ~100ms, allowing some margin for test execution
        assert!(elapsed >= Duration::from_millis(100));
        assert!(elapsed < Duration::from_millis(150));
    }

    #[tokio::test]
    async fn test_resource_cleanup_with_db_pool() {
        // Create an in-memory SQLite database for testing
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        let cleanup = ResourceCleanup::new().with_db_pool(pool);
        assert!(cleanup.db_pool.is_some());

        // Test cleanup execution
        cleanup.cleanup().await;
    }
}
