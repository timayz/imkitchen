//! Registration route handlers

use super::{render_template, AppState};
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
};
use axum_extra::extract::Form;
use imkitchen_user::aggregate::User;
use imkitchen_user::command::{Command, RegisterUserInput};
use imkitchen_user::event::EventMetadata;
use serde::Deserialize;
use tracing::{error, info};
use ulid::Ulid;

/// Registration page template
#[derive(Template)]
#[template(path = "pages/auth/register.html")]
struct RegisterPageTemplate {
    error: Option<String>,
}

/// Registration pending template
#[derive(Template)]
#[template(path = "partials/auth/register-pending.html")]
struct RegisterPendingTemplate {
    user_id: String,
}

/// Registration form data
#[derive(Deserialize)]
pub struct RegisterForm {
    email: String,
    password: String,
}

/// GET /auth/register - Show registration form
pub async fn get_register() -> Response {
    render_template(RegisterPageTemplate { error: None })
}

/// POST /auth/register - Handle registration submission
pub async fn post_register(
    State(state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Response {
    info!(email = %form.email, "Processing registration");

    let input = RegisterUserInput {
        email: form.email.clone(),
        password: form.password,
        is_admin: None, // Defaults to false for regular user registration
    };

    let metadata = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    let command = Command::new(state.evento.clone());

    match command.register_user(input, metadata).await {
        Ok(user_id) => {
            info!(user_id = %user_id, "User registration initiated");
            render_template(RegisterPendingTemplate { user_id })
        }
        Err(e) => {
            error!(error = %e, email = %form.email, "Registration failed");
            render_template(RegisterPageTemplate {
                error: Some(
                    "Failed to register. Please check your input and try again.".to_string(),
                ),
            })
        }
    }
}

/// GET /auth/register/status/{user_id} - Poll registration status
pub async fn get_register_status(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Response {
    // Load user aggregate to check status
    match evento::load::<User, _>(&state.evento, &user_id).await {
        Ok(user_result) => {
            let user = &user_result.item;

            match user.status.as_deref() {
                Some("active") => {
                    // Registration succeeded - redirect to login using ts-location
                    let mut response = Html("").into_response();
                    response
                        .headers_mut()
                        .insert("ts-location", "/auth/login".parse().unwrap());
                    response
                }
                Some("failed") => {
                    // Registration failed - show error
                    render_template(RegisterPageTemplate {
                        error: Some("Email already registered".to_string()),
                    })
                }
                _ => {
                    // Still pending
                    render_template(RegisterPendingTemplate { user_id })
                }
            }
        }
        Err(_) => {
            // User not found yet - still pending
            render_template(RegisterPendingTemplate { user_id })
        }
    }
}
