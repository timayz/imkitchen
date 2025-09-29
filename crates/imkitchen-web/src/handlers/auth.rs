// Authentication handlers with sync validation

use askama::Template;
use axum::{
    extract::{Form, Query, State},
    http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use validator::Validate;

use crate::middleware::auth::create_session_cookie_header;
use crate::AppState;
use imkitchen_shared::{Email, FamilySize, Password, SkillLevel};
use imkitchen_user::commands::register_user::{RegisterUserCommand, RegisterUserError};
use imkitchen_user::services::login_service::{LoginCommand, LoginError};

/// Login form data with validation
#[derive(Debug, Deserialize, Validate)]
pub struct LoginForm {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,
}

/// Query parameters for login page
#[derive(Debug, Deserialize)]
pub struct LoginQuery {
    pub success: Option<String>,
}

/// Registration form data with validation
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterForm {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,

    #[validate(length(min = 8))]
    pub password_confirm: String,
}

impl RegisterForm {
    /// Custom validation to check password confirmation
    pub fn validate_passwords_match(&self) -> Result<(), String> {
        if self.password != self.password_confirm {
            Err("Passwords do not match".to_string())
        } else {
            Ok(())
        }
    }
}

/// Validation error response for HTMX/TwinSpark
#[derive(Debug, Serialize)]
pub struct ValidationErrorResponse {
    pub errors: std::collections::HashMap<String, Vec<String>>,
}

/// Success response for login
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub redirect: String,
}

/// Askama template for login form (old simple version)
#[derive(Template)]
#[template(path = "auth/login_form.html")]
pub struct LoginFormTemplate {
    pub general_error: String,
}

/// Askama template for enhanced login page
#[derive(Template)]
#[template(path = "auth/login.html")]
pub struct LoginPageTemplate {
    pub general_error: String,
    pub success_message: String,
}

/// Askama template for registration page
#[derive(Template)]
#[template(path = "auth/register.html")]
pub struct RegisterPageTemplate {
    pub general_error: String,
}

/// Askama template for validation error fragments
#[derive(Template)]
#[template(path = "auth/validation_errors.html")]
pub struct ValidationErrorsTemplate {
    pub errors: std::collections::HashMap<String, Vec<String>>,
}

/// Login handler with form processing using DirectLoginService
pub async fn login_handler(
    State(app_state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Result<Response, StatusCode> {
    info!("Processing login for email: {}", form.email);

    // First validate the form
    if let Err(_validation_errors) = form.validate() {
        error!("Login form validation failed");
        let template = LoginFormTemplate {
            general_error: "Please check your input and try again".to_string(),
        };
        let html = template.to_string();
        return Ok(Html(html).into_response());
    }

    // Parse email and password using our domain value objects
    let email = match Email::new(form.email) {
        Ok(email) => email,
        Err(e) => {
            error!("Invalid email format: {:?}", e);
            let template = LoginFormTemplate {
                general_error: "Invalid email format".to_string(),
            };
            let html = template.to_string();
            return Ok(Html(html).into_response());
        }
    };

    let password = match Password::new(form.password) {
        Ok(password) => password,
        Err(e) => {
            error!("Invalid password: {:?}", e);
            let template = LoginFormTemplate {
                general_error: "Password does not meet requirements".to_string(),
            };
            let html = template.to_string();
            return Ok(Html(html).into_response());
        }
    };

    // Use DirectLoginService if available
    if let Some(ref login_service) = app_state.login_service {
        let login_command = LoginCommand::new(email.clone(), password);

        match login_service.login(login_command).await {
            Ok(login_response) => {
                info!("Login successful for user: {}", email.value);

                // Create session cookie header
                let cookie_header = create_session_cookie_header(login_response.user_id);

                // Return HTTP redirect to dashboard with cookie header
                let mut response = Redirect::to("/dashboard").into_response();
                response
                    .headers_mut()
                    .insert(SET_COOKIE, cookie_header.parse().unwrap());
                Ok(response)
            }
            Err(LoginError::InvalidCredentials) => {
                error!("Invalid credentials for email: {}", email.value);
                let template = LoginFormTemplate {
                    general_error: "Invalid email or password".to_string(),
                };
                let html = template.to_string();
                Ok(Html(html).into_response())
            }
            Err(e) => {
                error!("Login error: {:?}", e);
                let template = LoginFormTemplate {
                    general_error: "Login failed. Please try again later.".to_string(),
                };
                let html = template.to_string();
                Ok(Html(html).into_response())
            }
        }
    } else {
        // Fallback behavior if login service is not available
        error!("Login service not available - database not connected");
        let template = LoginFormTemplate {
            general_error: "Authentication service unavailable".to_string(),
        };
        let html = template.to_string();
        Ok(Html(html).into_response())
    }
}

/// Render the login form (GET request) - old simple version
pub async fn login_form() -> Result<Response, StatusCode> {
    let template = LoginFormTemplate {
        general_error: String::new(),
    };
    let html = template.to_string();
    Ok(Html(html).into_response())
}

/// Render the enhanced login page (GET request)
pub async fn login_page(Query(query): Query<LoginQuery>) -> Result<Response, StatusCode> {
    let success_message = match query.success.as_deref() {
        Some("registered") => "Registration successful! You can now log in.".to_string(),
        _ => String::new(),
    };

    let template = LoginPageTemplate {
        general_error: String::new(),
        success_message,
    };
    let html = template.to_string();
    Ok(Html(html).into_response())
}

/// Render the registration page (GET request)
pub async fn register_page() -> Result<Response, StatusCode> {
    let template = RegisterPageTemplate {
        general_error: String::new(),
    };
    let html = template.to_string();
    Ok(Html(html).into_response())
}

/// Handle registration form submission (POST request)
pub async fn register_handler(
    State(app_state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Result<Response, StatusCode> {
    info!("Processing registration for email: {}", form.email);

    // Validate the form
    if let Err(validation_errors) = form.validate() {
        let mut error_map = std::collections::HashMap::new();
        for (field, errors) in validation_errors.field_errors() {
            let error_messages: Vec<String> = errors
                .iter()
                .map(|e| {
                    e.message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "Invalid input".to_string())
                })
                .collect();
            error_map.insert(field.to_string(), error_messages);
        }

        let template = RegisterPageTemplate {
            general_error: "Please correct the errors below".to_string(),
        };
        let html = template.to_string();
        return Ok(Html(html).into_response());
    }

    // Check password confirmation
    if let Err(password_error) = form.validate_passwords_match() {
        let template = RegisterPageTemplate {
            general_error: password_error,
        };
        let html = template.to_string();
        return Ok(Html(html).into_response());
    }

    // Parse email and password using domain value objects
    let email = match Email::new(form.email) {
        Ok(email) => email,
        Err(e) => {
            error!("Invalid email format: {:?}", e);
            let template = RegisterPageTemplate {
                general_error: "Invalid email format".to_string(),
            };
            let html = template.to_string();
            return Ok(Html(html).into_response());
        }
    };

    let password = match Password::new(form.password) {
        Ok(password) => password,
        Err(e) => {
            error!("Invalid password: {:?}", e);
            let template = RegisterPageTemplate {
                general_error: "Password does not meet requirements".to_string(),
            };
            let html = template.to_string();
            return Ok(Html(html).into_response());
        }
    };

    // Use RegisterUserService if available
    if let Some(ref register_service) = app_state.register_service {
        // Create registration command with default values for required fields
        let register_command = RegisterUserCommand::new(
            email.clone(),
            password,
            FamilySize::FAMILY2,  // Default family size
            SkillLevel::Beginner, // Default skill level
        );

        match register_service.handle(register_command).await {
            Ok(_register_response) => {
                info!("Registration successful for user: {}", email.value);

                // TODO: Create session cookie for the newly registered user
                // For now, redirect to login page with success message
                // let session_cookie = create_session_cookie(register_response.user_id);
                // cookies.add(session_cookie);

                // Use ts-location header to redirect to login page with success message
                let mut headers = HeaderMap::new();
                headers.insert(
                    "ts-location",
                    HeaderValue::from_static("/auth/login?success=registered"),
                );

                let mut response = Html("").into_response();
                *response.headers_mut() = headers;
                Ok(response)
            }
            Err(RegisterUserError::EmailAlreadyExists) => {
                error!(
                    "Registration failed - email already exists: {}",
                    email.value
                );
                let template = RegisterPageTemplate {
                    general_error: "An account with this email already exists".to_string(),
                };
                let html = template.to_string();
                Ok(Html(html).into_response())
            }
            Err(e) => {
                error!("Registration error: {:?}", e);
                let template = RegisterPageTemplate {
                    general_error: "Registration failed. Please try again later.".to_string(),
                };
                let html = template.to_string();
                Ok(Html(html).into_response())
            }
        }
    } else {
        // Fallback behavior if registration service is not available
        error!("Registration service not available - database not connected");
        let template = RegisterPageTemplate {
            general_error: "Registration service unavailable".to_string(),
        };
        let html = template.to_string();
        Ok(Html(html).into_response())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_form_validation() {
        let valid_form = LoginForm {
            email: "user@example.com".to_string(),
            password: "ValidPass123!".to_string(),
        };
        assert!(valid_form.validate().is_ok());

        let invalid_email = LoginForm {
            email: "not-an-email".to_string(),
            password: "ValidPass123!".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        let short_password = LoginForm {
            email: "user@example.com".to_string(),
            password: "short".to_string(),
        };
        assert!(short_password.validate().is_err());
    }
}
