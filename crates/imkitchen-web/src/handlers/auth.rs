use axum::{
    extract::{ConnectInfo, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use imkitchen_core::services::{AuthError, AuthService};
use imkitchen_shared::auth_types::*;
use serde::Deserialize;
use std::net::SocketAddr;
use tower_cookies::cookie::time::OffsetDateTime;
use tower_cookies::{Cookie, Cookies};
use tracing::{error, info, warn};

/// Registration endpoint
pub async fn register_handler(
    State(shared_state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let ip_address = addr.ip().to_string();

    info!("Registration attempt for email: {}", request.email);

    // Get database pool from shared state
    let app_state = shared_state.read().await;
    let db_pool = app_state
        .db
        .as_ref()
        .expect("Database must be initialized")
        .clone();
    drop(app_state);

    let auth_service = AuthService::new(db_pool);

    // Convert RegisterRequest to CreateUserRequest
    let create_request = imkitchen_core::models::user::CreateUserRequest {
        email: request.email,
        password: request.password,
        name: request.name,
        family_size: request.family_size,
    };

    match auth_service.register_user(create_request, ip_address).await {
        Ok(auth_response) => {
            info!("Registration successful");

            // Convert core types to shared types
            let user_public = UserPublic {
                id: auth_response.user.id,
                email: auth_response.user.email,
                name: auth_response.user.name,
                family_size: auth_response.user.family_size,
                dietary_restrictions: auth_response.user.dietary_restrictions,
                cooking_skill_level: match auth_response.user.cooking_skill_level {
                    imkitchen_core::models::user::CookingSkillLevel::Beginner => {
                        CookingSkillLevel::Beginner
                    }
                    imkitchen_core::models::user::CookingSkillLevel::Intermediate => {
                        CookingSkillLevel::Intermediate
                    }
                    imkitchen_core::models::user::CookingSkillLevel::Advanced => {
                        CookingSkillLevel::Advanced
                    }
                },
                email_verified: auth_response.user.email_verified,
                created_at: auth_response.user.created_at.to_rfc3339(),
                last_active: auth_response.user.last_active.to_rfc3339(),
            };

            let response = AuthResponse {
                success: true,
                message: auth_response.message,
                user: Some(user_public),
            };

            Ok(Json(response))
        }
        Err(e) => {
            warn!("Registration failed: {}", e);
            let (status, error_response) = match e {
                AuthError::EmailExists => (
                    StatusCode::CONFLICT,
                    ErrorResponse::with_code(
                        "Email already exists".to_string(),
                        "EMAIL_EXISTS".to_string(),
                    ),
                ),
                AuthError::Password(password_error) => (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::with_code(
                        format!("Password validation failed: {}", password_error),
                        "PASSWORD_VALIDATION_FAILED".to_string(),
                    ),
                ),
                AuthError::RateLimitExceeded => (
                    StatusCode::TOO_MANY_REQUESTS,
                    ErrorResponse::with_code(
                        "Rate limit exceeded. Please try again later.".to_string(),
                        "RATE_LIMIT_EXCEEDED".to_string(),
                    ),
                ),
                _ => {
                    error!("Internal error during registration: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse::new("Internal server error".to_string()),
                    )
                }
            };
            Err((status, Json(error_response)))
        }
    }
}

/// Login endpoint
#[axum::debug_handler]
pub async fn login_handler(
    State(shared_state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    cookies: Cookies,
    Json(login_request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let ip_address = addr.ip().to_string();

    info!("Login attempt for email: {}", login_request.email);

    // Get database pool from shared state
    let app_state = shared_state.read().await;
    let db_pool = app_state
        .db
        .as_ref()
        .expect("Database must be initialized")
        .clone();
    drop(app_state);

    let auth_service = AuthService::new(db_pool);

    // Convert shared LoginRequest to core LoginRequest
    let core_request = imkitchen_core::models::user::LoginRequest {
        email: login_request.email,
        password: login_request.password,
    };

    match auth_service.login_user(core_request, ip_address).await {
        Ok((auth_response, session)) => {
            info!("Login successful");

            // Set session cookie
            let mut cookie = Cookie::new("session_id", session.session_token);
            cookie.set_http_only(true);
            cookie.set_secure(true); // HTTPS only in production
            cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
            cookie.set_path("/");
            let expires_offset =
                OffsetDateTime::from_unix_timestamp(session.expires_at.timestamp()).unwrap();
            cookie.set_expires(Some(expires_offset.into()));
            cookies.add(cookie);

            // Convert core types to shared types
            let user_public = UserPublic {
                id: auth_response.user.id,
                email: auth_response.user.email,
                name: auth_response.user.name,
                family_size: auth_response.user.family_size,
                dietary_restrictions: auth_response.user.dietary_restrictions,
                cooking_skill_level: match auth_response.user.cooking_skill_level {
                    imkitchen_core::models::user::CookingSkillLevel::Beginner => {
                        CookingSkillLevel::Beginner
                    }
                    imkitchen_core::models::user::CookingSkillLevel::Intermediate => {
                        CookingSkillLevel::Intermediate
                    }
                    imkitchen_core::models::user::CookingSkillLevel::Advanced => {
                        CookingSkillLevel::Advanced
                    }
                },
                email_verified: auth_response.user.email_verified,
                created_at: auth_response.user.created_at.to_rfc3339(),
                last_active: auth_response.user.last_active.to_rfc3339(),
            };

            let response = AuthResponse {
                success: true,
                message: auth_response.message,
                user: Some(user_public),
            };

            Ok(Json(response))
        }
        Err(e) => {
            warn!("Login failed: {}", e);
            let (status, error_response) = match e {
                AuthError::InvalidCredentials => (
                    StatusCode::UNAUTHORIZED,
                    ErrorResponse::with_code(
                        "Invalid email or password".to_string(),
                        "INVALID_CREDENTIALS".to_string(),
                    ),
                ),
                AuthError::EmailNotVerified => (
                    StatusCode::FORBIDDEN,
                    ErrorResponse::with_code(
                        "Please verify your email before logging in".to_string(),
                        "EMAIL_NOT_VERIFIED".to_string(),
                    ),
                ),
                AuthError::RateLimitExceeded => (
                    StatusCode::TOO_MANY_REQUESTS,
                    ErrorResponse::with_code(
                        "Too many login attempts. Please try again later.".to_string(),
                        "RATE_LIMIT_EXCEEDED".to_string(),
                    ),
                ),
                _ => {
                    error!("Internal error during login: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse::new("Internal server error".to_string()),
                    )
                }
            };
            Err((status, Json(error_response)))
        }
    }
}

/// Email verification endpoint
#[derive(Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

pub async fn verify_email_handler(
    State(shared_state): State<SharedState>,
    Query(query): Query<VerifyEmailQuery>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Email verification attempt");

    // Get database pool from shared state
    let app_state = shared_state.read().await;
    let db_pool = app_state
        .db
        .as_ref()
        .expect("Database must be initialized")
        .clone();
    drop(app_state);

    let auth_service = AuthService::new(db_pool);

    match auth_service.verify_email(&query.token).await {
        Ok(success) => {
            if success {
                info!("Email verification successful");
                let response = AuthResponse {
                    success: true,
                    message: "Email verified successfully".to_string(),
                    user: None,
                };
                Ok(Json(response))
            } else {
                warn!("Email verification failed - invalid or expired token");
                let error_response = ErrorResponse::with_code(
                    "Invalid or expired verification token".to_string(),
                    "INVALID_VERIFICATION_TOKEN".to_string(),
                );
                Err((StatusCode::BAD_REQUEST, Json(error_response)))
            }
        }
        Err(e) => {
            error!("Internal error during email verification: {}", e);
            let error_response = ErrorResponse::new("Internal server error".to_string());
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Password reset request endpoint
pub async fn request_password_reset_handler(
    State(shared_state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(request): Json<PasswordResetRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let ip_address = addr.ip().to_string();

    info!("Password reset request for email: {}", request.email);

    // Get database pool from shared state
    let app_state = shared_state.read().await;
    let db_pool = app_state
        .db
        .as_ref()
        .expect("Database must be initialized")
        .clone();
    drop(app_state);

    let auth_service = AuthService::new(db_pool);

    match auth_service
        .request_password_reset(&request.email, ip_address)
        .await
    {
        Ok(message) => {
            info!("Password reset request processed");
            let response = AuthResponse {
                success: true,
                message,
                user: None,
            };
            Ok(Json(response))
        }
        Err(e) => {
            warn!("Password reset request failed: {}", e);
            let (status, error_response) = match e {
                AuthError::RateLimitExceeded => (
                    StatusCode::TOO_MANY_REQUESTS,
                    ErrorResponse::with_code(
                        "Rate limit exceeded. Please try again later.".to_string(),
                        "RATE_LIMIT_EXCEEDED".to_string(),
                    ),
                ),
                _ => {
                    error!("Internal error during password reset request: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse::new("Internal server error".to_string()),
                    )
                }
            };
            Err((status, Json(error_response)))
        }
    }
}

/// Password reset confirmation endpoint
pub async fn reset_password_handler(
    State(shared_state): State<SharedState>,
    Json(request): Json<PasswordResetConfirmRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Password reset confirmation attempt");

    // Get database pool from shared state
    let app_state = shared_state.read().await;
    let db_pool = app_state
        .db
        .as_ref()
        .expect("Database must be initialized")
        .clone();
    drop(app_state);

    let auth_service = AuthService::new(db_pool);

    match auth_service
        .reset_password(&request.token, &request.new_password)
        .await
    {
        Ok(success) => {
            if success {
                info!("Password reset successful");
                let response = AuthResponse {
                    success: true,
                    message: "Password reset successfully".to_string(),
                    user: None,
                };
                Ok(Json(response))
            } else {
                warn!("Password reset failed - invalid or expired token");
                let error_response = ErrorResponse::with_code(
                    "Invalid or expired reset token".to_string(),
                    "INVALID_RESET_TOKEN".to_string(),
                );
                Err((StatusCode::BAD_REQUEST, Json(error_response)))
            }
        }
        Err(e) => {
            warn!("Password reset failed: {}", e);
            let (status, error_response) = match e {
                AuthError::Password(password_error) => (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::with_code(
                        format!("Password validation failed: {}", password_error),
                        "PASSWORD_VALIDATION_FAILED".to_string(),
                    ),
                ),
                _ => {
                    error!("Internal error during password reset: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ErrorResponse::new("Internal server error".to_string()),
                    )
                }
            };
            Err((status, Json(error_response)))
        }
    }
}

/// Logout endpoint  
#[axum::debug_handler]
pub async fn logout_handler(
    State(shared_state): State<SharedState>,
    cookies: Cookies,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("Logout attempt");

    if let Some(session_cookie) = cookies.get("session_id") {
        let session_token = session_cookie.value();

        // Get database pool from shared state
        let app_state = shared_state.read().await;
        let db_pool = app_state
            .db
            .as_ref()
            .expect("Database must be initialized")
            .clone();
        drop(app_state);

        let auth_service = AuthService::new(db_pool);

        match auth_service.logout_user(session_token).await {
            Ok(()) => {
                info!("Logout successful");

                // Clear session cookie
                let mut cookie = Cookie::new("session_id", "");
                cookie.set_http_only(true);
                cookie.set_secure(true);
                cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
                cookie.set_path("/");
                let past_time = OffsetDateTime::from_unix_timestamp(
                    (chrono::Utc::now() - chrono::Duration::days(1)).timestamp(),
                )
                .unwrap();
                cookie.set_expires(Some(past_time.into()));
                cookies.add(cookie);

                let response = AuthResponse {
                    success: true,
                    message: "Logged out successfully".to_string(),
                    user: None,
                };
                Ok(Json(response))
            }
            Err(e) => {
                error!("Internal error during logout: {}", e);
                let error_response = ErrorResponse::new("Internal server error".to_string());
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
            }
        }
    } else {
        warn!("Logout attempt without session cookie");
        let error_response = ErrorResponse::with_code(
            "No active session found".to_string(),
            "NO_ACTIVE_SESSION".to_string(),
        );
        Err((StatusCode::BAD_REQUEST, Json(error_response)))
    }
}

use crate::SharedState;

/// Create authentication router
pub fn create_auth_router() -> Router<SharedState> {
    Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/logout", post(logout_handler))
        .route("/verify-email", get(verify_email_handler))
        .route("/reset-password", post(request_password_reset_handler))
        .route("/reset-password/confirm", post(reset_password_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use imkitchen_core::AppState;
    use imkitchen_shared::{AppConfig, DatabaseConfig, LoggingConfig, ServerConfig};
    use serde_json::Value;
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tower::ServiceExt;

    #[allow(dead_code)]
    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        // Run migrations
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    async fn setup_test_state() -> SharedState {
        let config = AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0,
            },
            database: DatabaseConfig {
                url: "sqlite::memory:".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        };
        let mut app_state = AppState::new(config);
        app_state.initialize_database().await.unwrap();
        Arc::new(RwLock::new(app_state))
    }

    #[tokio::test]
    #[ignore] // Integration test requires complex setup with ConnectInfo
    async fn test_register_endpoint() {
        let app = create_auth_router().with_state(setup_test_state().await);

        let request_body = serde_json::json!({
            "email": "test@example.com",
            "password": "SecureKey123!",
            "name": "Test User",
            "familySize": 4
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/register")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let auth_response: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(auth_response["success"], true);
        assert!(auth_response["user"].is_object());
        assert_eq!(auth_response["user"]["email"], "test@example.com");
        assert_eq!(auth_response["user"]["familySize"], 4);
    }

    #[tokio::test]
    #[ignore] // Integration test requires complex setup with ConnectInfo
    async fn test_register_invalid_password() {
        let app = create_auth_router().with_state(setup_test_state().await);

        let request_body = serde_json::json!({
            "email": "test@example.com",
            "password": "weak",
            "name": "Test User"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/register")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error_response: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(error_response["success"], false);
        assert_eq!(error_response["code"], "PASSWORD_VALIDATION_FAILED");
    }
}
