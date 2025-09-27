use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};
use tracing_appender::rolling::Rotation;
use validator::Validate;

use crate::monitoring::LogFormat;

/// Enhanced configuration for the IMKitchen application
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
pub struct Config {
    /// Database configuration
    #[validate(nested)]
    pub database: DatabaseConfig,

    /// Server configuration
    #[validate(nested)]
    pub server: ServerConfig,

    /// Logging configuration
    #[validate(nested)]
    pub logging: LoggingConfig,

    /// Security configuration
    #[validate(nested)]
    pub security: SecurityConfig,

    /// Monitoring configuration
    #[validate(nested)]
    pub monitoring: MonitoringConfig,
}

/// Database configuration with validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DatabaseConfig {
    /// Database URL
    #[validate(length(min = 1, message = "Database URL cannot be empty"))]
    pub url: String,

    /// Maximum number of connections in the pool
    #[validate(range(
        min = 1,
        max = 100,
        message = "Max connections must be between 1 and 100"
    ))]
    pub max_connections: u32,

    /// Minimum number of connections in the pool
    #[validate(range(
        min = 0,
        max = 50,
        message = "Min connections must be between 0 and 50"
    ))]
    pub min_connections: u32,

    /// Connection timeout in seconds
    #[validate(range(
        min = 1,
        max = 300,
        message = "Connection timeout must be between 1 and 300 seconds"
    ))]
    pub connection_timeout: u64,

    /// Acquire timeout in seconds
    #[validate(range(
        min = 1,
        max = 60,
        message = "Acquire timeout must be between 1 and 60 seconds"
    ))]
    pub acquire_timeout: u64,

    /// Idle timeout in seconds
    #[validate(range(
        min = 60,
        max = 3600,
        message = "Idle timeout must be between 60 and 3600 seconds"
    ))]
    pub idle_timeout: u64,

    /// Maximum lifetime in seconds
    #[validate(range(
        min = 300,
        max = 7200,
        message = "Max lifetime must be between 300 and 7200 seconds"
    ))]
    pub max_lifetime: u64,

    /// Enable automatic migrations
    pub auto_migrate: bool,
}

/// Server configuration with validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfig {
    /// Host to bind to
    #[validate(length(min = 1, message = "Host cannot be empty"))]
    pub host: String,

    /// Port to bind to
    #[validate(range(
        min = 1024,
        max = 65535,
        message = "Port must be between 1024 and 65535"
    ))]
    pub port: u16,

    /// Request timeout in seconds
    #[validate(range(
        min = 1,
        max = 300,
        message = "Request timeout must be between 1 and 300 seconds"
    ))]
    pub request_timeout: u64,

    /// Maximum request body size in bytes
    #[validate(range(
        min = 1024,
        max = 104857600,
        message = "Max body size must be between 1KB and 100MB"
    ))]
    pub max_body_size: u64,

    /// Enable CORS
    pub enable_cors: bool,

    /// Graceful shutdown timeout in seconds
    #[validate(range(
        min = 5,
        max = 300,
        message = "Shutdown timeout must be between 5 and 300 seconds"
    ))]
    pub shutdown_timeout: u64,
}

/// Logging configuration with validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoggingConfig {
    /// Log level
    #[validate(length(min = 1, message = "Log level cannot be empty"))]
    pub level: String,

    /// Log format
    pub format: LogFormat,

    /// Log directory (optional)
    pub dir: Option<PathBuf>,

    /// Log rotation schedule
    #[serde(with = "rotation_serde")]
    pub rotation: Rotation,

    /// Enable structured logging
    pub structured: bool,

    /// Log to console
    pub console: bool,

    /// Log to file
    pub file: bool,
}

/// Security configuration with validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SecurityConfig {
    /// Session secret key (minimum 32 characters for security)
    #[validate(length(min = 32, message = "Session secret must be at least 32 characters"))]
    pub session_secret: String,

    /// Session timeout in seconds
    #[validate(range(
        min = 300,
        max = 86400,
        message = "Session timeout must be between 5 minutes and 24 hours"
    ))]
    pub session_timeout: u64,

    /// Enable HTTPS redirect
    pub force_https: bool,

    /// Trusted proxy headers
    pub trusted_proxies: Vec<String>,

    /// Rate limiting per IP (requests per minute)
    #[validate(range(
        min = 1,
        max = 10000,
        message = "Rate limit must be between 1 and 10000 requests per minute"
    ))]
    pub rate_limit_per_minute: u32,
}

/// Monitoring configuration with validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    pub enable_metrics: bool,

    /// Metrics endpoint path
    #[validate(length(min = 1, message = "Metrics endpoint cannot be empty"))]
    pub metrics_endpoint: String,

    /// Health check endpoint path
    #[validate(length(min = 1, message = "Health check endpoint cannot be empty"))]
    pub health_endpoint: String,

    /// Enable request tracing
    pub enable_tracing: bool,

    /// Metrics collection interval in seconds
    #[validate(range(
        min = 1,
        max = 300,
        message = "Metrics interval must be between 1 and 300 seconds"
    ))]
    pub metrics_interval: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:imkitchen.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            connection_timeout: 30,
            acquire_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
            auto_migrate: true,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            request_timeout: 30,
            max_body_size: 16777216, // 16MB
            enable_cors: false,
            shutdown_timeout: 30,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            dir: None,
            rotation: Rotation::DAILY,
            structured: true,
            console: true,
            file: true,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            session_secret: generate_secure_secret(),
            session_timeout: 3600, // 1 hour
            force_https: false,
            trusted_proxies: vec![],
            rate_limit_per_minute: 60,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_endpoint: "/metrics".to_string(),
            health_endpoint: "/health".to_string(),
            enable_tracing: true,
            metrics_interval: 30,
        }
    }
}

impl Config {
    /// Load configuration from file, CLI arguments, and environment variables
    /// Priority: CLI args > Environment variables > Config file > Defaults
    pub fn load_from_sources(config_path: &PathBuf, cli_args: &ConfigOverrides) -> Result<Self> {
        info!("Loading configuration from multiple sources");

        // Start with defaults
        let mut config = Config::default();

        // Load from config file if it exists
        if config_path.exists() {
            info!("Loading configuration from file: {:?}", config_path);
            config = Self::load_from_file(config_path)
                .context("Failed to load configuration from file")?;
        } else {
            info!(
                "Configuration file not found, using defaults: {:?}",
                config_path
            );
        }

        // Apply environment variable overrides
        config
            .apply_env_overrides()
            .context("Failed to apply environment variable overrides")?;

        // Apply CLI argument overrides
        config
            .apply_cli_overrides(cli_args)
            .context("Failed to apply CLI argument overrides")?;

        // Validate the final configuration
        config
            .validate()
            .context("Configuration validation failed")?;

        info!("Configuration loaded and validated successfully");
        Ok(config)
    }

    /// Load configuration from TOML file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        info!("Configuration loaded from file: {:?}", path);
        Ok(config)
    }

    /// Save configuration to TOML file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize configuration")?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;

        info!("Configuration saved to file: {:?}", path);
        Ok(())
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) -> Result<()> {
        // Database overrides
        if let Ok(url) = std::env::var("DATABASE_URL") {
            self.database.url = url;
        }
        if let Ok(max_conn) = std::env::var("DATABASE_MAX_CONNECTIONS") {
            self.database.max_connections = max_conn
                .parse()
                .context("Invalid DATABASE_MAX_CONNECTIONS value")?;
        }

        // Server overrides
        if let Ok(host) = std::env::var("SERVER_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = std::env::var("SERVER_PORT") {
            self.server.port = port.parse().context("Invalid SERVER_PORT value")?;
        }

        // Security overrides
        if let Ok(secret) = std::env::var("SESSION_SECRET") {
            self.security.session_secret = secret;
        }

        // Logging overrides
        if let Ok(level) = std::env::var("RUST_LOG") {
            self.logging.level = level;
        }
        if let Ok(format) = std::env::var("LOG_FORMAT") {
            self.logging.format = match format.as_str() {
                "json" => LogFormat::Json,
                "compact" => LogFormat::Compact,
                _ => LogFormat::Pretty,
            };
        }
        if let Ok(dir) = std::env::var("LOG_DIR") {
            self.logging.dir = Some(PathBuf::from(dir));
        }
        if let Ok(rotation) = std::env::var("LOG_ROTATION") {
            self.logging.rotation = match rotation.as_str() {
                "hourly" => Rotation::HOURLY,
                "daily" => Rotation::DAILY,
                "never" => Rotation::NEVER,
                _ => Rotation::DAILY,
            };
        }

        Ok(())
    }

    /// Apply CLI argument overrides
    fn apply_cli_overrides(&mut self, overrides: &ConfigOverrides) -> Result<()> {
        if let Some(ref database_url) = overrides.database_url {
            self.database.url = database_url.clone();
        }

        if let Some(ref log_level) = overrides.log_level {
            self.logging.level = log_level.clone();
        }

        if let Some(host) = &overrides.host {
            self.server.host = host.clone();
        }

        if let Some(port) = overrides.port {
            self.server.port = port;
        }

        Ok(())
    }

    /// Generate a sample configuration file
    pub fn generate_sample_config(path: &PathBuf) -> Result<()> {
        let config = Config::default();
        config.save_to_file(path)?;
        info!("Sample configuration generated at: {:?}", path);
        Ok(())
    }

    /// Validate sensitive configuration values
    pub fn validate_security(&self) -> Result<()> {
        // Check session secret strength
        if self.security.session_secret.len() < 32 {
            return Err(anyhow::anyhow!(
                "Session secret must be at least 32 characters for security"
            ));
        }

        // Warn about default values in production
        if self.security.session_secret == generate_secure_secret() {
            warn!("Using default session secret - please set SESSION_SECRET environment variable in production");
        }

        if !self.security.force_https
            && !self.server.host.starts_with("127.")
            && self.server.host != "localhost"
        {
            warn!("HTTPS not enforced for non-localhost binding - consider enabling force_https in production");
        }

        Ok(())
    }
}

/// CLI argument overrides for configuration
#[derive(Debug, Default)]
pub struct ConfigOverrides {
    pub database_url: Option<String>,
    pub log_level: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
}

/// Generate a secure random secret for sessions
fn generate_secure_secret() -> String {
    use uuid::Uuid;
    // Generate two UUIDs and concatenate for 64-character string
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

/// Custom serialization for Rotation enum
mod rotation_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use tracing_appender::rolling::Rotation;

    pub fn serialize<S>(rotation: &Rotation, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match *rotation {
            Rotation::MINUTELY => "minutely",
            Rotation::HOURLY => "hourly",
            Rotation::DAILY => "daily",
            Rotation::NEVER => "never",
        };
        s.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Rotation, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "minutely" => Ok(Rotation::MINUTELY),
            "hourly" => Ok(Rotation::HOURLY),
            "daily" => Ok(Rotation::DAILY),
            "never" => Ok(Rotation::NEVER),
            _ => Ok(Rotation::DAILY),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(config.database.url, deserialized.database.url);
        assert_eq!(config.server.port, deserialized.server.port);
    }

    #[test]
    fn test_config_file_operations() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let config = Config::default();
        config.save_to_file(&config_path).unwrap();

        let loaded_config = Config::load_from_file(&config_path).unwrap();
        assert_eq!(config.database.url, loaded_config.database.url);
    }

    #[test]
    fn test_validation_errors() {
        let mut config = Config::default();

        // Test invalid port
        config.server.port = 80; // Too low
        assert!(config.validate().is_err());

        // Test invalid database URL
        config.server.port = 3000; // Fix port
        config.database.url = "".to_string(); // Empty URL
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_env_overrides() {
        std::env::set_var("DATABASE_URL", "sqlite:test.db");
        std::env::set_var("SERVER_PORT", "8080");

        let mut config = Config::default();
        config.apply_env_overrides().unwrap();

        assert_eq!(config.database.url, "sqlite:test.db");
        assert_eq!(config.server.port, 8080);

        // Cleanup
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_cli_overrides() {
        let overrides = ConfigOverrides {
            database_url: Some("sqlite:cli.db".to_string()),
            port: Some(9000),
            ..Default::default()
        };

        let mut config = Config::default();
        config.apply_cli_overrides(&overrides).unwrap();

        assert_eq!(config.database.url, "sqlite:cli.db");
        assert_eq!(config.server.port, 9000);
    }

    #[test]
    fn test_secure_secret_generation() {
        let secret = generate_secure_secret();
        assert!(secret.len() >= 32);

        // Ensure different calls generate different secrets
        let secret2 = generate_secure_secret();
        assert_ne!(secret, secret2);
    }
}
