use super::data::{NotificationEmailData, PasswordResetEmailData, RegistrationEmailData};
use super::templates::*;
use askama::Template;
use thiserror::Error;
use validator::Validate;

#[derive(Debug, Error)]
pub enum EmailTemplateError {
    #[error("Template rendering failed: {0}")]
    RenderingFailed(String),

    #[error("Missing required data: {0}")]
    MissingRequiredData(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(#[from] validator::ValidationErrors),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),
}

/// Rendered email with both HTML and text versions
#[derive(Debug, Clone)]
pub struct EmailTemplate {
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

/// Email template renderer with type-safe data binding
pub struct EmailTemplateRenderer {
    // Future: Add template caching, localization support
}

impl EmailTemplateRenderer {
    /// Create a new email template renderer
    pub fn new() -> Self {
        Self {}
    }

    /// Render registration email template
    pub async fn render_registration_email(
        &self,
        data: &RegistrationEmailData,
    ) -> Result<EmailTemplate, EmailTemplateError> {
        // Validate input data
        data.validate()?;

        // Handle empty user name gracefully
        if data.user_name.trim().is_empty() {
            return Err(EmailTemplateError::MissingRequiredData(
                "User name cannot be empty".to_string(),
            ));
        }

        // Render HTML template
        let html_template = RegistrationEmailHtml::from(data);
        let html_body = html_template
            .render()
            .map_err(|e| EmailTemplateError::RenderingFailed(e.to_string()))?;

        // Render text template
        let text_template = RegistrationEmailText::from(data);
        let text_body = text_template
            .render()
            .map_err(|e| EmailTemplateError::RenderingFailed(e.to_string()))?;

        Ok(EmailTemplate {
            subject: format!("Welcome to {} - Please verify your email", data.app_name),
            html_body,
            text_body,
        })
    }

    /// Render password reset email template
    pub async fn render_password_reset_email(
        &self,
        data: &PasswordResetEmailData,
    ) -> Result<EmailTemplate, EmailTemplateError> {
        // Validate input data
        data.validate()?;

        // Render HTML template
        let html_template = PasswordResetEmailHtml::from(data);
        let html_body = html_template
            .render()
            .map_err(|e| EmailTemplateError::RenderingFailed(e.to_string()))?;

        // Render text template
        let text_template = PasswordResetEmailText::from(data);
        let text_body = text_template
            .render()
            .map_err(|e| EmailTemplateError::RenderingFailed(e.to_string()))?;

        Ok(EmailTemplate {
            subject: format!("Reset your {} password", data.app_name),
            html_body,
            text_body,
        })
    }

    /// Render notification email template
    pub async fn render_notification_email(
        &self,
        data: &NotificationEmailData,
    ) -> Result<EmailTemplate, EmailTemplateError> {
        // Validate input data
        data.validate()?;

        // Render HTML template
        let html_template = NotificationEmailHtml::from(data);
        let html_body = html_template
            .render()
            .map_err(|e| EmailTemplateError::RenderingFailed(e.to_string()))?;

        // Render text template
        let text_template = NotificationEmailText::from(data);
        let text_body = text_template
            .render()
            .map_err(|e| EmailTemplateError::RenderingFailed(e.to_string()))?;

        Ok(EmailTemplate {
            subject: format!("{} - {}", data.notification_title, data.app_name),
            html_body,
            text_body,
        })
    }

    /// Generate preview data for development testing
    pub async fn generate_preview_data(&self) -> Result<EmailPreviewData, EmailTemplateError> {
        let registration_data = RegistrationEmailData {
            user_name: "John Doe".to_string(),
            verification_url: "https://imkitchen.com/verify?token=preview123".to_string(),
            app_name: "IMKitchen".to_string(),
        };

        let password_reset_data = PasswordResetEmailData {
            user_name: "Jane Smith".to_string(),
            reset_url: "https://imkitchen.com/reset?token=preview456".to_string(),
            expiry_hours: 24,
            app_name: "IMKitchen".to_string(),
        };

        let notification_data = NotificationEmailData {
            user_name: "Bob Wilson".to_string(),
            notification_title: "Your Weekly Meal Plan is Ready".to_string(),
            notification_body: "We've prepared your personalized meal plan for next week. Check it out and start cooking!".to_string(),
            action_url: Some("https://imkitchen.com/meal-plan/preview".to_string()),
            action_text: Some("View Meal Plan".to_string()),
            app_name: "IMKitchen".to_string(),
        };

        Ok(EmailPreviewData {
            registration: self.render_registration_email(&registration_data).await?,
            password_reset: self
                .render_password_reset_email(&password_reset_data)
                .await?,
            notification: self.render_notification_email(&notification_data).await?,
        })
    }

    /// Validate template HTML structure
    pub fn validate_html_structure(&self, html: &str) -> bool {
        // Basic HTML structure validation
        html.contains("<!DOCTYPE html>")
            && html.contains("<html")
            && html.contains("<head>")
            && html.contains("<body>")
            && html.contains("</html>")
    }

    /// Validate text alternative (no HTML tags)
    pub fn validate_text_alternative(&self, text: &str) -> bool {
        !text.contains('<') && !text.contains('>')
    }
}

impl Default for EmailTemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Preview data for all email templates
#[derive(Debug)]
pub struct EmailPreviewData {
    pub registration: EmailTemplate,
    pub password_reset: EmailTemplate,
    pub notification: EmailTemplate,
}
