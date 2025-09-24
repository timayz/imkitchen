use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database_status: DatabaseStatus,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseStatus {
    Connected,
    Disconnected,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Server error: {0}")]
    Server(String),
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:imkitchen.db".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_database_config_defaults() {
        let config = DatabaseConfig::default();
        assert_eq!(config.url, "sqlite:imkitchen.db");
    }

    #[test]
    fn test_logging_config_defaults() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, "json");
    }

    #[test]
    fn test_app_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.database.url, "sqlite:imkitchen.db");
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.logging.format, "json");
    }

    #[test]
    fn test_health_response_serialization() {
        use serde_json;

        let health = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            database_status: DatabaseStatus::Connected,
            uptime_seconds: 42,
        };

        let json = serde_json::to_string(&health).unwrap();
        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"version\":\"0.1.0\""));
        assert!(json.contains("\"database_status\":\"Connected\""));
        assert!(json.contains("\"uptime_seconds\":42"));
    }

    #[test]
    fn test_database_status_variants() {
        assert!(matches!(
            DatabaseStatus::Connected,
            DatabaseStatus::Connected
        ));
        assert!(matches!(
            DatabaseStatus::Disconnected,
            DatabaseStatus::Disconnected
        ));
        assert!(matches!(
            DatabaseStatus::Error("test".to_string()),
            DatabaseStatus::Error(_)
        ));
    }

    #[test]
    fn test_app_error_display() {
        let db_error = AppError::Database("connection failed".to_string());
        assert_eq!(format!("{}", db_error), "Database error: connection failed");

        let config_error = AppError::Config("invalid config".to_string());
        assert_eq!(
            format!("{}", config_error),
            "Configuration error: invalid config"
        );

        let server_error = AppError::Server("bind failed".to_string());
        assert_eq!(format!("{}", server_error), "Server error: bind failed");
    }
}
