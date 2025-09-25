use crate::models::user::{
    AuthResponse, CreateUserRequest, LoginRequest, User, UserPublic, UserSession,
};
use crate::repositories::{UserRepository, UserRepositoryError};
use crate::services::email_service::{EmailError, EmailService};
use crate::utils::password::{hash_password, verify_password, PasswordError};
use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Repository error: {0}")]
    Repository(#[from] UserRepositoryError),
    #[error("Password error: {0}")]
    Password(#[from] PasswordError),
    #[error("Email error: {0}")]
    Email(#[from] EmailError),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Email already exists")]
    EmailExists,
    #[error("Email not verified")]
    EmailNotVerified,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("User not found")]
    UserNotFound,
    #[error("Session expired or invalid")]
    InvalidSession,
    #[error("Email service unavailable")]
    EmailServiceUnavailable,
}

/// Rate limiting structure
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub attempts: u32,
    pub last_attempt: DateTime<Utc>,
}

#[derive(Clone)]
pub struct AuthService {
    user_repository: UserRepository,
    email_service: EmailService,
    // In-memory rate limiting - in production would use Redis
    login_rate_limits: Arc<RwLock<HashMap<String, RateLimit>>>,
    registration_rate_limits: Arc<RwLock<HashMap<String, RateLimit>>>,
    password_reset_rate_limits: Arc<RwLock<HashMap<String, RateLimit>>>,
}

impl AuthService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            user_repository: UserRepository::new(pool),
            email_service: EmailService::new(true), // Development mode for now
            login_rate_limits: Arc::new(RwLock::new(HashMap::new())),
            registration_rate_limits: Arc::new(RwLock::new(HashMap::new())),
            password_reset_rate_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new user
    pub async fn register_user(
        &self,
        request: CreateUserRequest,
        ip_address: String,
    ) -> Result<AuthResponse, AuthError> {
        // Check rate limit for registration (3 per hour per IP)
        self.check_rate_limit(
            &self.registration_rate_limits,
            &ip_address,
            3,
            Duration::hours(1),
        )
        .await?;

        info!("Attempting to register user with email: {}", request.email);

        // Validate email format
        if !self.is_valid_email(&request.email) {
            warn!("Invalid email format: {}", request.email);
            return Err(AuthError::InvalidCredentials);
        }

        // Hash password
        let password_hash = hash_password(&request.password)?;

        // Create user
        let user = match self
            .user_repository
            .create_user(request, password_hash)
            .await
        {
            Ok(user) => user,
            Err(UserRepositoryError::EmailExists) => {
                warn!("Attempted registration with existing email");
                return Err(AuthError::EmailExists);
            }
            Err(e) => return Err(AuthError::Repository(e)),
        };

        info!("User registered successfully: {}", user.id);

        // Update rate limit
        self.update_rate_limit(&self.registration_rate_limits, &ip_address)
            .await;

        // Send verification email
        if let Some(verification_token) = &user.email_verification_token {
            match self
                .email_service
                .send_verification_email(&user.email, &user.name, verification_token)
                .await
            {
                Ok(()) => {
                    info!("Verification email sent successfully to: {}", user.email);
                }
                Err(e) => {
                    warn!("Failed to send verification email to {}: {}", user.email, e);
                    // Don't fail registration if email sending fails, just log the error
                }
            }
        } else {
            warn!("No verification token found for user: {}", user.id);
        }

        let user_public = UserPublic::from(user);
        Ok(AuthResponse {
            user: user_public,
            message: "Registration successful. Please check your email to verify your account."
                .to_string(),
        })
    }

    /// Authenticate user login
    pub async fn login_user(
        &self,
        request: LoginRequest,
        ip_address: String,
    ) -> Result<(AuthResponse, UserSession), AuthError> {
        // Check rate limit for login (5 attempts per 15 minutes per IP)
        self.check_rate_limit(
            &self.login_rate_limits,
            &ip_address,
            5,
            Duration::minutes(15),
        )
        .await?;

        info!("Attempting login for email: {}", request.email);

        // Find user by email
        let user = match self.user_repository.find_by_email(&request.email).await? {
            Some(user) => user,
            None => {
                warn!("Login attempt for non-existent email: {}", request.email);
                self.update_rate_limit(&self.login_rate_limits, &ip_address)
                    .await;
                return Err(AuthError::InvalidCredentials);
            }
        };

        // Verify password
        match verify_password(&request.password, &user.password_hash)? {
            true => {
                info!("Password verification successful for user: {}", user.id);
            }
            false => {
                warn!("Password verification failed for user: {}", user.id);
                self.update_rate_limit(&self.login_rate_limits, &ip_address)
                    .await;
                return Err(AuthError::InvalidCredentials);
            }
        }

        // Check if email is verified
        if !user.email_verified {
            warn!("Login attempt for unverified email: {}", user.email);
            return Err(AuthError::EmailNotVerified);
        }

        // Create session
        let session = self.user_repository.create_session(&user.id).await?;

        // Update last active
        self.user_repository.update_last_active(&user.id).await?;

        info!("User logged in successfully: {}", user.id);

        // Clear rate limit on successful login
        self.clear_rate_limit(&self.login_rate_limits, &ip_address)
            .await;

        let user_public = UserPublic::from(user);
        let auth_response = AuthResponse {
            user: user_public,
            message: "Login successful".to_string(),
        };

        Ok((auth_response, session))
    }

    /// Validate session
    pub async fn validate_session(&self, session_token: &str) -> Result<User, AuthError> {
        let session = match self.user_repository.find_session(session_token).await? {
            Some(session) => session,
            None => return Err(AuthError::InvalidSession),
        };

        if session.is_expired() {
            warn!("Expired session attempted: {}", session.id);
            return Err(AuthError::InvalidSession);
        }

        // Get user
        let user = match self.user_repository.find_by_id(&session.user_id).await? {
            Some(user) => user,
            None => {
                error!("Session exists for non-existent user: {}", session.user_id);
                return Err(AuthError::UserNotFound);
            }
        };

        // Update last active
        self.user_repository.update_last_active(&user.id).await?;

        Ok(user)
    }

    /// Logout user (delete session)
    pub async fn logout_user(&self, session_token: &str) -> Result<(), AuthError> {
        self.user_repository.delete_session(session_token).await?;
        info!("User session deleted: {}", session_token);
        Ok(())
    }

    /// Verify email with token
    pub async fn verify_email(&self, token: &str) -> Result<bool, AuthError> {
        let success = self.user_repository.verify_email(token).await?;
        if success {
            info!("Email verification successful for token: {}", token);
        } else {
            warn!("Email verification failed for token: {}", token);
        }
        Ok(success)
    }

    /// Request password reset
    pub async fn request_password_reset(
        &self,
        email: &str,
        ip_address: String,
    ) -> Result<String, AuthError> {
        // Check rate limit for password reset (1 per 10 minutes per email)
        self.check_rate_limit(
            &self.password_reset_rate_limits,
            &ip_address,
            1,
            Duration::minutes(10),
        )
        .await?;

        info!("Password reset requested for email: {}", email);

        // Check if user exists
        if self.user_repository.find_by_email(email).await?.is_none() {
            warn!("Password reset requested for non-existent email: {}", email);
            // Don't reveal if email exists, but still update rate limit
            self.update_rate_limit(&self.password_reset_rate_limits, &ip_address)
                .await;
            return Ok("If the email exists, a password reset link has been sent.".to_string());
        }

        // Generate reset token
        let reset_token = uuid::Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::hours(1); // 1 hour to reset

        // Get user info for email sending
        let user = self.user_repository.find_by_email(email).await?.unwrap(); // We already checked it exists

        // Set password reset token
        self.user_repository
            .set_password_reset_token(email, &reset_token, expires_at)
            .await?;

        // Update rate limit
        self.update_rate_limit(&self.password_reset_rate_limits, &ip_address)
            .await;

        // Send password reset email
        match self
            .email_service
            .send_password_reset_email(email, &user.name, &reset_token)
            .await
        {
            Ok(()) => {
                info!("Password reset email sent successfully to: {}", email);
            }
            Err(e) => {
                warn!("Failed to send password reset email to {}: {}", email, e);
                // Don't fail the request if email sending fails, just log the error
            }
        }

        info!("Password reset token generated for email: {}", email);
        Ok("If the email exists, a password reset link has been sent.".to_string())
    }

    /// Reset password with token
    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<bool, AuthError> {
        info!("Password reset attempted with token: {}", token);

        // Hash new password
        let password_hash = hash_password(new_password)?;

        // Reset password
        let success = self
            .user_repository
            .reset_password(token, &password_hash)
            .await?;

        if success {
            info!("Password reset successful for token: {}", token);
        } else {
            warn!("Password reset failed for token: {}", token);
        }

        Ok(success)
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<u64, AuthError> {
        let count = self.user_repository.cleanup_expired_sessions().await?;
        if count > 0 {
            info!("Cleaned up {} expired sessions", count);
        }
        Ok(count)
    }

    /// Basic email validation
    fn is_valid_email(&self, email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() > 5 && email.len() < 255
    }

    /// Check rate limit
    async fn check_rate_limit(
        &self,
        rate_limits: &Arc<RwLock<HashMap<String, RateLimit>>>,
        key: &str,
        max_attempts: u32,
        window: Duration,
    ) -> Result<(), AuthError> {
        let mut limits = rate_limits.write().await;
        let now = Utc::now();

        if let Some(limit) = limits.get(key) {
            // If window has passed, reset attempts
            if now - limit.last_attempt > window {
                limits.remove(key);
            } else if limit.attempts >= max_attempts {
                return Err(AuthError::RateLimitExceeded);
            }
        }

        Ok(())
    }

    /// Update rate limit
    async fn update_rate_limit(
        &self,
        rate_limits: &Arc<RwLock<HashMap<String, RateLimit>>>,
        key: &str,
    ) {
        let mut limits = rate_limits.write().await;
        let now = Utc::now();

        let limit = limits.entry(key.to_string()).or_insert(RateLimit {
            attempts: 0,
            last_attempt: now,
        });

        limit.attempts += 1;
        limit.last_attempt = now;
    }

    /// Clear rate limit
    async fn clear_rate_limit(
        &self,
        rate_limits: &Arc<RwLock<HashMap<String, RateLimit>>>,
        key: &str,
    ) {
        let mut limits = rate_limits.write().await;
        limits.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_service() -> AuthService {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        // Run migrations
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        AuthService::new(pool)
    }

    #[tokio::test]
    async fn test_user_registration_and_login_flow() {
        let auth_service = setup_test_service().await;
        let ip_address = "127.0.0.1".to_string();

        // Register user
        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "SecureKey123!".to_string(),
            name: "Test User".to_string(),
            family_size: Some(4),
        };

        let auth_response = auth_service
            .register_user(request, ip_address.clone())
            .await
            .unwrap();
        assert_eq!(auth_response.user.email, "test@example.com");
        assert_eq!(auth_response.user.name, "Test User");
        assert!(!auth_response.user.email_verified);

        // Verify email (simulate clicking verification link)
        // First get the user to access verification token
        let user = auth_service
            .user_repository
            .find_by_email("test@example.com")
            .await
            .unwrap()
            .unwrap();
        let verification_token = user.email_verification_token.unwrap();

        let verified = auth_service
            .verify_email(&verification_token)
            .await
            .unwrap();
        assert!(verified);

        // Now try to login
        let login_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "SecureKey123!".to_string(),
        };

        let (login_response, session) = auth_service
            .login_user(login_request, ip_address)
            .await
            .unwrap();
        assert_eq!(login_response.user.email, "test@example.com");
        assert!(login_response.user.email_verified);
        assert!(!session.is_expired());
    }

    #[tokio::test]
    async fn test_invalid_password_validation() {
        let auth_service = setup_test_service().await;
        let ip_address = "127.0.0.1".to_string();

        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "weak".to_string(), // Too weak
            name: "Test User".to_string(),
            family_size: None,
        };

        let result = auth_service.register_user(request, ip_address).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::Password(_)));
    }

    #[tokio::test]
    async fn test_duplicate_email_registration() {
        let auth_service = setup_test_service().await;
        let ip_address = "127.0.0.1".to_string();

        let request = CreateUserRequest {
            email: "duplicate@example.com".to_string(),
            password: "SecureKey123!".to_string(),
            name: "First User".to_string(),
            family_size: None,
        };

        // First registration should succeed
        auth_service
            .register_user(request.clone(), ip_address.clone())
            .await
            .unwrap();

        // Second registration should fail
        let result = auth_service.register_user(request, ip_address).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::EmailExists));
    }
}
