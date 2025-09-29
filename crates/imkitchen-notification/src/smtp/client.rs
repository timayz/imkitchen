use crate::config::{SmtpConfig, SmtpSecurity};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::SmtpTransport;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SmtpConnectionError {
    #[error("SMTP connection failed: {0}")]
    ConnectionFailed(String),

    #[error("SMTP authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Connection timeout")]
    Timeout,

    #[error("TLS configuration error: {0}")]
    TlsError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// SMTP client with connection management and authentication
pub struct SmtpClient {
    config: SmtpConfig,
    transport: SmtpTransport,
}

impl SmtpClient {
    /// Create a new SMTP client with the given configuration
    pub fn new(config: SmtpConfig) -> Result<Self, SmtpConnectionError> {
        let transport = Self::build_transport(&config)?;

        Ok(SmtpClient { config, transport })
    }

    /// Build the SMTP transport based on configuration
    fn build_transport(config: &SmtpConfig) -> Result<SmtpTransport, SmtpConnectionError> {
        let mut builder = SmtpTransport::builder_dangerous(&config.host)
            .port(config.port)
            .timeout(Some(Duration::from_secs(config.timeout_seconds as u64)));

        // Configure TLS/Security
        builder = match config.security {
            SmtpSecurity::None => builder,
            SmtpSecurity::StartTls => {
                let tls = TlsParameters::new(config.host.clone())
                    .map_err(|e| SmtpConnectionError::TlsError(e.to_string()))?;
                builder.tls(Tls::Required(tls))
            }
            SmtpSecurity::Ssl => {
                let tls = TlsParameters::new(config.host.clone())
                    .map_err(|e| SmtpConnectionError::TlsError(e.to_string()))?;
                builder.tls(Tls::Wrapper(tls))
            }
        };

        // Configure authentication
        if !config.username.is_empty() && !config.password.is_empty() {
            let credentials = Credentials::new(config.username.clone(), config.password.clone());
            builder = builder.credentials(credentials);
        }

        Ok(builder.build())
    }

    /// Get the configuration used by this client
    pub fn config(&self) -> &SmtpConfig {
        &self.config
    }

    /// Get the connection timeout duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.config.timeout_seconds as u64)
    }

    /// Test the SMTP connection without sending an email
    pub async fn test_connection(&self) -> Result<(), SmtpConnectionError> {
        // Create a test transport to verify connection
        match self.transport.test_connection() {
            Ok(true) => Ok(()),
            Ok(false) => Err(SmtpConnectionError::ConnectionFailed(
                "Connection test returned false".to_string(),
            )),
            Err(e) => Err(SmtpConnectionError::ConnectionFailed(e.to_string())),
        }
    }

    /// Get the underlying transport (for sending emails)
    pub fn transport(&self) -> &SmtpTransport {
        &self.transport
    }
}
