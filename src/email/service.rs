//! Email notification service using lettre

use crate::config::EmailConfig;
use lettre::{
    Message, SmtpTransport, Transport, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use tracing::{error, info};

/// Email service for sending notifications
#[derive(Clone)]
pub struct EmailService {
    mailer: SmtpTransport,
    from: String,
    admin_emails: Vec<String>,
    skip_sending: bool,
}

impl EmailService {
    /// Create a new email service from configuration
    pub fn new(config: &EmailConfig) -> anyhow::Result<Self> {
        let mailer = if config.smtp_username.is_empty() || config.smtp_password.is_empty() {
            info!(
                smtp_host = %config.smtp_host,
                smtp_port = config.smtp_port,
                "SMTP credentials not configured, using unauthenticated connection (e.g., MailDev)"
            );
            // Use builder_dangerous for unauthenticated SMTP (e.g., MailDev)
            SmtpTransport::builder_dangerous(&config.smtp_host)
                .port(config.smtp_port)
                .build()
        } else {
            info!(
                smtp_host = %config.smtp_host,
                smtp_port = config.smtp_port,
                from = %config.from_address,
                admin_count = config.admin_emails.len(),
                "Email service initialized with authentication and TLS"
            );
            // SmtpTransport::relay() uses STARTTLS by default for secure connections
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
            admin_emails: config.admin_emails.clone(),
            skip_sending: false,
        })
    }

    /// Create a mock email service for testing (skips actual SMTP)
    ///
    /// This function is intended for test use only. It creates an EmailService
    /// that logs email operations but skips actual SMTP connections.
    pub fn new_mock(config: &EmailConfig) -> anyhow::Result<Self> {
        // Create a stub transport that won't be used
        let mailer = SmtpTransport::builder_dangerous("localhost")
            .port(1025)
            .build();

        info!(
            from = %config.from_address,
            admin_count = config.admin_emails.len(),
            "Mock email service initialized (SMTP calls skipped)"
        );

        Ok(Self {
            mailer,
            from: config.from_address.clone(),
            admin_emails: config.admin_emails.clone(),
            skip_sending: true,
        })
    }

    // Send contact form notification to all admin emails
    // pub async fn send_contact_notification(
    //     &self,
    //     submission: &ContactFormSubmitted,
    // ) -> anyhow::Result<()> {
    //     info!(
    //         name = %submission.name,
    //         email = %submission.email,
    //         subject = %submission.subject,
    //         admin_count = self.admin_emails.len(),
    //         "Sending contact form notification to admins"
    //     );
    //
    //     // Skip actual SMTP in mock mode (for testing)
    //     if self.skip_sending {
    //         info!("Mock email service: Skipping actual SMTP send (test mode)");
    //         return Ok(());
    //     }
    //
    //     for admin_email in &self.admin_emails {
    //         let email_body = format!(
    //             "New Contact Form Submission\n\
    //              ============================\n\n\
    //              From: {} <{}>\n\
    //              Subject: {}\n\n\
    //              Message:\n\
    //              {}\n",
    //             submission.name, submission.email, submission.subject, submission.message
    //         );
    //
    //         match Message::builder()
    //             .from(self.from.parse()?)
    //             .to(admin_email.parse()?)
    //             .subject(format!("New Contact Form: {}", submission.subject))
    //             .header(ContentType::TEXT_PLAIN)
    //             .body(email_body)
    //         {
    //             Ok(email) => match self.mailer.send(&email) {
    //                 Ok(_) => {
    //                     info!(
    //                         admin_email = %admin_email,
    //                         from_name = %submission.name,
    //                         "Contact notification sent successfully"
    //                     );
    //                 }
    //                 Err(e) => {
    //                     error!(
    //                         error = %e,
    //                         admin_email = %admin_email,
    //                         "Failed to send contact notification via SMTP"
    //                     );
    //                     return Err(anyhow::anyhow!("SMTP error: {}", e));
    //                 }
    //             },
    //             Err(e) => {
    //                 error!(
    //                     error = %e,
    //                     admin_email = %admin_email,
    //                     "Failed to build contact notification email"
    //                 );
    //                 return Err(anyhow::anyhow!("Email building error: {}", e));
    //             }
    //         }
    //     }
    //
    //     Ok(())
    // }
}
