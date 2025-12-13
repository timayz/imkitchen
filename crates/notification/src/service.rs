//! Email notification service using lettre

use lettre::{
    Message, SmtpTransport, Transport,
    message::{MultiPart, header},
    transport::smtp::authentication::Credentials,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_address: String,
    pub contact_address: String,
}

/// Email service for sending notifications
#[derive(Clone)]
pub struct EmailService {
    mailer: SmtpTransport,
    from: String,
}

impl EmailService {
    /// Create a new email service from configuration
    pub fn new(config: &EmailConfig) -> anyhow::Result<Self> {
        let mailer = if config.smtp_username.is_empty() || config.smtp_password.is_empty() {
            tracing::info!(
                smtp_host = %config.smtp_host,
                smtp_port = config.smtp_port,
                "SMTP credentials not configured, using unauthenticated connection (e.g., MailDev)"
            );
            // Use builder_dangerous for unauthenticated SMTP (e.g., MailDev)
            SmtpTransport::builder_dangerous(&config.smtp_host)
                .port(config.smtp_port)
                .build()
        } else {
            tracing::info!(
                smtp_host = %config.smtp_host,
                smtp_port = config.smtp_port,
                from = %config.from_address,
                "Email service initialized with authentication and TLS"
            );

            // This is appropriate for most SMTP servers on port 587
            let creds =
                Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());

            SmtpTransport::relay(&config.smtp_host)?
                .port(config.smtp_port)
                .credentials(creds)
                .build()
        };

        Ok(Self {
            mailer,
            from: config.from_address.clone(),
        })
    }

    pub async fn send(
        &self,
        to: impl Into<String>,
        subject: impl Into<String>,
        html: impl Into<String>,
        plain: impl Into<String>,
    ) -> anyhow::Result<()> {
        let to = to.into();
        let subject = subject.into();
        let html = html.into();
        let plain = plain.into();

        tracing::Span::current()
            .record("to", &to)
            .record("subject", &subject);

        tracing::info!("Sending email");

        let message = Message::builder()
            .from(self.from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .multipart(MultiPart::alternative_plain_html(plain, html))?;

        self.mailer.send(&message)?;

        Ok(())
    }

    pub async fn send_plain(
        &self,
        to: impl Into<String>,
        subject: impl Into<String>,
        plain: impl Into<String>,
    ) -> anyhow::Result<()> {
        let to = to.into();
        let subject = subject.into();
        let plain = plain.into();

        tracing::Span::current()
            .record("to", &to)
            .record("subject", &subject);

        tracing::info!("Sending email text plain");

        let message = Message::builder()
            .from(self.from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .header(header::ContentType::TEXT_PLAIN)
            .body(plain)?;

        self.mailer.send(&message)?;

        Ok(())
    }
}
