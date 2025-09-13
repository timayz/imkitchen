use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use redis::AsyncCommands;
use thiserror::Error;
use uuid::Uuid;

use crate::models::user::{AuthResponse, CreateUserRequest, LoginRequest, User, UserClaims, UserProfile};
use crate::repositories::UserRepository;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Password hashing error")]
    PasswordHash,
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Email not verified")]
    EmailNotVerified,
    #[error("Invalid verification token")]
    InvalidVerificationToken,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    redis_client: redis::Client,
    jwt_secret: String,
    jwt_expiry_hours: i64,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, redis_client: redis::Client, jwt_secret: String) -> Self {
        Self {
            user_repo,
            redis_client,
            jwt_secret,
            jwt_expiry_hours: 1, // 1 hour for access tokens
        }
    }

    pub async fn register_user(&self, request: CreateUserRequest) -> Result<UserProfile, AuthError> {
        // Validate input
        if request.email.is_empty() || request.password.is_empty() {
            return Err(AuthError::InvalidInput("Email and password are required".to_string()));
        }

        if request.password.len() < 8 {
            return Err(AuthError::InvalidInput("Password must be at least 8 characters long".to_string()));
        }

        // Check if user already exists
        if let Some(_) = self.user_repo.find_by_email(&request.email).await? {
            return Err(AuthError::UserAlreadyExists);
        }

        // Hash password
        let password_hash = self.hash_password(&request.password)?;
        
        // Generate email verification token
        let verification_token = Uuid::new_v4().to_string();

        // Create user
        let user = self.user_repo.create(request, password_hash, Some(verification_token)).await?;

        Ok(user.into())
    }

    pub async fn authenticate_user(&self, request: LoginRequest) -> Result<AuthResponse, AuthError> {
        // Check rate limiting
        self.check_rate_limit(&request.email).await?;

        // Find user by email
        let user = self.user_repo.find_by_email(&request.email).await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        if !self.verify_password(&request.password, &user.password_hash)? {
            self.record_failed_attempt(&request.email).await?;
            return Err(AuthError::InvalidCredentials);
        }

        // Check if email is verified
        if !user.email_verified {
            return Err(AuthError::EmailNotVerified);
        }

        // Generate JWT token
        let token = self.generate_jwt_token(&user)?;

        // Store refresh token in Redis
        self.store_refresh_token(&user.id, &token).await?;

        // Clear failed attempts
        self.clear_failed_attempts(&request.email).await?;

        Ok(AuthResponse {
            access_token: token,
            user: user.into(),
        })
    }

    pub async fn validate_token(&self, token: &str) -> Result<UserClaims, AuthError> {
        let token_data = decode::<UserClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    pub async fn logout(&self, user_id: Uuid, token: &str) -> Result<(), AuthError> {
        // Add token to blacklist in Redis with expiry
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let blacklist_key = format!("blacklist:{}", token);
        let _: () = conn.set_ex(blacklist_key, "1", 3600).await?; // 1 hour expiry

        // Remove refresh token
        let refresh_key = format!("refresh:{}", user_id);
        let _: () = conn.del(refresh_key).await?;

        Ok(())
    }

    pub async fn verify_email(&self, token: &str) -> Result<UserProfile, AuthError> {
        let user = self.user_repo.find_by_verification_token(token).await?
            .ok_or(AuthError::InvalidVerificationToken)?;

        if user.email_verified {
            return Ok(user.into());
        }

        let verified_user = self.user_repo.verify_email(user.id).await?;
        Ok(verified_user.into())
    }

    pub async fn is_token_blacklisted(&self, token: &str) -> Result<bool, AuthError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let blacklist_key = format!("blacklist:{}", token);
        let exists: bool = conn.exists(blacklist_key).await?;
        Ok(exists)
    }

    fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthError::PasswordHash)?;
        Ok(password_hash.to_string())
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| AuthError::PasswordHash)?;
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    fn generate_jwt_token(&self, user: &User) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.jwt_expiry_hours);

        let claims = UserClaims {
            sub: user.id,
            email: user.email.clone(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    async fn store_refresh_token(&self, user_id: &Uuid, token: &str) -> Result<(), AuthError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("refresh:{}", user_id);
        let _: () = conn.set_ex(key, token, 30 * 24 * 3600).await?; // 30 days
        Ok(())
    }

    async fn check_rate_limit(&self, email: &str) -> Result<(), AuthError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("rate_limit:{}", email);
        let attempts: Option<i32> = conn.get(&key).await?;
        
        if let Some(count) = attempts {
            if count >= 5 { // Max 5 attempts
                return Err(AuthError::RateLimitExceeded);
            }
        }
        
        Ok(())
    }

    async fn record_failed_attempt(&self, email: &str) -> Result<(), AuthError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("rate_limit:{}", email);
        let _: () = conn.incr(&key, 1).await?;
        let _: () = conn.expire(&key, 300).await?; // 5 minutes expiry
        Ok(())
    }

    async fn clear_failed_attempts(&self, email: &str) -> Result<(), AuthError> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let key = format!("rate_limit:{}", email);
        let _: () = conn.del(&key).await?;
        Ok(())
    }
}