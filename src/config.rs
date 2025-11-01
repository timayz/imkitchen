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

impl Config {
    /// Load configuration from optional file and environment variables
    ///
    /// Configuration is loaded in this order (later sources override earlier ones):
    /// 1. Default values (hardcoded)
    /// 2. Custom config file (if path provided via --config)
    /// 3. Environment variables (prefix: IMKITCHEN_)
    ///
    /// Example environment variable: IMKITCHEN_SERVER__PORT=8080
    pub fn load(config_path: Option<&str>) -> Result<Self, ConfigError> {
        let mut builder =
            ConfigLoader::builder().add_source(File::with_name("config/default").required(false));

        // Add custom config file if provided
        if let Some(path) = config_path {
            builder = builder.add_source(File::with_name(path));
        }

        // Add environment variables (with prefix IMKITCHEN_)
        // Example: IMKITCHEN_SERVER__PORT=8080
        let config = builder
            .add_source(Environment::with_prefix("IMKITCHEN").separator("__"))
            .build()?;

        config.try_deserialize()
    }
}
