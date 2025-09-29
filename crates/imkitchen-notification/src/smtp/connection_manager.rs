use super::client::{SmtpClient, SmtpConnectionError};
use crate::config::SmtpConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
        }
    }
}

#[derive(Debug)]
pub struct ConnectionHealth {
    pub is_healthy: bool,
    pub last_check: Instant,
    pub error_count: u32,
    pub last_error: Option<String>,
}

/// SMTP connection manager with pooling and health checks
pub struct SmtpConnectionManager {
    config: SmtpConfig,
    max_connections: usize,
    retry_config: RetryConfig,
    health: Arc<RwLock<ConnectionHealth>>,
    clients: Arc<RwLock<Vec<Arc<SmtpClient>>>>,
}

impl SmtpConnectionManager {
    /// Create a new connection manager
    pub async fn new(
        config: SmtpConfig,
        max_connections: usize,
    ) -> Result<Self, SmtpConnectionError> {
        let manager = SmtpConnectionManager {
            config,
            max_connections,
            retry_config: RetryConfig::default(),
            health: Arc::new(RwLock::new(ConnectionHealth {
                is_healthy: true,
                last_check: Instant::now(),
                error_count: 0,
                last_error: None,
            })),
            clients: Arc::new(RwLock::new(Vec::new())),
        };

        info!(
            "SMTP connection manager created with max {} connections",
            max_connections
        );
        Ok(manager)
    }

    /// Get the maximum number of connections
    pub fn max_connections(&self) -> usize {
        self.max_connections
    }

    /// Get the retry configuration
    pub fn retry_config(&self) -> &RetryConfig {
        &self.retry_config
    }

    /// Check if the connection manager is healthy
    pub async fn is_healthy(&self) -> bool {
        let health = self.health.read().await;
        health.is_healthy
    }

    /// Perform a health check on the SMTP connections
    pub async fn health_check(&self) -> Result<(), SmtpConnectionError> {
        debug!("Performing SMTP connection health check");

        let mut health = self.health.write().await;
        health.last_check = Instant::now();

        // For now, just check that we can create a client
        match SmtpClient::new(self.config.clone()) {
            Ok(_) => {
                health.is_healthy = true;
                health.error_count = 0;
                health.last_error = None;
                debug!("SMTP health check passed");
                Ok(())
            }
            Err(e) => {
                health.is_healthy = false;
                health.error_count += 1;
                health.last_error = Some(e.to_string());
                warn!("SMTP health check failed: {}", e);
                Err(e)
            }
        }
    }

    /// Get or create an SMTP client
    pub async fn get_client(&self) -> Result<Arc<SmtpClient>, SmtpConnectionError> {
        let mut clients = self.clients.write().await;

        // If we have available clients, return one
        if let Some(client) = clients.pop() {
            debug!("Reusing existing SMTP client");
            return Ok(client);
        }

        // Create new client if under limit
        if clients.len() < self.max_connections {
            debug!("Creating new SMTP client");
            let client = SmtpClient::new(self.config.clone())?;
            Ok(Arc::new(client))
        } else {
            // Wait for a client to become available or create a temporary one
            debug!("Connection pool full, creating temporary client");
            let client = SmtpClient::new(self.config.clone())?;
            Ok(Arc::new(client))
        }
    }

    /// Return a client to the pool
    pub async fn return_client(&self, client: Arc<SmtpClient>) {
        let mut clients = self.clients.write().await;
        if clients.len() < self.max_connections {
            clients.push(client);
            debug!("Returned SMTP client to pool");
        } else {
            debug!("Pool full, dropping client");
        }
    }

    /// Execute an operation with retry logic
    pub async fn with_retry<F, T>(&self, operation: F) -> Result<T, SmtpConnectionError>
    where
        F: Fn() -> Result<T, SmtpConnectionError>,
    {
        let mut delay = self.retry_config.base_delay;
        let mut last_error = None;

        for attempt in 0..=self.retry_config.max_retries {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);

                    if attempt < self.retry_config.max_retries {
                        warn!(
                            "SMTP operation failed (attempt {}/{}), retrying in {:?}",
                            attempt + 1,
                            self.retry_config.max_retries + 1,
                            delay
                        );

                        tokio::time::sleep(delay).await;
                        delay = std::cmp::min(delay * 2, self.retry_config.max_delay);
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Get connection statistics
    pub async fn stats(&self) -> ConnectionStats {
        let clients = self.clients.read().await;
        let health = self.health.read().await;

        ConnectionStats {
            active_connections: clients.len(),
            max_connections: self.max_connections,
            is_healthy: health.is_healthy,
            error_count: health.error_count,
            last_check: health.last_check,
        }
    }
}

#[derive(Debug)]
pub struct ConnectionStats {
    pub active_connections: usize,
    pub max_connections: usize,
    pub is_healthy: bool,
    pub error_count: u32,
    pub last_check: Instant,
}
