// Direct login service bypassing Evento for simple authentication

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;
use validator::Validate;

use imkitchen_shared::{Email, Password};
use crate::domain::Session;
use crate::queries::UserRepository;

/// Simple login command that bypasses Evento
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginCommand {
    pub email: Email,
    pub password: Password,
    
    /// Optional context for security logging
    pub login_ip: Option<String>,
    pub user_agent: Option<String>,
}

impl LoginCommand {
    pub fn new(email: Email, password: Password) -> Self {
        Self {
            email,
            password,
            login_ip: None,
            user_agent: None,
        }
    }

    pub fn with_context(
        email: Email,
        password: Password,
        login_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            email,
            password,
            login_ip,
            user_agent,
        }
    }
}

/// Response from successful login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: Uuid,
    pub email: Email,
    pub session_token: String,
    pub expires_at: DateTime<Utc>,
    pub is_email_verified: bool,
}

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub session_id: String,
    pub user_id: Uuid,
    pub email: Email,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub login_ip: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

impl UserSession {
    /// Create a new user session
    pub fn new(
        user_id: Uuid,
        email: Email,
        login_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        let session_id = format!("{}{}", Uuid::new_v4(), Uuid::new_v4());
        let expires_at = now + Duration::hours(24); // Session expires in 24 hours

        Self {
            session_id,
            user_id,
            email,
            created_at: now,
            expires_at,
            last_activity: now,
            login_ip,
            user_agent,
            is_active: true,
        }
    }

    /// Check if session is valid (not expired and active)
    pub fn is_valid(&self) -> bool {
        self.is_active && Utc::now() < self.expires_at
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Invalidate the session
    pub fn invalidate(&mut self) {
        self.is_active = false;
    }

    /// Extend session expiry
    pub fn extend(&mut self, hours: i64) {
        self.expires_at = self.expires_at + Duration::hours(hours);
        self.update_activity();
    }
}

/// Login service with session management
#[derive(Clone)]
pub struct DirectLoginService {
    _db_pool: SqlitePool,
    user_repository: UserRepository,
}

impl DirectLoginService {
    pub fn new(db_pool: SqlitePool) -> Self {
        let user_repository = UserRepository::new(db_pool.clone());
        Self {
            _db_pool: db_pool,
            user_repository,
        }
    }

    /// Handle login with session management
    pub async fn login(&self, command: LoginCommand) -> Result<LoginResponse, LoginError> {
        // Validate command
        command.validate()?;

        // Find user by email
        let user_record = self
            .user_repository
            .find_by_email(&command.email.value)
            .await?
            .ok_or(LoginError::InvalidCredentials)?;

        // Verify password hash
        if !self.verify_password(&command.password, &user_record.password_hash)? {
            return Err(LoginError::InvalidCredentials);
        }

        let user_id = Uuid::parse_str(&user_record.user_id).map_err(|_| LoginError::InvalidUserId)?;

        // Create session using the Session domain object
        let session = Session::new(
            user_id,
            command.login_ip.clone(),
            command.user_agent.clone(),
        );

        // TODO: Store session in database
        // For now, we'll just create the response with the session token

        let response = LoginResponse {
            user_id: session.user_id,
            email: command.email,
            session_token: session.session_id.to_string(),
            expires_at: session.expires_at,
            is_email_verified: true, // Placeholder
        };

        Ok(response)
    }

    /// Verify password hash
    fn verify_password(&self, password: &Password, hash: &str) -> Result<bool, LoginError> {
        // TODO: Implement actual password verification using bcrypt or similar
        // For now, this is a placeholder
        Ok(password.hash() == hash)
    }

    /// Validate session token
    pub async fn validate_session(&self, session_token: &str) -> Result<Session, LoginError> {
        // Parse session token as UUID
        let _session_id = Uuid::parse_str(session_token)
            .map_err(|_| LoginError::InvalidSession)?;

        // TODO: Load session from database or event store
        // For now, return an error as we haven't implemented session storage yet
        Err(LoginError::InvalidSession)
    }

    /// Logout user (invalidate session)
    pub async fn logout(&self, session_token: &str) -> Result<(), LoginError> {
        // Parse session token as UUID
        let _session_id = Uuid::parse_str(session_token)
            .map_err(|_| LoginError::InvalidSession)?;

        // TODO: Invalidate session in database or event store
        // For now, this is a placeholder
        Ok(())
    }

    /// Refresh session (extend expiry)
    pub async fn refresh_session(&self, session_token: &str) -> Result<Session, LoginError> {
        // Parse session token as UUID
        let _session_id = Uuid::parse_str(session_token)
            .map_err(|_| LoginError::InvalidSession)?;

        // TODO: Load and update session in database or event store
        // For now, this is a placeholder
        Err(LoginError::InvalidSession)
    }
}

/// Error types for direct login operations
#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Email not verified")]
    EmailNotVerified,
    
    #[error("Account locked")]
    AccountLocked,
    
    #[error("Invalid session")]
    InvalidSession,
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("Invalid user ID format")]
    InvalidUserId,
    
    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("User repository error: {0}")]
    UserRepositoryError(#[from] crate::commands::EmailValidationError),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use imkitchen_shared::{Email, Password};

    #[test]
    fn test_login_command_creation() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let password = Password::new("SecurePass123!".to_string()).unwrap();
        
        let command = LoginCommand::new(email.clone(), password);
        
        assert_eq!(command.email, email);
        assert!(command.login_ip.is_none());
        assert!(command.user_agent.is_none());
    }

    #[test]
    fn test_user_session_creation() {
        let user_id = Uuid::new_v4();
        let email = Email::new("test@example.com".to_string()).unwrap();
        
        let session = UserSession::new(user_id, email.clone(), None, None);
        
        assert_eq!(session.user_id, user_id);
        assert_eq!(session.email, email);
        assert!(session.is_valid());
        assert!(session.is_active);
    }

    #[test]
    fn test_user_session_invalidation() {
        let user_id = Uuid::new_v4();
        let email = Email::new("test@example.com".to_string()).unwrap();
        
        let mut session = UserSession::new(user_id, email, None, None);
        assert!(session.is_valid());
        
        session.invalidate();
        assert!(!session.is_valid());
    }

    #[test]
    fn test_user_session_extension() {
        let user_id = Uuid::new_v4();
        let email = Email::new("test@example.com".to_string()).unwrap();
        
        let mut session = UserSession::new(user_id, email, None, None);
        let original_expiry = session.expires_at;
        
        session.extend(12); // Extend by 12 hours
        assert!(session.expires_at > original_expiry);
    }
}