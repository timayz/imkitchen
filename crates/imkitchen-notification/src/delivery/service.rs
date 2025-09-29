use crate::{
    smtp::SmtpConnectionManager,
    templates::{
        EmailTemplateError, EmailTemplateRenderer, NotificationEmailData, PasswordResetEmailData,
        RegistrationEmailData,
    },
};
use lettre::{message::header::ContentType, Message, Transport};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

#[derive(Debug, Error)]
pub enum EmailDeliveryError {
    #[error("SMTP connection failed: {0}")]
    SmtpConnectionFailed(String),

    #[error("Template rendering failed: {0}")]
    TemplateRenderingFailed(#[from] EmailTemplateError),

    #[error("Email delivery failed: {0}")]
    DeliveryFailed(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid email address: {0}")]
    InvalidEmailAddress(String),

    #[error("Message building failed: {0}")]
    MessageBuildingFailed(String),
}

#[derive(Debug, Clone)]
pub enum EmailStatus {
    Queued,
    Sent,
    Failed,
    Retrying,
}

#[derive(Debug, Clone)]
pub struct DeliveryAttempt {
    pub timestamp: Instant,
    pub status: EmailStatus,
    pub error_message: Option<String>,
    pub retry_count: u32,
}

#[derive(Debug, Clone)]
pub struct DeliveryStats {
    pub total_attempts: u64,
    pub successful_deliveries: u64,
    pub failed_deliveries: u64,
    pub retry_attempts: u64,
    pub rate_limit_hits: u64,
}

#[derive(Debug)]
struct RateLimiter {
    max_emails: u32,
    time_window: Duration,
    attempts: VecDeque<Instant>,
}

impl RateLimiter {
    fn new(max_emails: u32, time_window: Duration) -> Self {
        Self {
            max_emails,
            time_window,
            attempts: VecDeque::new(),
        }
    }

    fn can_send(&mut self) -> bool {
        let now = Instant::now();

        // Remove old attempts outside the time window
        while let Some(&front) = self.attempts.front() {
            if now.duration_since(front) > self.time_window {
                self.attempts.pop_front();
            } else {
                break;
            }
        }

        self.attempts.len() < self.max_emails as usize
    }

    fn record_attempt(&mut self) {
        self.attempts.push_back(Instant::now());
    }
}

/// Email delivery service with async sending and status tracking
pub struct EmailDeliveryService {
    connection_manager: Arc<SmtpConnectionManager>,
    template_renderer: EmailTemplateRenderer,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    stats: Arc<RwLock<DeliveryStats>>,
}

impl EmailDeliveryService {
    /// Create a new email delivery service
    pub fn new(
        connection_manager: SmtpConnectionManager,
        template_renderer: EmailTemplateRenderer,
    ) -> Result<Self, EmailDeliveryError> {
        info!("Creating email delivery service");

        Ok(Self {
            connection_manager: Arc::new(connection_manager),
            template_renderer,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(100, Duration::from_secs(3600)))), // 100 emails per hour default
            stats: Arc::new(RwLock::new(DeliveryStats {
                total_attempts: 0,
                successful_deliveries: 0,
                failed_deliveries: 0,
                retry_attempts: 0,
                rate_limit_hits: 0,
            })),
        })
    }

    /// Set rate limiting parameters
    pub fn set_rate_limit(&mut self, max_emails: u32, time_window: Duration) {
        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(max_emails, time_window)));
        self.rate_limiter = rate_limiter;
        info!(
            "Rate limit set to {} emails per {:?}",
            max_emails, time_window
        );
    }

    /// Send registration email
    pub async fn send_registration_email(
        &self,
        to_email: &str,
        data: &RegistrationEmailData,
    ) -> Result<EmailStatus, EmailDeliveryError> {
        debug!("Sending registration email to: {}", to_email);

        // Check rate limiting
        {
            let mut rate_limiter = self.rate_limiter.lock().await;
            if !rate_limiter.can_send() {
                let mut stats = self.stats.write().await;
                stats.rate_limit_hits += 1;
                warn!("Rate limit exceeded for email to: {}", to_email);
                return Err(EmailDeliveryError::RateLimitExceeded);
            }
            rate_limiter.record_attempt();
        }

        // Render template
        let email_template = self
            .template_renderer
            .render_registration_email(data)
            .await?;

        // Send email
        self.send_email(
            to_email,
            &email_template.subject,
            &email_template.html_body,
            &email_template.text_body,
        )
        .await
    }

    /// Send password reset email
    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        data: &PasswordResetEmailData,
    ) -> Result<EmailStatus, EmailDeliveryError> {
        debug!("Sending password reset email to: {}", to_email);

        // Check rate limiting
        {
            let mut rate_limiter = self.rate_limiter.lock().await;
            if !rate_limiter.can_send() {
                let mut stats = self.stats.write().await;
                stats.rate_limit_hits += 1;
                warn!("Rate limit exceeded for email to: {}", to_email);
                return Err(EmailDeliveryError::RateLimitExceeded);
            }
            rate_limiter.record_attempt();
        }

        // Render template
        let email_template = self
            .template_renderer
            .render_password_reset_email(data)
            .await?;

        // Send email
        self.send_email(
            to_email,
            &email_template.subject,
            &email_template.html_body,
            &email_template.text_body,
        )
        .await
    }

    /// Send notification email
    pub async fn send_notification_email(
        &self,
        to_email: &str,
        data: &NotificationEmailData,
    ) -> Result<EmailStatus, EmailDeliveryError> {
        debug!("Sending notification email to: {}", to_email);

        // Check rate limiting
        {
            let mut rate_limiter = self.rate_limiter.lock().await;
            if !rate_limiter.can_send() {
                let mut stats = self.stats.write().await;
                stats.rate_limit_hits += 1;
                warn!("Rate limit exceeded for email to: {}", to_email);
                return Err(EmailDeliveryError::RateLimitExceeded);
            }
            rate_limiter.record_attempt();
        }

        // Render template
        let email_template = self
            .template_renderer
            .render_notification_email(data)
            .await?;

        // Send email
        self.send_email(
            to_email,
            &email_template.subject,
            &email_template.html_body,
            &email_template.text_body,
        )
        .await
    }

    /// Internal method to send email via SMTP
    async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<EmailStatus, EmailDeliveryError> {
        let mut stats = self.stats.write().await;
        stats.total_attempts += 1;
        drop(stats);

        // Get SMTP client from connection manager
        let client = self
            .connection_manager
            .get_client()
            .await
            .map_err(|e| EmailDeliveryError::SmtpConnectionFailed(e.to_string()))?;

        let config = client.config();

        // Build email message
        let email = Message::builder()
            .from(
                format!("{} <{}>", config.from_name, config.from_email)
                    .parse()
                    .map_err(|e| {
                        EmailDeliveryError::InvalidEmailAddress(format!("From address: {}", e))
                    })?,
            )
            .to(to_email.parse().map_err(|e| {
                EmailDeliveryError::InvalidEmailAddress(format!("To address: {}", e))
            })?)
            .subject(subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text_body.to_string()),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_body.to_string()),
                    ),
            )
            .map_err(|e| EmailDeliveryError::MessageBuildingFailed(e.to_string()))?;

        // Send the email
        match client.transport().send(&email) {
            Ok(_) => {
                info!("Email sent successfully to: {}", to_email);
                let mut stats = self.stats.write().await;
                stats.successful_deliveries += 1;
                Ok(EmailStatus::Sent)
            }
            Err(e) => {
                error!("Failed to send email to {}: {}", to_email, e);
                let mut stats = self.stats.write().await;
                stats.failed_deliveries += 1;

                // Return the client to the pool
                self.connection_manager.return_client(client).await;

                Err(EmailDeliveryError::DeliveryFailed(e.to_string()))
            }
        }
    }

    /// Get delivery statistics
    pub async fn get_delivery_stats(&self) -> DeliveryStats {
        self.stats.read().await.clone()
    }

    /// Check if the service is healthy
    pub async fn health_check(&self) -> bool {
        self.connection_manager.is_healthy().await
    }
}
