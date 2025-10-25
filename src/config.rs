use base64::{engine::general_purpose::STANDARD, Engine as _};
use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    #[serde(default)]
    pub email: EmailConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub stripe: StripeConfig,
    #[serde(default)]
    pub vapid: VapidConfig,
    #[serde(default)]
    pub features: FeatureConfig,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FeatureConfig {
    /// Bypass premium restrictions for MVP/demo mode
    /// When true, all users are treated as premium (no recipe limit, full features)
    #[serde(default)]
    pub bypass_premium: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    #[serde(default = "default_smtp_host")]
    pub smtp_host: String,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    #[serde(default = "default_smtp_username")]
    pub smtp_username: String,
    #[serde(default)]
    pub smtp_password: String,
    #[serde(default = "default_from_email")]
    pub from_email: String,
    #[serde(default = "default_from_name")]
    pub from_name: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_smtp_tls")]
    pub smtp_tls: bool,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: default_smtp_host(),
            smtp_port: default_smtp_port(),
            smtp_username: default_smtp_username(),
            smtp_password: String::new(),
            from_email: default_from_email(),
            from_name: default_from_name(),
            base_url: default_base_url(),
            smtp_tls: default_smtp_tls(),
        }
    }
}

fn default_smtp_host() -> String {
    "localhost".to_string()
}

fn default_smtp_port() -> u16 {
    587
}

fn default_smtp_username() -> String {
    "noreply@imkitchen.app".to_string()
}

fn default_from_email() -> String {
    "noreply@imkitchen.app".to_string()
}

fn default_from_name() -> String {
    "imkitchen".to_string()
}

fn default_base_url() -> String {
    "http://localhost:3000".to_string()
}

fn default_smtp_tls() -> bool {
    true
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

#[derive(Debug, Deserialize, Clone, Default)]
pub struct StripeConfig {
    #[serde(default)]
    pub secret_key: String,
    #[serde(default)]
    pub webhook_secret: String,
    #[serde(default)]
    pub price_id: String, // Stripe Price ID for $9.99/month subscription
}

#[derive(Debug, Deserialize, Clone)]
pub struct VapidConfig {
    #[serde(default)]
    pub public_key: String,
    #[serde(default)]
    pub private_key: String,
    #[serde(default = "default_vapid_subject")]
    pub subject: String,
}

impl Default for VapidConfig {
    fn default() -> Self {
        Self {
            public_key: String::new(),
            private_key: String::new(),
            subject: default_vapid_subject(),
        }
    }
}

impl VapidConfig {
    /// Convert base64url-encoded private key to PEM format
    /// The web-push crate requires PEM format for VAPID signatures
    pub fn private_key_as_pem(&self) -> Result<String, String> {
        if self.private_key.is_empty() {
            return Err("VAPID private key is empty".to_string());
        }

        // Check if already in PEM format
        if self.private_key.starts_with("-----BEGIN") {
            return Ok(self.private_key.clone());
        }

        // Convert base64url to base64 standard and add padding
        let mut base64_standard = self.private_key.replace('-', "+").replace('_', "/");

        // Add padding if needed (base64 strings must be multiple of 4 chars)
        match base64_standard.len() % 4 {
            2 => base64_standard.push_str("=="),
            3 => base64_standard.push('='),
            _ => {} // 0 or 1 (1 would be invalid, but we'll let decode catch it)
        }

        // Decode base64 to raw bytes
        let raw_key = STANDARD
            .decode(&base64_standard)
            .map_err(|e| format!("Failed to decode VAPID private key: {}", e))?;

        // Verify key length (should be 32 bytes for P-256)
        if raw_key.len() != 32 {
            return Err(format!(
                "Invalid VAPID private key length: expected 32 bytes, got {}",
                raw_key.len()
            ));
        }

        // Build PEM-encoded EC private key (PKCS#8 format)
        // EC private key structure for P-256:
        // SEQUENCE {
        //   INTEGER 1 (version)
        //   OCTET STRING (32-byte private key)
        //   [0] OBJECT IDENTIFIER prime256v1 (1.2.840.10045.3.1.7)
        //   [1] BIT STRING (65-byte public key, uncompressed)
        // }

        // For simplicity, we'll construct the DER encoding manually
        let mut der = Vec::new();

        // SEQUENCE tag + length (will be calculated)
        der.push(0x30); // SEQUENCE

        // Version: INTEGER 1
        der.extend_from_slice(&[0x02, 0x01, 0x01]); // INTEGER 1

        // Private key: OCTET STRING (32 bytes)
        der.push(0x04); // OCTET STRING
        der.push(32); // length
        der.extend_from_slice(&raw_key);

        // Curve OID: [0] EXPLICIT (prime256v1)
        der.extend_from_slice(&[
            0xa0, 0x0a, // [0] EXPLICIT, length 10
            0x06, 0x08, // OBJECT IDENTIFIER, length 8
            0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, // prime256v1 OID
        ]);

        // Calculate and insert total length at position 1
        let total_length = der.len() - 1;
        der.insert(1, total_length as u8);

        // Encode DER as base64
        let base64_der = STANDARD.encode(&der);

        // Format as PEM
        let pem = format!(
            "-----BEGIN EC PRIVATE KEY-----\n{}\n-----END EC PRIVATE KEY-----",
            base64_der
                .as_bytes()
                .chunks(64)
                .map(|chunk| std::str::from_utf8(chunk).unwrap())
                .collect::<Vec<&str>>()
                .join("\n")
        );

        Ok(pem)
    }
}

fn default_vapid_subject() -> String {
    "mailto:contact@imkitchen.local".to_string()
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
        if let Ok(stripe_secret) = env::var("STRIPE_SECRET_KEY") {
            builder = builder.set_override("stripe.secret_key", stripe_secret)?;
        }
        if let Ok(stripe_webhook_secret) = env::var("STRIPE_WEBHOOK_SECRET") {
            builder = builder.set_override("stripe.webhook_secret", stripe_webhook_secret)?;
        }
        if let Ok(stripe_price_id) = env::var("STRIPE_PRICE_ID") {
            builder = builder.set_override("stripe.price_id", stripe_price_id)?;
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
            email: EmailConfig::default(),
            observability: ObservabilityConfig::default(),
            stripe: StripeConfig::default(),
            vapid: VapidConfig::default(),
            features: FeatureConfig::default(),
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
            email: EmailConfig::default(),
            observability: ObservabilityConfig::default(),
            stripe: StripeConfig::default(),
            vapid: VapidConfig::default(),
            features: FeatureConfig::default(),
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
            email: EmailConfig::default(),
            observability: ObservabilityConfig::default(),
            stripe: StripeConfig::default(),
            vapid: VapidConfig::default(),
            features: FeatureConfig::default(),
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
            email: EmailConfig::default(),
            observability: ObservabilityConfig::default(),
            stripe: StripeConfig::default(),
            vapid: VapidConfig::default(),
            features: FeatureConfig::default(),
        };

        assert!(config.validate().is_ok());
    }
}
