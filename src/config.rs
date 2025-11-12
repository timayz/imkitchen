use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub root: RootConfig,
    // pub email: EmailConfig,
    // pub stripe: StripeConfig,
    pub features: FeaturesConfig,
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
    pub premium: bool,
}

// #[derive(Debug, Deserialize, Clone)]
// pub struct EmailConfig {
//     pub smtp_host: String,
//     pub smtp_port: u16,
//     pub smtp_username: String,
//     pub smtp_password: String,
//     pub from_address: String,
// }

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    // pub url: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RootConfig {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub audience: String,
    pub issuer: String,
    pub secret: String,
    pub expiration_days: u64,
}

// #[derive(Debug, Deserialize, Clone)]
// pub struct StripeConfig {
//     pub secret_key: String,
//     pub webhook_secret: String,
//     pub price_id: String, // Stripe Price ID for $9.99/month subscription
// }

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
            .set_default("server.url", "https://inkitchen.localhost")?
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("root.email", "root@imkitchen.localhost")?
            .set_default("root.password", "imkitchen")?
            .set_default("database.url", "sqlite:imkitchen.db")?
            .set_default("database.max_connections", 5)?
            .set_default("jwt.audience", "https://imkitchen.localhost")?
            .set_default("jwt.issuer", "imkitchen.localhost")?
            .set_default("jwt.secret", "TOKEN-NOT-SECURE-MUST-BE-CHANGE")?
            .set_default("jwt.expiration_days", 7)?
            .set_default("features.premium", true)?
            .set_default("monitoring.log_level", "debug,sqlx=info")?
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
            .set_default("email.from_address", "no-reply@imkitchen.localhost")?
            .add_source(File::with_name(&config_path).required(false))
            .add_source(Environment::with_prefix("IMKITCHEN"))
            .build()?
            .try_deserialize()
    }
}
