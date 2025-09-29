use serde::{Deserialize, Serialize};
use validator::Validate;

/// Data for registration email template
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegistrationEmailData {
    #[validate(length(min = 1, message = "User name is required"))]
    pub user_name: String,

    #[validate(url(message = "Verification URL must be valid"))]
    pub verification_url: String,

    #[validate(length(min = 1, message = "App name is required"))]
    pub app_name: String,
}

/// Data for password reset email template
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PasswordResetEmailData {
    #[validate(length(min = 1, message = "User name is required"))]
    pub user_name: String,

    #[validate(url(message = "Reset URL must be valid"))]
    pub reset_url: String,

    #[validate(range(min = 1, max = 72, message = "Expiry hours must be between 1 and 72"))]
    pub expiry_hours: u32,

    #[validate(length(min = 1, message = "App name is required"))]
    pub app_name: String,
}

/// Data for notification email template
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NotificationEmailData {
    #[validate(length(min = 1, message = "User name is required"))]
    pub user_name: String,

    #[validate(length(min = 1, message = "Notification title is required"))]
    pub notification_title: String,

    #[validate(length(min = 1, message = "Notification body is required"))]
    pub notification_body: String,

    #[validate(url(message = "Action URL must be valid"))]
    pub action_url: Option<String>,

    pub action_text: Option<String>,

    #[validate(length(min = 1, message = "App name is required"))]
    pub app_name: String,
}

/// Unified trait for email template data
pub trait EmailTemplateData {
    fn validate(&self) -> Result<(), validator::ValidationErrors>;
    fn user_name(&self) -> &str;
    fn app_name(&self) -> &str;
}
