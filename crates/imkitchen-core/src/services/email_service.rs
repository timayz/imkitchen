use thiserror::Error;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Email service unavailable")]
    ServiceUnavailable,
    #[error("Invalid email address: {0}")]
    InvalidAddress(String),
    #[error("Template error: {0}")]
    TemplateError(String),
}

#[derive(Debug, Clone)]
pub struct EmailTemplate {
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

#[derive(Clone)]
pub struct EmailService {
    // For now, we'll use development mode (logging only)
    // In production, this would contain SMTP client or API credentials
    development_mode: bool,
}

impl EmailService {
    pub fn new(development_mode: bool) -> Self {
        Self { development_mode }
    }

    /// Send email verification email
    pub async fn send_verification_email(
        &self,
        email: &str,
        name: &str,
        verification_token: &str,
    ) -> Result<(), EmailError> {
        let template = self.create_verification_email_template(name, verification_token)?;

        if self.development_mode {
            self.log_email_for_development(email, &template).await;
            Ok(())
        } else {
            // TODO: Implement actual email sending for production
            warn!(
                "Production email sending not implemented yet, falling back to development logging"
            );
            self.log_email_for_development(email, &template).await;
            Ok(())
        }
    }

    /// Send password reset email
    pub async fn send_password_reset_email(
        &self,
        email: &str,
        name: &str,
        reset_token: &str,
    ) -> Result<(), EmailError> {
        let template = self.create_password_reset_email_template(name, reset_token)?;

        if self.development_mode {
            self.log_email_for_development(email, &template).await;
            Ok(())
        } else {
            // TODO: Implement actual email sending for production
            warn!(
                "Production email sending not implemented yet, falling back to development logging"
            );
            self.log_email_for_development(email, &template).await;
            Ok(())
        }
    }

    /// Create email verification template
    fn create_verification_email_template(
        &self,
        name: &str,
        verification_token: &str,
    ) -> Result<EmailTemplate, EmailError> {
        let verification_url = format!(
            "http://localhost:3000/verify-email?token={}",
            verification_token
        );

        let subject = "Verify Your ImKitchen Account".to_string();

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>Verify Your Account</title>
            </head>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
                <div style="background-color: #f8f9fa; padding: 20px; text-align: center;">
                    <h1 style="color: #343a40;">Welcome to ImKitchen!</h1>
                </div>
                <div style="padding: 20px;">
                    <p>Hello {name},</p>
                    
                    <p>Thank you for registering with ImKitchen! To complete your registration and start planning delicious meals, please verify your email address.</p>
                    
                    <div style="text-align: center; margin: 30px 0;">
                        <a href="{verification_url}" 
                           style="background-color: #007bff; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; display: inline-block;">
                            Verify Email Address
                        </a>
                    </div>
                    
                    <p>If the button doesn't work, you can copy and paste this link into your browser:</p>
                    <p style="word-break: break-all; color: #6c757d;">{verification_url}</p>
                    
                    <p><strong>This link will expire in 7 days.</strong></p>
                    
                    <hr style="margin: 30px 0;">
                    <p style="color: #6c757d; font-size: 14px;">
                        If you didn't create an account with ImKitchen, you can safely ignore this email.
                    </p>
                </div>
            </body>
            </html>
            "#,
            name = name,
            verification_url = verification_url
        );

        let text_body = format!(
            r#"Welcome to ImKitchen!

Hello {name},

Thank you for registering with ImKitchen! To complete your registration and start planning delicious meals, please verify your email address.

Please copy and paste this link into your browser to verify your account:
{verification_url}

This link will expire in 7 days.

If you didn't create an account with ImKitchen, you can safely ignore this email.

--
The ImKitchen Team
            "#,
            name = name,
            verification_url = verification_url
        );

        Ok(EmailTemplate {
            subject,
            html_body,
            text_body,
        })
    }

    /// Create password reset template
    fn create_password_reset_email_template(
        &self,
        name: &str,
        reset_token: &str,
    ) -> Result<EmailTemplate, EmailError> {
        let reset_url = format!("http://localhost:3000/reset-password?token={}", reset_token);

        let subject = "Reset Your ImKitchen Password".to_string();

        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>Reset Your Password</title>
            </head>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
                <div style="background-color: #f8f9fa; padding: 20px; text-align: center;">
                    <h1 style="color: #343a40;">Password Reset Request</h1>
                </div>
                <div style="padding: 20px;">
                    <p>Hello {name},</p>
                    
                    <p>We received a request to reset your ImKitchen account password. If you made this request, please click the button below to reset your password.</p>
                    
                    <div style="text-align: center; margin: 30px 0;">
                        <a href="{reset_url}" 
                           style="background-color: #dc3545; color: white; padding: 12px 30px; text-decoration: none; border-radius: 5px; display: inline-block;">
                            Reset Password
                        </a>
                    </div>
                    
                    <p>If the button doesn't work, you can copy and paste this link into your browser:</p>
                    <p style="word-break: break-all; color: #6c757d;">{reset_url}</p>
                    
                    <p><strong>This link will expire in 1 hour.</strong></p>
                    
                    <hr style="margin: 30px 0;">
                    <p style="color: #6c757d; font-size: 14px;">
                        If you didn't request a password reset, you can safely ignore this email. Your password will not be changed.
                    </p>
                </div>
            </body>
            </html>
            "#,
            name = name,
            reset_url = reset_url
        );

        let text_body = format!(
            r#"Password Reset Request

Hello {name},

We received a request to reset your ImKitchen account password. If you made this request, please copy and paste this link into your browser to reset your password:

{reset_url}

This link will expire in 1 hour.

If you didn't request a password reset, you can safely ignore this email. Your password will not be changed.

--
The ImKitchen Team
            "#,
            name = name,
            reset_url = reset_url
        );

        Ok(EmailTemplate {
            subject,
            html_body,
            text_body,
        })
    }

    /// Log email to console for development
    async fn log_email_for_development(&self, email: &str, template: &EmailTemplate) {
        let email_id = Uuid::new_v4();

        info!("📧 Development Email Sent (ID: {})", email_id);
        info!("   To: {}", email);
        info!("   Subject: {}", template.subject);
        info!("   --- Email Content (Text) ---");
        for line in template.text_body.lines() {
            info!("   {}", line);
        }
        info!("   --- End Email Content ---");
        info!("   HTML version available but not displayed in logs");
    }

    /// Validate email address format
    #[allow(dead_code)]
    fn is_valid_email(&self, email: &str) -> bool {
        // Basic email validation - in production would use a proper email validation library
        email.contains('@') && email.contains('.') && email.len() > 5 && email.len() < 255
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_email_service_creation() {
        let email_service = EmailService::new(true);
        assert!(email_service.development_mode);
    }

    #[tokio::test]
    async fn test_verification_email_template() {
        let email_service = EmailService::new(true);
        let template = email_service
            .create_verification_email_template("Test User", "test-token")
            .unwrap();

        assert_eq!(template.subject, "Verify Your ImKitchen Account");
        assert!(template.html_body.contains("Test User"));
        assert!(template.html_body.contains("test-token"));
        assert!(template.text_body.contains("Test User"));
        assert!(template.text_body.contains("test-token"));
    }

    #[tokio::test]
    async fn test_password_reset_email_template() {
        let email_service = EmailService::new(true);
        let template = email_service
            .create_password_reset_email_template("Test User", "reset-token")
            .unwrap();

        assert_eq!(template.subject, "Reset Your ImKitchen Password");
        assert!(template.html_body.contains("Test User"));
        assert!(template.html_body.contains("reset-token"));
        assert!(template.text_body.contains("Test User"));
        assert!(template.text_body.contains("reset-token"));
    }

    #[tokio::test]
    async fn test_send_verification_email() {
        let email_service = EmailService::new(true);
        let result = email_service
            .send_verification_email("test@example.com", "Test User", "verification-token")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_password_reset_email() {
        let email_service = EmailService::new(true);
        let result = email_service
            .send_password_reset_email("test@example.com", "Test User", "reset-token")
            .await;

        assert!(result.is_ok());
    }
}
