pub mod data;
pub mod renderer;
pub mod templates;

pub use data::{
    EmailTemplateData, NotificationEmailData, PasswordResetEmailData, RegistrationEmailData,
};
pub use renderer::{EmailPreviewData, EmailTemplate, EmailTemplateError, EmailTemplateRenderer};
pub use templates::*;
