use super::data::{NotificationEmailData, PasswordResetEmailData, RegistrationEmailData};
use askama::Template;

#[derive(Template)]
#[template(path = "emails/registration.html")]
pub struct RegistrationEmailHtml {
    pub user_name: String,
    pub verification_url: String,
    pub app_name: String,
}

#[derive(Template)]
#[template(path = "emails/registration.txt")]
pub struct RegistrationEmailText {
    pub user_name: String,
    pub verification_url: String,
    pub app_name: String,
}

#[derive(Template)]
#[template(path = "emails/password_reset.html")]
pub struct PasswordResetEmailHtml {
    pub user_name: String,
    pub reset_url: String,
    pub expiry_hours: u32,
    pub app_name: String,
}

#[derive(Template)]
#[template(path = "emails/password_reset.txt")]
pub struct PasswordResetEmailText {
    pub user_name: String,
    pub reset_url: String,
    pub expiry_hours: u32,
    pub app_name: String,
}

#[derive(Template)]
#[template(path = "emails/notification.html")]
pub struct NotificationEmailHtml {
    pub user_name: String,
    pub notification_title: String,
    pub notification_body: String,
    pub action_url: String,
    pub action_text: String,
    pub has_action: bool,
    pub app_name: String,
}

#[derive(Template)]
#[template(path = "emails/notification.txt")]
pub struct NotificationEmailText {
    pub user_name: String,
    pub notification_title: String,
    pub notification_body: String,
    pub action_url: String,
    pub action_text: String,
    pub has_action: bool,
    pub app_name: String,
}

impl From<&RegistrationEmailData> for RegistrationEmailHtml {
    fn from(data: &RegistrationEmailData) -> Self {
        Self {
            user_name: data.user_name.clone(),
            verification_url: data.verification_url.clone(),
            app_name: data.app_name.clone(),
        }
    }
}

impl From<&RegistrationEmailData> for RegistrationEmailText {
    fn from(data: &RegistrationEmailData) -> Self {
        Self {
            user_name: data.user_name.clone(),
            verification_url: data.verification_url.clone(),
            app_name: data.app_name.clone(),
        }
    }
}

impl From<&PasswordResetEmailData> for PasswordResetEmailHtml {
    fn from(data: &PasswordResetEmailData) -> Self {
        Self {
            user_name: data.user_name.clone(),
            reset_url: data.reset_url.clone(),
            expiry_hours: data.expiry_hours,
            app_name: data.app_name.clone(),
        }
    }
}

impl From<&PasswordResetEmailData> for PasswordResetEmailText {
    fn from(data: &PasswordResetEmailData) -> Self {
        Self {
            user_name: data.user_name.clone(),
            reset_url: data.reset_url.clone(),
            expiry_hours: data.expiry_hours,
            app_name: data.app_name.clone(),
        }
    }
}

impl From<&NotificationEmailData> for NotificationEmailHtml {
    fn from(data: &NotificationEmailData) -> Self {
        let has_action = data.action_url.is_some() && data.action_text.is_some();
        Self {
            user_name: data.user_name.clone(),
            notification_title: data.notification_title.clone(),
            notification_body: data.notification_body.clone(),
            action_url: data.action_url.clone().unwrap_or_default(),
            action_text: data.action_text.clone().unwrap_or_default(),
            has_action,
            app_name: data.app_name.clone(),
        }
    }
}

impl From<&NotificationEmailData> for NotificationEmailText {
    fn from(data: &NotificationEmailData) -> Self {
        let has_action = data.action_url.is_some() && data.action_text.is_some();
        Self {
            user_name: data.user_name.clone(),
            notification_title: data.notification_title.clone(),
            notification_body: data.notification_body.clone(),
            action_url: data.action_url.clone().unwrap_or_default(),
            action_text: data.action_text.clone().unwrap_or_default(),
            has_action,
            app_name: data.app_name.clone(),
        }
    }
}
