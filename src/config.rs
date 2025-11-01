//! Application configuration

use config::{Config as ConfigLoader, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

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
    /// Load configuration from files and environment
    ///
    /// Configuration is loaded in this order (later sources override earlier ones):
    /// 1. config/default.toml (required)
    /// 2. config/dev.toml (optional, for local development)
    /// 3. Environment variables (optional, prefix: IMKITCHEN_)
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let config = ConfigLoader::builder()
            // Start with default config
            .add_source(File::with_name("config/default"))
            // Add optional environment-specific config
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add dev.toml if it exists (for local development overrides)
            .add_source(File::with_name("config/dev").required(false))
            // Add environment variables (with prefix IMKITCHEN_)
            // Example: IMKITCHEN_SERVER__PORT=8080
            .add_source(Environment::with_prefix("IMKITCHEN").separator("__"))
            .build()?;

        config.try_deserialize()
    }
}
