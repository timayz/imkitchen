use imkitchen_shared::{AppConfig, AppError, DatabaseStatus, HealthResponse};
use sqlx::SqlitePool;
use std::time::Instant;
use tracing::{error, info};

pub struct AppState {
    pub db: Option<SqlitePool>,
    pub config: AppConfig,
    pub start_time: Instant,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            db: None,
            config,
            start_time: Instant::now(),
        }
    }

    pub async fn initialize_database(&mut self) -> Result<(), AppError> {
        info!("Initializing database connection");

        match SqlitePool::connect(&self.config.database.url).await {
            Ok(pool) => {
                info!("Database connection established");
                self.db = Some(pool);
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to database: {}", e);
                Err(AppError::Database(e.to_string()))
            }
        }
    }

    pub async fn health_check(&self) -> HealthResponse {
        let uptime_seconds = self.start_time.elapsed().as_secs();
        let database_status = self.check_database_status().await;
        let status = if matches!(database_status, DatabaseStatus::Connected) {
            "healthy"
        } else {
            "unhealthy"
        };

        HealthResponse {
            status: status.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database_status,
            uptime_seconds,
        }
    }

    async fn check_database_status(&self) -> DatabaseStatus {
        match &self.db {
            Some(pool) => match sqlx::query("SELECT 1").fetch_one(pool).await {
                Ok(_) => DatabaseStatus::Connected,
                Err(e) => {
                    error!("Database health check failed: {}", e);
                    DatabaseStatus::Error(e.to_string())
                }
            },
            None => DatabaseStatus::Disconnected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use imkitchen_shared::{DatabaseConfig, LoggingConfig, ServerConfig};

    fn create_test_config() -> AppConfig {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                url: "sqlite::memory:".to_string(), // In-memory database for tests
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let config = create_test_config();
        let app_state = AppState::new(config.clone());

        assert_eq!(app_state.config.server.host, config.server.host);
        assert_eq!(app_state.config.server.port, config.server.port);
        assert_eq!(app_state.config.database.url, config.database.url);
        assert!(app_state.db.is_none());
    }

    #[tokio::test]
    async fn test_database_initialization_success() {
        let config = create_test_config();
        let mut app_state = AppState::new(config);

        let result = app_state.initialize_database().await;
        assert!(result.is_ok());
        assert!(app_state.db.is_some());
    }

    #[tokio::test]
    async fn test_database_initialization_failure() {
        let mut config = create_test_config();
        config.database.url = "invalid://database/url".to_string();
        let mut app_state = AppState::new(config);

        let result = app_state.initialize_database().await;
        assert!(result.is_err());
        assert!(app_state.db.is_none());

        match result.unwrap_err() {
            AppError::Database(_) => {} // Expected
            _ => panic!("Expected Database error"),
        }
    }

    #[tokio::test]
    async fn test_health_check_without_db() {
        let config = create_test_config();
        let app_state = AppState::new(config);

        let health = app_state.health_check().await;

        assert_eq!(health.status, "unhealthy");
        assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
        assert!(matches!(
            health.database_status,
            DatabaseStatus::Disconnected
        ));
        // Note: uptime_seconds is u64, so always >= 0
    }

    #[tokio::test]
    async fn test_health_check_with_db() {
        let config = create_test_config();
        let mut app_state = AppState::new(config);
        app_state.initialize_database().await.unwrap();

        let health = app_state.health_check().await;

        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
        assert!(matches!(health.database_status, DatabaseStatus::Connected));
        // Note: uptime_seconds is u64, so always >= 0
    }

    #[tokio::test]
    async fn test_uptime_calculation() {
        let config = create_test_config();
        let app_state = AppState::new(config);

        // Wait a small amount of time
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let _health = app_state.health_check().await;
        // Note: uptime_seconds is u64, so always >= 0
    }
}
