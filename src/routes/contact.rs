//! Contact form route handlers

use super::{render_template, AppState};
use askama::Template;
use axum::{extract::State, response::Response};
use axum_extra::extract::Form;
use imkitchen_user::command::{Command, SubmitContactFormInput};
use imkitchen_user::event::EventMetadata;
use serde::Deserialize;
use tracing::{error, info};
use ulid::Ulid;

/// Contact form page template
#[derive(Template)]
#[template(path = "pages/contact.html")]
struct ContactPageTemplate {
    error: Option<String>,
}

/// Contact success template
#[derive(Template)]
#[template(path = "partials/contact-success.html")]
struct ContactSuccessTemplate {
    message_id: String,
}

/// Contact form data
#[derive(Deserialize)]
pub struct ContactForm {
    name: String,
    email: String,
    subject: String,
    message: String,
}

/// GET /contact - Show contact form (public access)
pub async fn get_contact() -> Response {
    render_template(ContactPageTemplate { error: None })
}

/// POST /contact - Handle contact form submission (public access)
pub async fn post_contact(
    State(state): State<AppState>,
    Form(form): Form<ContactForm>,
) -> Response {
    info!(
        email = %form.email,
        subject = %form.subject,
        "Processing contact form submission"
    );

    let input = SubmitContactFormInput {
        name: form.name.clone(),
        email: form.email.clone(),
        subject: form.subject.clone(),
        message: form.message.clone(),
    };

    let metadata = EventMetadata {
        user_id: None, // Public form - no user authentication required
        request_id: Ulid::new().to_string(),
    };

    let command = Command::new(state.evento.clone());

    match command.submit_contact_form(input, metadata).await {
        Ok(message_id) => {
            info!(
                message_id = %message_id,
                email = %form.email,
                "Contact form submitted successfully"
            );
            render_template(ContactSuccessTemplate { message_id })
        }
        Err(e) => {
            error!(
                error = %e,
                email = %form.email,
                "Contact form submission failed"
            );
            render_template(ContactPageTemplate {
                error: Some(format!("Failed to send message: {}", e)),
            })
        }
    }
}
