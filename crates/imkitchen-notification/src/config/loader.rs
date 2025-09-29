use super::{SmtpConfig, SmtpConfigError};
use std::env;
use tracing::{info, warn};

/// Configuration loader with fallback support for development
pub struct SmtpConfigLoader;

impl SmtpConfigLoader {
    /// Load SMTP configuration with fallback to development mode
    pub fn load_with_fallback() -> SmtpConfig {
        match Self::load_from_env() {
            Ok(config) => {
                info!("SMTP configuration loaded from environment variables");
                config
            }
            Err(e) => {
                warn!("Failed to load SMTP config from environment: {}, falling back to development mode", e);
                Self::development_fallback()
            }
        }
    }

    /// Load configuration from environment variables (strict)
    pub fn load_from_env() -> Result<SmtpConfig, SmtpConfigError> {
        SmtpConfig::from_env()
    }

    /// Create development fallback configuration
    pub fn development_fallback() -> SmtpConfig {
        let config = SmtpConfig::development_fallback();
        info!("Using development SMTP fallback configuration (localhost:1025)");
        config
    }

    /// Validate that the current environment has required SMTP configuration
    pub fn validate_environment() -> Result<(), SmtpConfigError> {
        Self::load_from_env().map(|_| ())
    }

    /// Check if development mode should be used based on environment
    pub fn is_development_mode() -> bool {
        env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            == "development"
    }

    /// Get configuration for specific SMTP providers
    pub fn gmail_config(username: String, password: String, from_name: String) -> SmtpConfig {
        SmtpConfig {
            host: "smtp.gmail.com".to_string(),
            port: 587,
            username: username.clone(),
            password,
            from_email: username,
            from_name,
            security: super::SmtpSecurity::StartTls,
            timeout_seconds: 30,
        }
    }

    pub fn sendgrid_config(api_key: String, from_email: String, from_name: String) -> SmtpConfig {
        SmtpConfig {
            host: "smtp.sendgrid.net".to_string(),
            port: 587,
            username: "apikey".to_string(),
            password: api_key,
            from_email,
            from_name,
            security: super::SmtpSecurity::StartTls,
            timeout_seconds: 30,
        }
    }
}
