use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Form,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use user::validate_jwt;
use uuid::Uuid;
use validator::Validate;

use crate::routes::AppState;

#[derive(Template)]
#[template(path = "pages/privacy.html")]
struct PrivacyTemplate {
    pub user: Option<()>, // Some(()) if authenticated, None if not
    pub current_path: String,
}

#[derive(Template)]
#[template(path = "pages/terms.html")]
struct TermsTemplate {
    pub user: Option<()>, // Some(()) if authenticated, None if not
    pub current_path: String,
}

#[derive(Template)]
#[template(path = "pages/help.html")]
struct HelpTemplate {
    pub user: Option<()>, // Some(()) if authenticated, None if not
    pub current_path: String,
}

#[derive(Template)]
#[template(path = "pages/contact.html")]
struct ContactTemplate {
    pub user: Option<()>, // Some(()) if authenticated, None if not
    pub current_path: String,
}

/// GET /privacy - Privacy Policy page (public)
pub async fn get_privacy(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    // Try to extract authentication from cookie (optional - no redirect on failure)
    let user = if let Some(cookie) = jar.get("auth_token") {
        // Validate JWT
        if let Ok(claims) = validate_jwt(cookie.value(), &state.jwt_secret) {
            // Verify user exists in read model
            let user_exists = sqlx::query("SELECT id FROM users WHERE id = ?1")
                .bind(&claims.sub)
                .fetch_optional(&state.db_pool)
                .await;

            match user_exists {
                Ok(Some(_)) => Some(()), // User is authenticated
                _ => None,               // User not found or error
            }
        } else {
            None // Invalid JWT
        }
    } else {
        None // No auth cookie
    };

    let template = PrivacyTemplate {
        user,
        current_path: "/privacy".to_string(),
    };

    Html(template.render().unwrap_or_else(|e| {
        tracing::error!("Failed to render privacy template: {}", e);
        format!("Error rendering template: {}", e)
    }))
}

/// GET /terms - Terms of Service page (public)
pub async fn get_terms(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    // Try to extract authentication from cookie (optional - no redirect on failure)
    let user = if let Some(cookie) = jar.get("auth_token") {
        // Validate JWT
        if let Ok(claims) = validate_jwt(cookie.value(), &state.jwt_secret) {
            // Verify user exists in read model
            let user_exists = sqlx::query("SELECT id FROM users WHERE id = ?1")
                .bind(&claims.sub)
                .fetch_optional(&state.db_pool)
                .await;

            match user_exists {
                Ok(Some(_)) => Some(()), // User is authenticated
                _ => None,               // User not found or error
            }
        } else {
            None // Invalid JWT
        }
    } else {
        None // No auth cookie
    };

    let template = TermsTemplate {
        user,
        current_path: "/terms".to_string(),
    };

    Html(template.render().unwrap_or_else(|e| {
        tracing::error!("Failed to render terms template: {}", e);
        format!("Error rendering template: {}", e)
    }))
}

/// GET /help - Help Center page (public)
pub async fn get_help(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    // Try to extract authentication from cookie (optional - no redirect on failure)
    let user = if let Some(cookie) = jar.get("auth_token") {
        // Validate JWT
        if let Ok(claims) = validate_jwt(cookie.value(), &state.jwt_secret) {
            // Verify user exists in read model
            let user_exists = sqlx::query("SELECT id FROM users WHERE id = ?1")
                .bind(&claims.sub)
                .fetch_optional(&state.db_pool)
                .await;

            match user_exists {
                Ok(Some(_)) => Some(()), // User is authenticated
                _ => None,               // User not found or error
            }
        } else {
            None // Invalid JWT
        }
    } else {
        None // No auth cookie
    };

    let template = HelpTemplate {
        user,
        current_path: "/help".to_string(),
    };

    Html(template.render().unwrap_or_else(|e| {
        tracing::error!("Failed to render help template: {}", e);
        format!("Error rendering template: {}", e)
    }))
}

/// GET /contact - Contact Us page (public)
pub async fn get_contact(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    // Try to extract authentication from cookie (optional - no redirect on failure)
    let user = if let Some(cookie) = jar.get("auth_token") {
        // Validate JWT
        if let Ok(claims) = validate_jwt(cookie.value(), &state.jwt_secret) {
            // Verify user exists in read model
            let user_exists = sqlx::query("SELECT id FROM users WHERE id = ?1")
                .bind(&claims.sub)
                .fetch_optional(&state.db_pool)
                .await;

            match user_exists {
                Ok(Some(_)) => Some(()), // User is authenticated
                _ => None,               // User not found or error
            }
        } else {
            None // Invalid JWT
        }
    } else {
        None // No auth cookie
    };

    let template = ContactTemplate {
        user,
        current_path: "/contact".to_string(),
    };

    Html(template.render().unwrap_or_else(|e| {
        tracing::error!("Failed to render contact template: {}", e);
        format!("Error rendering template: {}", e)
    }))
}

#[derive(Deserialize, Debug, Validate)]
pub struct ContactFormData {
    #[validate(length(
        min = 2,
        max = 100,
        message = "Name must be between 2 and 100 characters"
    ))]
    pub name: String,

    #[validate(email(message = "Please enter a valid email address"))]
    pub email: String,

    #[validate(length(min = 1, message = "Please select a subject"))]
    pub subject: String,

    #[validate(length(
        min = 10,
        max = 5000,
        message = "Message must be between 10 and 5000 characters"
    ))]
    pub message: String,
}

#[derive(Template)]
#[template(path = "partials/contact-form-success.html")]
struct ContactFormSuccessTemplate {}

#[derive(Template)]
#[template(path = "partials/contact-form-error.html")]
struct ContactFormErrorTemplate {
    error_message: String,
    name: String,
    email: String,
    subject: String,
    message: String,
}

/// POST /contact - Handle contact form submission (public)
pub async fn post_contact(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form_data): Form<ContactFormData>,
) -> Response {
    // Validate form data using validator
    if let Err(validation_errors) = form_data.validate() {
        // Get the first error message
        let error_message = validation_errors
            .field_errors()
            .values()
            .next()
            .and_then(|errors| errors.first())
            .and_then(|error| error.message.as_ref())
            .map(|msg| msg.to_string())
            .unwrap_or_else(|| "Invalid form data".to_string());

        return render_error_form(&error_message, &form_data);
    }

    // Extract user_id if authenticated
    let user_id = if let Some(cookie) = jar.get("auth_token") {
        if let Ok(claims) = validate_jwt(cookie.value(), &state.jwt_secret) {
            Some(claims.sub)
        } else {
            None
        }
    } else {
        None
    };

    // Generate submission ID
    let submission_id = Uuid::new_v4().to_string();

    // Store in database
    let result = sqlx::query(
        r#"
        INSERT INTO contact_submissions (id, user_id, name, email, subject, message, created_at, status)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), 'pending')
        "#,
    )
    .bind(&submission_id)
    .bind(&user_id)
    .bind(&form_data.name)
    .bind(&form_data.email)
    .bind(&form_data.subject)
    .bind(&form_data.message)
    .execute(&state.write_pool)
    .await;

    match result {
        Ok(_) => {
            tracing::info!(
                submission_id = %submission_id,
                subject = %form_data.subject,
                user_id = ?user_id,
                "Contact form submitted"
            );

            // Send email notification to support team (async, don't block response)
            let email_config = state.email_config.clone();
            let submission_id_clone = submission_id.clone();
            let name = form_data.name.clone();
            let email = form_data.email.clone();
            let subject = form_data.subject.clone();
            let message = form_data.message.clone();
            let user_id_clone = user_id.clone();
            let submitted_at = chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string();

            tokio::spawn(async move {
                let notification = crate::email::ContactFormNotification {
                    submission_id: &submission_id_clone,
                    name: &name,
                    email: &email,
                    subject: &subject,
                    message: &message,
                    user_id: user_id_clone.as_deref(),
                    submitted_at: &submitted_at,
                };

                if let Err(e) =
                    crate::email::send_contact_form_notification(&notification, &email_config).await
                {
                    tracing::error!(
                        error = ?e,
                        submission_id = %submission_id_clone,
                        "Failed to send contact form notification email"
                    );
                }
            });

            // Return success template with fresh form
            let template = ContactFormSuccessTemplate {};
            Html(template.render().unwrap_or_else(|e| {
                tracing::error!("Failed to render success template: {}", e);
                format!("Error: {}", e)
            }))
            .into_response()
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to save contact form submission");
            render_error_form(
                "An error occurred. Please try again or email us directly at support@imkitchen.app",
                &form_data,
            )
        }
    }
}

fn render_error_form(error_message: &str, form_data: &ContactFormData) -> Response {
    let template = ContactFormErrorTemplate {
        error_message: error_message.to_string(),
        name: form_data.name.clone(),
        email: form_data.email.clone(),
        subject: form_data.subject.clone(),
        message: form_data.message.clone(),
    };

    Html(template.render().unwrap_or_else(|e| {
        tracing::error!("Failed to render error form template: {}", e);
        format!("Error: {}", e)
    }))
    .into_response()
}
