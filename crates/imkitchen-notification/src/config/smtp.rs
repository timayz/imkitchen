use serde::{Deserialize, Serialize};
use std::env;
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SmtpSecurity {
    None,
    StartTls,
    Ssl,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SmtpConfig {
    #[validate(length(min = 1, message = "SMTP host is required"))]
    pub host: String,

    #[validate(range(
        min = 1,
        max = 65535,
        message = "SMTP port must be between 1 and 65535"
    ))]
    pub port: u16,

    pub username: String,

    pub password: String,

    #[validate(email(message = "From email must be a valid email address"))]
    pub from_email: String,

    #[validate(length(
        min = 1,
        max = 100,
        message = "From name must be between 1 and 100 characters"
    ))]
    pub from_name: String,

    pub security: SmtpSecurity,

    #[validate(range(
        min = 5,
        max = 300,
        message = "Timeout must be between 5 and 300 seconds"
    ))]
    pub timeout_seconds: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum SmtpConfigError {
    #[error("Environment variable missing: {0}")]
    MissingEnvVar(String),

    #[error("Environment variable invalid: {0} - {1}")]
    InvalidEnvVar(String, String),

    #[error("Validation failed: {0}")]
    ValidationFailed(#[from] validator::ValidationErrors),
}

impl SmtpConfig {
    /// Create SMTP configuration from environment variables
    pub fn from_env() -> Result<Self, SmtpConfigError> {
        let host = env::var("SMTP_HOST")
            .map_err(|_| SmtpConfigError::MissingEnvVar("SMTP_HOST".to_string()))?;

        let port_str = env::var("SMTP_PORT")
            .map_err(|_| SmtpConfigError::MissingEnvVar("SMTP_PORT".to_string()))?;
        let port = port_str
            .parse::<u16>()
            .map_err(|e| SmtpConfigError::InvalidEnvVar("SMTP_PORT".to_string(), e.to_string()))?;

        let username = env::var("SMTP_USERNAME")
            .map_err(|_| SmtpConfigError::MissingEnvVar("SMTP_USERNAME".to_string()))?;

        let password = env::var("SMTP_PASSWORD")
            .map_err(|_| SmtpConfigError::MissingEnvVar("SMTP_PASSWORD".to_string()))?;

        let from_email = env::var("SMTP_FROM_EMAIL")
            .map_err(|_| SmtpConfigError::MissingEnvVar("SMTP_FROM_EMAIL".to_string()))?;

        let from_name = env::var("SMTP_FROM_NAME")
            .map_err(|_| SmtpConfigError::MissingEnvVar("SMTP_FROM_NAME".to_string()))?;

        // Optional security setting, defaults to StartTls
        let security = match env::var("SMTP_SECURITY").as_deref() {
            Ok("ssl") => SmtpSecurity::Ssl,
            Ok("none") => SmtpSecurity::None,
            _ => SmtpSecurity::StartTls, // Default
        };

        // Optional timeout setting, defaults to 30 seconds
        let timeout_seconds = env::var("SMTP_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);

        let config = SmtpConfig {
            host,
            port,
            username,
            password,
            from_email,
            from_name,
            security,
            timeout_seconds,
        };

        config.validate()?;
        Ok(config)
    }

    /// Create development fallback configuration for local testing
    pub fn development_fallback() -> Self {
        SmtpConfig {
            host: "localhost".to_string(),
            port: 1025, // Mailhog default port
            username: "".to_string(),
            password: "".to_string(),
            from_email: "dev@imkitchen.local".to_string(),
            from_name: "IMKitchen Dev".to_string(),
            security: SmtpSecurity::None,
            timeout_seconds: 30,
        }
    }

    /// Check if this configuration requires encryption
    pub fn requires_encryption(&self) -> bool {
        matches!(self.security, SmtpSecurity::StartTls | SmtpSecurity::Ssl)
    }

    /// Get the default port for the security type
    pub fn default_port_for_security(security: &SmtpSecurity) -> u16 {
        match security {
            SmtpSecurity::None => 25,
            SmtpSecurity::StartTls => 587,
            SmtpSecurity::Ssl => 465,
        }
    }

    /// Validate configuration and perform additional business logic checks
    pub fn validate(&self) -> Result<(), SmtpConfigError> {
        // Run validator derive validation
        Validate::validate(self)?;

        // Additional business logic validation
        if self.security != SmtpSecurity::None {
            // For secured connections, username must be a valid email and password required
            if self.username.is_empty() || self.password.is_empty() {
                let mut errors = validator::ValidationErrors::new();
                errors.add(
                    "authentication",
                    ValidationError::new("authentication_required"),
                );
                return Err(SmtpConfigError::ValidationFailed(errors));
            }

            // Validate username is email format for authenticated connections
            if !self.username.contains('@') {
                let mut errors = validator::ValidationErrors::new();
                errors.add("username", ValidationError::new("email_format_required"));
                return Err(SmtpConfigError::ValidationFailed(errors));
            }
        }

        Ok(())
    }
}
