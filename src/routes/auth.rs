use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use user::{
    generate_jwt, generate_reset_token, query_user_by_email, query_user_for_login,
    register_user, reset_password, validate_jwt, verify_password, RegisterUserCommand,
    ResetPasswordCommand, UserError,
};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub evento_executor: evento::Sqlite,
    pub jwt_secret: String,
    pub email_config: crate::email::EmailConfig,
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub email: String,
    pub password: String,
    pub password_confirm: String,
}

#[derive(Template)]
#[template(path = "pages/register.html")]
pub struct RegisterPageTemplate {
    pub error: String,
    pub user: Option<()>, // None for public pages
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Template)]
#[template(path = "pages/login.html")]
pub struct LoginPageTemplate {
    pub error: String,
    pub user: Option<()>, // None for public pages
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequestForm {
    pub email: String,
}

#[derive(Template)]
#[template(path = "pages/password-reset-request.html")]
pub struct PasswordResetRequestTemplate {
    pub error: String,
    pub success: bool,
    pub email: String,
    pub user: Option<()>, // None for public pages
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetCompleteForm {
    pub new_password: String,
    pub password_confirm: String,
}

#[derive(Template)]
#[template(path = "pages/password-reset-complete.html")]
pub struct PasswordResetCompleteTemplate {
    pub error: String,
    pub token: String,
    pub user: Option<()>, // None for public pages
}

/// GET /register - Display registration form
#[tracing::instrument]
pub async fn get_register() -> impl IntoResponse {
    let template = RegisterPageTemplate {
        error: String::new(),
        user: None,
    };
    Html(template.render().unwrap())
}

/// POST /register - Handle registration form submission
#[tracing::instrument(skip(state, form), fields(email = %form.email))]
pub async fn post_register(
    State(state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Response {
    // Validate password confirmation
    if form.password != form.password_confirm {
        let template = RegisterPageTemplate {
            error: "Passwords do not match".to_string(),
            user: None,
        };
        // Return 200 OK for TwinSpark form swap (progressive enhancement)
        return Html(template.render().unwrap()).into_response();
    }

    // Create command
    let command = RegisterUserCommand {
        email: form.email.clone(),
        password: form.password,
    };

    // Execute registration (evento event sourcing)
    match register_user(command, &state.evento_executor, &state.db_pool).await {
        Ok(aggregator_id) => {
            // Generate JWT token
            let token = match generate_jwt(
                aggregator_id,
                form.email,
                "free".to_string(),
                &state.jwt_secret,
            ) {
                Ok(t) => t,
                Err(e) => {
                    tracing::error!("Failed to generate JWT token: {:?}", e);
                    let template = RegisterPageTemplate {
                        error: "Registration succeeded but failed to generate session token. Please try logging in.".to_string(),
                        user: None,
                    };
                    // Return 200 OK for TwinSpark form swap (progressive enhancement)
                    return Html(template.render().unwrap()).into_response();
                }
            };

            // Set HTTP-only secure cookie with CSRF protection
            let cookie = format!(
                "auth_token={}; HttpOnly; Secure; SameSite=Lax; Max-Age={}; Path=/",
                token,
                7 * 24 * 60 * 60 // 7 days in seconds
            );

            // Redirect to dashboard using TwinSpark (progressive enhancement)
            // Returns 200 OK for proper form swap, ts-location triggers client-side navigation
            (
                StatusCode::OK,
                [
                    ("Set-Cookie", cookie.as_str()),
                    ("ts-location", "/dashboard"),
                ],
                (),
            )
                .into_response()
        }
        Err(e) => {
            let error_message = match e {
                UserError::EmailAlreadyExists => "Email already registered".to_string(),
                UserError::InvalidEmail => "Invalid email format".to_string(),
                UserError::PasswordTooShort => "Password must be at least 8 characters".to_string(),
                UserError::ValidationError(msg) => msg,
                _ => {
                    tracing::error!("Registration error: {:?}", e);
                    "Registration failed".to_string()
                }
            };

            // Return 200 OK for TwinSpark form swap (progressive enhancement)
            let template = RegisterPageTemplate {
                error: error_message,
                user: None,
            };

            Html(template.render().unwrap()).into_response()
        }
    }
}

/// GET /login - Display login form
#[tracing::instrument]
pub async fn get_login() -> impl IntoResponse {
    let template = LoginPageTemplate {
        error: String::new(),
        user: None,
    };
    Html(template.render().unwrap())
}

/// POST /login - Handle login form submission
#[tracing::instrument(skip(state, form), fields(email = %form.email))]
pub async fn post_login(State(state): State<AppState>, Form(form): Form<LoginForm>) -> Response {
    // Query user by email from read model
    let user = match query_user_for_login(&form.email, &state.db_pool).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            // User not found - return generic error (AC: 4 - no user enumeration)
            tracing::warn!("Failed login attempt for email: {}", form.email);
            let template = LoginPageTemplate {
                error: "Invalid credentials".to_string(),
                user: None,
            };
            return Html(template.render().unwrap()).into_response();
        }
        Err(e) => {
            tracing::error!("Database error during login: {:?}", e);
            let template = LoginPageTemplate {
                error: "An error occurred. Please try again.".to_string(),
                user: None,
            };
            return Html(template.render().unwrap()).into_response();
        }
    };

    // Verify password (AC: 2)
    let password_valid = match verify_password(&form.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(e) => {
            tracing::error!("Password verification error: {:?}", e);
            let template = LoginPageTemplate {
                error: "An error occurred. Please try again.".to_string(),
                user: None,
            };
            return Html(template.render().unwrap()).into_response();
        }
    };

    if !password_valid {
        // Password incorrect - return generic error (AC: 4 - no user enumeration)
        tracing::warn!(
            "Failed login attempt (incorrect password) for email: {}",
            form.email
        );
        let template = LoginPageTemplate {
            error: "Invalid credentials".to_string(),
            user: None,
        };
        return Html(template.render().unwrap()).into_response();
    }

    // Generate JWT token (AC: 3, 7)
    let token = match generate_jwt(user.id, user.email, user.tier, &state.jwt_secret) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to generate JWT token: {:?}", e);
            let template = LoginPageTemplate {
                error: "Login succeeded but failed to generate session token. Please try again."
                    .to_string(),
                user: None,
            };
            return Html(template.render().unwrap()).into_response();
        }
    };

    // Set HTTP-only secure cookie with CSRF protection (AC: 3, 6, 8)
    let cookie = format!(
        "auth_token={}; HttpOnly; Secure; SameSite=Lax; Max-Age={}; Path=/",
        token,
        7 * 24 * 60 * 60 // 7 days in seconds (AC: 8)
    );

    // Redirect to dashboard using TwinSpark (progressive enhancement) (AC: 5)
    // Returns 200 OK for proper form swap, ts-location triggers client-side navigation
    (
        StatusCode::OK,
        [
            ("Set-Cookie", cookie.as_str()),
            ("ts-location", "/dashboard"),
        ],
        (),
    )
        .into_response()
}

/// GET /password-reset - Display password reset request form
#[tracing::instrument]
pub async fn get_password_reset() -> impl IntoResponse {
    let template = PasswordResetRequestTemplate {
        error: String::new(),
        success: false,
        email: String::new(),
        user: None,
    };
    Html(template.render().unwrap())
}

/// POST /password-reset - Handle password reset request form submission
#[tracing::instrument(skip(state, form), fields(email = %form.email))]
pub async fn post_password_reset(
    State(state): State<AppState>,
    Form(form): Form<PasswordResetRequestForm>,
) -> Response {
    tracing::info!("Password reset requested for email: {}", form.email);

    // Query user by email (AC: 2)
    let user_id_opt = match query_user_by_email(&form.email, &state.db_pool).await {
        Ok(id_opt) => id_opt,
        Err(e) => {
            tracing::error!("Database error during password reset query: {:?}", e);
            // Return success to prevent user enumeration (AC: 4)
            let template = PasswordResetRequestTemplate {
                error: String::new(),
                success: true,
                email: form.email.clone(),
                user: None,
            };
            return Html(template.render().unwrap()).into_response();
        }
    };

    // Always return success message to prevent user enumeration (AC: 4)
    if let Some(user_id) = user_id_opt {
        // User exists - generate reset token and send email
        match generate_reset_token(user_id, form.email.clone(), &state.jwt_secret) {
            Ok(reset_token) => {
                // Send password reset email (AC: 3)
                if let Err(e) = crate::email::send_password_reset_email(
                    &form.email,
                    &reset_token,
                    &state.email_config,
                    &state.base_url,
                )
                .await
                {
                    tracing::warn!(
                        "Failed to send password reset email to {}: {:?}",
                        form.email,
                        e
                    );
                }
                tracing::info!("Password reset email sent successfully to {}", form.email);
            }
            Err(e) => {
                tracing::error!("Failed to generate reset token: {:?}", e);
            }
        }
    } else {
        // User doesn't exist - log but return success (prevent enumeration)
        tracing::warn!(
            "Password reset requested for non-existent email: {}",
            form.email
        );
    }

    // Always show success message (AC: 4)
    let template = PasswordResetRequestTemplate {
        error: String::new(),
        success: true,
        email: form.email,
        user: None,
    };
    Html(template.render().unwrap()).into_response()
}

/// GET /password-reset/:token - Display password reset completion form
#[tracing::instrument(skip(state))]
pub async fn get_password_reset_complete(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Response {
    // Validate token before rendering form (AC: 4, 8)
    match validate_jwt(&token, &state.jwt_secret) {
        Ok(claims) => {
            // Check if token type is "reset"
            if claims.tier != "reset" {
                tracing::warn!("Invalid token type for password reset: {}", claims.tier);
                return Html("<html><body><h1>Invalid Reset Token</h1><p>This password reset link is invalid. Please request a new password reset.</p><a href=\"/password-reset\">Request Password Reset</a></body></html>").into_response();
            }

            // Token valid - render form
            let template = PasswordResetCompleteTemplate {
                error: String::new(),
                token,
                user: None,
            };
            Html(template.render().unwrap()).into_response()
        }
        Err(e) => {
            // Token invalid or expired (AC: 8)
            tracing::warn!("Invalid or expired password reset token: {:?}", e);
            Html("<html><body><h1>Invalid or Expired Reset Token</h1><p>This password reset link has expired or is invalid. Reset tokens are valid for 1 hour. Please request a new password reset.</p><a href=\"/password-reset\">Request Password Reset</a></body></html>").into_response()
        }
    }
}

/// POST /password-reset/:token - Handle password reset completion
#[tracing::instrument(skip(state, form, token))]
pub async fn post_password_reset_complete(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Form(form): Form<PasswordResetCompleteForm>,
) -> Response {
    // Validate token (AC: 4, 8)
    let claims = match validate_jwt(&token, &state.jwt_secret) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Invalid or expired token during password reset completion: {:?}", e);
            return Html("<html><body><h1>Invalid or Expired Reset Token</h1><p>This password reset link has expired or is invalid. Please request a new password reset.</p><a href=\"/password-reset\">Request Password Reset</a></body></html>").into_response();
        }
    };

    // Verify token type
    if claims.tier != "reset" {
        tracing::warn!("Invalid token type for password reset: {}", claims.tier);
        return Html("<html><body><h1>Invalid Reset Token</h1><p>This password reset link is invalid. Please request a new password reset.</p><a href=\"/password-reset\">Request Password Reset</a></body></html>").into_response();
    }

    // Validate password length (AC: 5)
    if form.new_password.len() < 8 {
        let template = PasswordResetCompleteTemplate {
            error: "Password must be at least 8 characters long".to_string(),
            token,
            user: None,
        };
        return Html(template.render().unwrap()).into_response();
    }

    // Validate password confirmation matches (AC: 5)
    if form.new_password != form.password_confirm {
        let template = PasswordResetCompleteTemplate {
            error: "Passwords do not match".to_string(),
            token,
            user: None,
        };
        return Html(template.render().unwrap()).into_response();
    }

    // Reset password using evento command (AC: 6)
    // This creates a PasswordChanged event which is automatically projected to read model
    let command = ResetPasswordCommand {
        user_id: claims.sub.clone(),
        new_password: form.new_password.clone(),
    };

    match reset_password(command, &state.evento_executor).await {
        Ok(_) => {
            // Password successfully reset and PasswordChanged event committed (AC: 6, 7)
            tracing::info!("Password reset successful for user: {}", claims.sub);

            // Redirect to login page with success message (AC: 7)
            (
                StatusCode::SEE_OTHER,
                [("Location", "/login?reset_success=true")],
                (),
            )
                .into_response()
        }
        Err(e) => {
            // Password reset failed
            tracing::error!("Failed to reset password: {:?}", e);
            let template = PasswordResetCompleteTemplate {
                error: "An error occurred. Please try again.".to_string(),
                token,
                user: None,
            };
            Html(template.render().unwrap()).into_response()
        }
    }
}
