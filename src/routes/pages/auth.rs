use axum::{
    extract::{Extension, Form, State},
    http::StatusCode,
    response::{Html, Json, Redirect, Response, IntoResponse},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    models::user::{AuthResponse, CreateUserRequest, LoginRequest, UserClaims, UserProfile},
    services::{AuthError, AuthService},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub field: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub message: String,
}

// Form structs for HTML forms
#[derive(Debug, Deserialize)]
pub struct RegistrationForm {
    pub email: String,
    pub password: String,
    pub confirm_password: String,
    pub skill_level: Option<String>,
    pub household_size: Option<String>,
    pub dietary_preferences: Option<String>, // comma-separated
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

pub fn auth_routes() -> Router<AuthService> {
    Router::new()
        // Public page routes
        .route("/register", get(show_register_page))
        .route("/login", get(show_login_page))
        .route("/verify/:token", get(verify_email_handler))
        // Public form handling routes
        .route("/register", post(handle_registration))
        .route("/login", post(handle_login))
        // Public API routes
        .route("/api/register", post(api_register))
        .route("/api/login", post(api_login))
        .route("/api/verify/:token", post(api_verify_email))
        // Protected routes - will be properly protected when integrated with main app
        .route("/profile", get(show_profile_page))
        .route("/logout", post(handle_logout))
        .route("/api/logout", post(api_logout))
        .route("/api/profile", get(api_get_profile))
}

// Page handlers
pub async fn show_register_page() -> Html<&'static str> {
    Html(include_str!("../../../templates/pages/auth/register.html"))
}

pub async fn show_login_page() -> Html<&'static str> {
    Html(include_str!("../../../templates/pages/auth/login.html"))
}

pub async fn show_profile_page(
    user_claims: Option<Extension<UserClaims>>,
) -> Result<Html<String>, StatusCode> {
    match user_claims {
        Some(Extension(claims)) => {
            let html = format!(
                r#"
                <html>
                <head><title>Profile - ImKitchen</title></head>
                <body>
                    <h1>Profile</h1>
                    <p>User ID: {}</p>
                    <p>Email: {}</p>
                    <a href="/auth/logout">Logout</a>
                </body>
                </html>
                "#,
                claims.sub, claims.email
            );
            Ok(Html(html))
        }
        None => {
            let html = r#"
                <html>
                <head><title>Unauthorized - ImKitchen</title></head>
                <body>
                    <h1>Unauthorized</h1>
                    <p>Please <a href="/auth/login">login</a> to view your profile.</p>
                </body>
                </html>
            "#;
            Ok(Html(html.to_string()))
        }
    }
}

// Form handlers
pub async fn handle_registration(
    State(auth_service): State<AuthService>,
    Form(form): Form<RegistrationForm>,
) -> Result<Response, StatusCode> {
    // Validate form
    let mut errors = Vec::new();

    if form.email.is_empty() {
        errors.push("Email is required");
    }
    if form.password.is_empty() {
        errors.push("Password is required");
    }
    if form.password != form.confirm_password {
        errors.push("Passwords do not match");
    }
    if form.password.len() < 8 {
        errors.push("Password must be at least 8 characters long");
    }

    if !errors.is_empty() {
        // Return back to registration form with errors
        let error_html = format!(
            r#"
            <html>
            <head><title>Registration Error - ImKitchen</title></head>
            <body>
                <h1>Registration Error</h1>
                <ul>
                    {}
                </ul>
                <a href="/auth/register">Back to Registration</a>
            </body>
            </html>
            "#,
            errors.into_iter().map(|e| format!("<li>{}</li>", e)).collect::<Vec<_>>().join("")
        );
        return Ok(Html(error_html).into_response());
    }

    // Parse optional fields
    let skill_level = form.skill_level.as_deref().and_then(|s| s.parse().ok());
    let household_size = form.household_size.as_deref().and_then(|s| s.parse().ok());
    let dietary_preferences = form.dietary_preferences
        .map(|s| s.split(',').map(|s| s.trim().to_string()).collect());

    let request = CreateUserRequest {
        email: form.email,
        password: form.password,
        skill_level,
        household_size,
        dietary_preferences,
        kitchen_equipment: None,
    };

    match auth_service.register_user(request).await {
        Ok(_user) => {
            // Redirect to login with success message
            Ok(Redirect::to("/auth/login?message=Registration successful! Please check your email to verify your account.").into_response())
        }
        Err(AuthError::UserAlreadyExists) => {
            let error_html = r#"
                <html>
                <head><title>Registration Error - ImKitchen</title></head>
                <body>
                    <h1>Registration Error</h1>
                    <p>An account with this email already exists.</p>
                    <a href="/auth/login">Login</a> | <a href="/auth/register">Try again</a>
                </body>
                </html>
            "#;
            Ok(Html(error_html).into_response())
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_login(
    State(auth_service): State<AuthService>,
    Form(form): Form<LoginForm>,
) -> Result<Response, StatusCode> {
    // Validate form
    if form.email.is_empty() || form.password.is_empty() {
        let error_html = r#"
            <html>
            <head><title>Login Error - ImKitchen</title></head>
            <body>
                <h1>Login Error</h1>
                <p>Email and password are required.</p>
                <a href="/auth/login">Try again</a>
            </body>
            </html>
        "#;
        return Ok(Html(error_html).into_response());
    }

    let request = LoginRequest {
        email: form.email,
        password: form.password,
    };

    match auth_service.authenticate_user(request).await {
        Ok(_auth_response) => {
            // In a real implementation, you'd set an HTTP-only cookie here
            // For now, redirect to profile
            Ok(Redirect::to("/auth/profile").into_response())
        }
        Err(AuthError::InvalidCredentials) => {
            let error_html = r#"
                <html>
                <head><title>Login Error - ImKitchen</title></head>
                <body>
                    <h1>Login Error</h1>
                    <p>Invalid email or password.</p>
                    <a href="/auth/login">Try again</a>
                </body>
                </html>
            "#;
            Ok(Html(error_html).into_response())
        }
        Err(AuthError::EmailNotVerified) => {
            let error_html = r#"
                <html>
                <head><title>Email Not Verified - ImKitchen</title></head>
                <body>
                    <h1>Email Not Verified</h1>
                    <p>Please check your email and click the verification link.</p>
                    <a href="/auth/login">Back to Login</a>
                </body>
                </html>
            "#;
            Ok(Html(error_html).into_response())
        }
        Err(AuthError::RateLimitExceeded) => {
            let error_html = r#"
                <html>
                <head><title>Too Many Attempts - ImKitchen</title></head>
                <body>
                    <h1>Too Many Attempts</h1>
                    <p>Too many failed login attempts. Please try again later.</p>
                    <a href="/auth/login">Back to Login</a>
                </body>
                </html>
            "#;
            Ok(Html(error_html).into_response())
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn handle_logout(
    State(auth_service): State<AuthService>,
    user_claims: Option<Extension<UserClaims>>,
) -> Result<Redirect, StatusCode> {
    if let Some(Extension(claims)) = user_claims {
        // In a real implementation, you'd extract the token from cookies
        // For now, we'll use a placeholder token
        let _ = auth_service.logout(claims.sub, "placeholder_token").await;
    }
    Ok(Redirect::to("/auth/login?message=Successfully logged out"))
}

pub async fn verify_email_handler(
    State(auth_service): State<AuthService>,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> Result<Response, StatusCode> {
    match auth_service.verify_email(&token).await {
        Ok(_) => {
            let success_html = r#"
                <html>
                <head><title>Email Verified - ImKitchen</title></head>
                <body>
                    <h1>Email Verified</h1>
                    <p>Your email has been successfully verified!</p>
                    <a href="/auth/login">Login</a>
                </body>
                </html>
            "#;
            Ok(Html(success_html).into_response())
        }
        Err(AuthError::InvalidVerificationToken) => {
            let error_html = r#"
                <html>
                <head><title>Invalid Token - ImKitchen</title></head>
                <body>
                    <h1>Invalid Verification Token</h1>
                    <p>The verification token is invalid or has expired.</p>
                    <a href="/auth/register">Register Again</a>
                </body>
                </html>
            "#;
            Ok(Html(error_html).into_response())
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// API handlers
pub async fn api_register(
    State(auth_service): State<AuthService>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserProfile>, (StatusCode, Json<ErrorResponse>)> {
    match auth_service.register_user(request).await {
        Ok(user) => Ok(Json(user)),
        Err(AuthError::UserAlreadyExists) => Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "User already exists".to_string(),
                field: Some("email".to_string()),
            }),
        )),
        Err(AuthError::InvalidInput(msg)) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: msg,
                field: None,
            }),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Internal server error".to_string(),
                field: None,
            }),
        )),
    }
}

pub async fn api_login(
    State(auth_service): State<AuthService>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    match auth_service.authenticate_user(request).await {
        Ok(auth_response) => Ok(Json(auth_response)),
        Err(AuthError::InvalidCredentials) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid credentials".to_string(),
                field: None,
            }),
        )),
        Err(AuthError::EmailNotVerified) => Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "Email not verified".to_string(),
                field: Some("email".to_string()),
            }),
        )),
        Err(AuthError::RateLimitExceeded) => Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrorResponse {
                error: "Rate limit exceeded".to_string(),
                field: None,
            }),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Internal server error".to_string(),
                field: None,
            }),
        )),
    }
}

pub async fn api_logout(
    State(auth_service): State<AuthService>,
    user_claims: Option<Extension<UserClaims>>,
) -> Result<Json<SuccessResponse>, StatusCode> {
    if let Some(Extension(claims)) = user_claims {
        // In a real implementation, you'd extract the token from the Authorization header
        let _ = auth_service.logout(claims.sub, "placeholder_token").await;
    }
    Ok(Json(SuccessResponse {
        message: "Successfully logged out".to_string(),
    }))
}

pub async fn api_get_profile(
    user_claims: Option<Extension<UserClaims>>,
) -> Result<Json<UserClaims>, StatusCode> {
    match user_claims {
        Some(Extension(claims)) => Ok(Json(claims)),
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn api_verify_email(
    State(auth_service): State<AuthService>,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> Result<Json<UserProfile>, (StatusCode, Json<ErrorResponse>)> {
    match auth_service.verify_email(&token).await {
        Ok(user) => Ok(Json(user)),
        Err(AuthError::InvalidVerificationToken) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid verification token".to_string(),
                field: None,
            }),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Internal server error".to_string(),
                field: None,
            }),
        )),
    }
}