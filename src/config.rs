use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub _jwt: JwtConfig,
    pub _email: EmailConfig,
    pub _stripe: StripeConfig,
    pub _features: FeaturesConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MonitoringConfig {
    pub log_level: String,
    pub log_json: bool,
    pub log_target: bool,
    pub log_line_number: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FeaturesConfig {
    pub _premium: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    pub _smtp_host: String,
    pub _smtp_port: u16,
    pub _smtp_username: String,
    pub _smtp_password: String,
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
    pub _secret: String,
    pub _expiration_days: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StripeConfig {
    pub _secret_key: String,
    pub _webhook_secret: String,
    pub _price_id: String, // Stripe Price ID for $9.99/month subscription
}

impl Config {
    /// Load configuration from file and environment variables
    ///
    /// Priority (highest to lowest):
    /// 1. Environment variables (IMKITCHEN__DATABASE__URL, etc.)
    /// 2. Config file specified by path
    /// 3. Hardcoded defaults
    pub fn load(config_path: Option<String>) -> Result<Self, ConfigError> {
        let config_path = config_path.unwrap_or_else(|| "imkitchen.toml".to_string());
        ConfigBuilder::builder()
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 3000)?
            .set_default("database.url", "sqlite:imkitchen.db")?
            .set_default("database.max_connections", 5)?
            .set_default("jwt.secret", "TOKEN-NOT-SECURE-MUST-BE-CHANGE")?
            .set_default("jwt.expiration_days", 7)?
            .set_default("features.premium", true)?
            .set_default("monitoring.log_level", "debug")?
            .set_default("monitoring.log_json", false)?
            .set_default("monitoring.log_target", true)?
            .set_default("monitoring.log_line_number", true)?
            .set_default("stripe.secret_key", "")?
            .set_default("stripe.webhook_secret", "")?
            .set_default("stripe.price_id", "")?
            .set_default("email.smtp_host", "localhost")?
            .set_default("email.smtp_port", "1025")?
            .set_default("email.smtp_username", "")?
            .set_default("email.smtp_password", "")?
            .add_source(File::with_name(&config_path).required(false))
            .add_source(Environment::with_prefix("IMKITCHEN"))
            .build()?
            .try_deserialize()
    }
}
