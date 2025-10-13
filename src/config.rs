use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ObservabilityConfig {
    #[serde(default = "default_otel_endpoint")]
    pub otel_endpoint: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            otel_endpoint: default_otel_endpoint(),
            log_level: default_log_level(),
        }
    }
}

fn default_otel_endpoint() -> String {
    "http://localhost:4317".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_days: i64,
}

impl Config {
    /// Load configuration from file and environment variables
    ///
    /// Priority (highest to lowest):
    /// 1. Environment variables (IMKITCHEN__DATABASE__URL, etc.)
    /// 2. Config file specified by path
    /// 3. Hardcoded defaults
    pub fn load(config_path: Option<String>) -> Result<Self, ConfigError> {
        let mut builder = ConfigBuilder::builder();

        // Set defaults
        builder = builder
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 3000)?
            .set_default("database.url", "sqlite:imkitchen.db")?
            .set_default("database.max_connections", 5)?
            .set_default("jwt.expiration_days", 7)?;

        // Load config file if path provided or CONFIG_PATH env var set
        let config_file_path = config_path
            .or_else(|| env::var("CONFIG_PATH").ok())
            .unwrap_or_else(|| "config/default.toml".to_string());

        // Try to load config file (optional - ignore if not found)
        if std::path::Path::new(&config_file_path).exists() {
            builder = builder.add_source(File::with_name(&config_file_path));
        }

        // Override with environment variables (IMKITCHEN__DATABASE__URL, etc.)
        builder = builder.add_source(
            Environment::with_prefix("IMKITCHEN")
                .separator("__")
                .try_parsing(true),
        );

        // Also support legacy environment variables without prefix
        if let Ok(database_url) = env::var("DATABASE_URL") {
            builder = builder.set_override("database.url", database_url)?;
        }
        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            builder = builder.set_override("jwt.secret", jwt_secret)?;
        }

        builder.build()?.try_deserialize()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.jwt.secret.len() < 32 {
            return Err("JWT secret must be at least 32 characters long".to_string());
        }
        if self.database.max_connections < 1 {
            return Err("Database max_connections must be at least 1".to_string());
        }
        if self.server.port == 0 {
            return Err("Server port must be greater than 0".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_short_secret() {
        let config = Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                url: "sqlite:test.db".to_string(),
                max_connections: 5,
            },
            jwt: JwtConfig {
                secret: "short".to_string(),
                expiration_days: 7,
            },
            observability: ObservabilityConfig::default(),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_zero_port() {
        let config = Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0,
            },
            database: DatabaseConfig {
                url: "sqlite:test.db".to_string(),
                max_connections: 5,
            },
            jwt: JwtConfig {
                secret: "test_secret_key_minimum_32_characters_long".to_string(),
                expiration_days: 7,
            },
            observability: ObservabilityConfig::default(),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_zero_connections() {
        let config = Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                url: "sqlite:test.db".to_string(),
                max_connections: 0,
            },
            jwt: JwtConfig {
                secret: "test_secret_key_minimum_32_characters_long".to_string(),
                expiration_days: 7,
            },
            observability: ObservabilityConfig::default(),
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_valid_config() {
        let config = Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                url: "sqlite:test.db".to_string(),
                max_connections: 5,
            },
            jwt: JwtConfig {
                secret: "test_secret_key_minimum_32_characters_long".to_string(),
                expiration_days: 7,
            },
            observability: ObservabilityConfig::default(),
        };

        assert!(config.validate().is_ok());
    }
}
