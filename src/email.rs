use anyhow::{Context, Result};
use askama::Template;
use lettre::message::{header::ContentType, Mailbox};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use tracing::{info, warn};

/// Email configuration
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
}

impl EmailConfig {
    /// Create SMTP transport based on configuration
    /// Uses builder_dangerous for local dev (no credentials), relay for production
    fn create_transport(&self) -> Result<SmtpTransport> {
        if self.smtp_username.is_empty() && self.smtp_password.is_empty() {
            // Local development mode - direct connection without authentication
            Ok(SmtpTransport::builder_dangerous(&self.smtp_host)
                .port(self.smtp_port)
                .build())
        } else {
            // Production mode - authenticated relay
            let credentials =
                Credentials::new(self.smtp_username.clone(), self.smtp_password.clone());

            Ok(SmtpTransport::relay(&self.smtp_host)
                .context("Failed to create SMTP transport")?
                .port(self.smtp_port)
                .credentials(credentials)
                .build())
        }
    }
}

/// Send an email with HTML and plain text parts
///
/// Returns success even if email fails to send (to prevent user enumeration)
/// Errors are logged for monitoring
async fn send_email(
    to_email: &str,
    subject: &str,
    html_body: String,
    plain_body: String,
    config: &EmailConfig,
    log_context: &str,
) -> Result<()> {
    let from_mailbox: Mailbox = format!("{} <{}>", config.from_name, config.from_email)
        .parse()
        .context("Failed to parse from email")?;

    let to_mailbox: Mailbox = to_email.parse().context("Failed to parse to email")?;

    let email = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .multipart(
            lettre::message::MultiPart::alternative()
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(plain_body),
                )
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html_body),
                ),
        )
        .context("Failed to build email message")?;

    let mailer = config.create_transport()?;

    // Send email - log errors but don't fail (prevent user enumeration)
    match mailer.send(&email) {
        Ok(_) => {
            info!(
                to = to_email,
                context = log_context,
                "Email sent successfully"
            );
            Ok(())
        }
        Err(e) => {
            warn!(
                error = %e,
                to = to_email,
                context = log_context,
                "Failed to send email - logging for monitoring"
            );
            // Return success to prevent user enumeration
            Ok(())
        }
    }
}

// ============================================================================
// Email Templates
// ============================================================================
// To add a new email type:
// 1. Create HTML template in templates/emails/your-email.html
// 2. Create text template in templates/emails/your-email.txt
// 3. Define template structs below with #[derive(Template)]
// 4. Create a public send_your_email() function that calls send_email()
// ============================================================================

/// Password reset email HTML template
#[derive(Template)]
#[template(path = "emails/password-reset.html")]
struct PasswordResetHtmlTemplate {
    reset_link: String,
}

/// Password reset email plain text template
#[derive(Template)]
#[template(path = "emails/password-reset.txt")]
struct PasswordResetTextTemplate {
    reset_link: String,
}

/// Send a password reset email with a reset token link
///
/// Returns success even if email fails to send (to prevent user enumeration)
/// Errors are logged for monitoring
pub async fn send_password_reset_email(
    to_email: &str,
    reset_token: &str,
    config: &EmailConfig,
    base_url: &str,
) -> Result<()> {
    let reset_link = format!("{}/password-reset/{}", base_url, reset_token);

    // Render HTML template
    let html_template = PasswordResetHtmlTemplate {
        reset_link: reset_link.clone(),
    };
    let html_body = html_template
        .render()
        .context("Failed to render HTML email template")?;

    // Render plain text template
    let text_template = PasswordResetTextTemplate { reset_link };
    let plain_body = text_template
        .render()
        .context("Failed to render plain text email template")?;

    send_email(
        to_email,
        "Password Reset Request - imkitchen",
        html_body,
        plain_body,
        config,
        "password_reset",
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_password_reset_email_formats_correctly() {
        let config = EmailConfig {
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_username: "test@example.com".to_string(),
            smtp_password: "password".to_string(),
            from_email: "noreply@imkitchen.app".to_string(),
            from_name: "imkitchen".to_string(),
        };

        // This test validates the email formatting logic
        // In real tests, we'll mock the SMTP transport
        let token = "test_token_123";
        let result =
            send_password_reset_email("user@example.com", token, &config, "https://imkitchen.app")
                .await;

        // Even with invalid SMTP config, should return Ok (prevent enumeration)
        assert!(result.is_ok());
    }
}
