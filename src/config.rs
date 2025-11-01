//! Application configuration

use config::{Config as ConfigLoader, ConfigError, Environment, File};
use serde::Deserialize;

/// Application configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

/// Server configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Database configuration
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub evento_db: String,
    pub queries_db: String,
    pub validation_db: String,
}

/// Logging configuration
#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                evento_db: "evento.db".to_string(),
                queries_db: "queries.db".to_string(),
                validation_db: "validation.db".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
            },
        }
    }
}

impl Config {
    /// Load configuration from optional file and environment variables
    ///
    /// Configuration is loaded in this order (later sources override earlier ones):
    /// 1. Default values (hardcoded)
    /// 2. Optional config file (if path provided via IMKITCHEN_CONFIG env var)
    /// 3. Environment variables (prefix: IMKITCHEN_)
    ///
    /// Example environment variable: IMKITCHEN_SERVER__PORT=8080
    pub fn load() -> Result<Self, ConfigError> {
        let mut builder = ConfigLoader::builder();

        // Add optional config file if IMKITCHEN_CONFIG is set
        if let Ok(config_path) = std::env::var("IMKITCHEN_CONFIG") {
            builder = builder.add_source(File::with_name(&config_path));
        }

        // Add environment variables (with prefix IMKITCHEN_)
        // Example: IMKITCHEN_SERVER__PORT=8080
        let config = builder
            .add_source(Environment::with_prefix("IMKITCHEN").separator("__"))
            .build()?;

        // Deserialize with defaults
        let mut settings = Self::default();

        // Override with config values if present
        if let Ok(server_host) = config.get_string("server.host") {
            settings.server.host = server_host;
        }
        if let Ok(server_port) = config.get_int("server.port") {
            settings.server.port = server_port as u16;
        }
        if let Ok(evento_db) = config.get_string("database.evento_db") {
            settings.database.evento_db = evento_db;
        }
        if let Ok(queries_db) = config.get_string("database.queries_db") {
            settings.database.queries_db = queries_db;
        }
        if let Ok(validation_db) = config.get_string("database.validation_db") {
            settings.database.validation_db = validation_db;
        }
        if let Ok(logging_level) = config.get_string("logging.level") {
            settings.logging.level = logging_level;
        }
        if let Ok(logging_format) = config.get_string("logging.format") {
            settings.logging.format = logging_format;
        }

        Ok(settings)
    }
}
