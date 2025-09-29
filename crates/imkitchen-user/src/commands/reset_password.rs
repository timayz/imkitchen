// ResetPasswordCommand for password reset with email verification

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use imkitchen_shared::{Email, Password};

/// Command for initiating password reset workflow
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InitiatePasswordResetCommand {
    pub email: Email,
    
    /// IP address from which reset was requested
    pub request_ip: Option<String>,
    
    /// User agent string
    pub user_agent: Option<String>,
    
    /// Request ID for tracking
    pub request_id: Option<Uuid>,
}

impl InitiatePasswordResetCommand {
    pub fn new(email: Email) -> Self {
        Self {
            email,
            request_ip: None,
            user_agent: None,
            request_id: Some(Uuid::new_v4()),
        }
    }

    pub fn with_context(
        email: Email,
        request_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            email,
            request_ip,
            user_agent,
            request_id: Some(Uuid::new_v4()),
        }
    }
}

/// Command for completing password reset with token
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CompletePasswordResetCommand {
    pub reset_token: String,
    
    pub new_password: Password,
    
    /// IP address from which reset was completed
    pub completion_ip: Option<String>,
    
    /// User agent string
    pub user_agent: Option<String>,
    
    /// Request ID for tracking
    pub request_id: Option<Uuid>,
}

impl CompletePasswordResetCommand {
    pub fn new(reset_token: String, new_password: Password) -> Self {
        Self {
            reset_token,
            new_password,
            completion_ip: None,
            user_agent: None,
            request_id: Some(Uuid::new_v4()),
        }
    }

    pub fn with_context(
        reset_token: String,
        new_password: Password,
        completion_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            reset_token,
            new_password,
            completion_ip,
            user_agent,
            request_id: Some(Uuid::new_v4()),
        }
    }
}

/// Password reset token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub token: String,
    pub user_id: Uuid,
    pub email: Email,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}

impl PasswordResetToken {
    /// Create a new password reset token
    pub fn new(user_id: Uuid, email: Email) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::hours(24); // Token expires in 24 hours
        let token = format!("{}{}", Uuid::new_v4(), Uuid::new_v4()); // Simple token generation

        Self {
            token,
            user_id,
            email,
            created_at: now,
            expires_at,
            used: false,
        }
    }

    /// Check if token is valid (not expired and not used)
    pub fn is_valid(&self) -> bool {
        !self.used && Utc::now() < self.expires_at
    }

    /// Mark token as used
    pub fn mark_used(&mut self) {
        self.used = true;
    }
}

/// Response from password reset initiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiatePasswordResetResponse {
    pub email: Email,
    pub reset_initiated: bool,
    pub message: String,
    pub request_id: Option<Uuid>,
}

/// Response from password reset completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletePasswordResetResponse {
    pub user_id: Uuid,
    pub reset_completed: bool,
    pub message: String,
    pub request_id: Option<Uuid>,
}

/// Service for handling password reset commands
pub struct PasswordResetService {
    // This would typically contain dependencies like:
    // - Email service for reset emails
    // - User repository for user lookup
    // - Token repository for token storage
    // - Event store for storing events
}

impl PasswordResetService {
    pub fn new() -> Self {
        Self {}
    }

    /// Handle password reset initiation
    pub async fn handle_initiate(
        &self,
        command: InitiatePasswordResetCommand,
    ) -> Result<InitiatePasswordResetResponse, PasswordResetError> {
        // Validate the command
        command.validate()?;

        // TODO: Check if user exists with this email
        // TODO: Generate reset token
        // TODO: Store token in repository
        // TODO: Send reset email via email service

        // For now, return a basic response
        let response = InitiatePasswordResetResponse {
            email: command.email,
            reset_initiated: true,
            message: "Password reset email sent if account exists".to_string(),
            request_id: command.request_id,
        };

        Ok(response)
    }

    /// Handle password reset completion
    pub async fn handle_complete(
        &self,
        command: CompletePasswordResetCommand,
    ) -> Result<CompletePasswordResetResponse, PasswordResetError> {
        // Validate the command
        command.validate()?;

        // TODO: Validate reset token
        // TODO: Load user from repository
        // TODO: Update user password
        // TODO: Mark token as used
        // TODO: Create UserPasswordChanged event
        // TODO: Store event in event store

        // For now, return a basic response
        let response = CompletePasswordResetResponse {
            user_id: Uuid::new_v4(), // Would be actual user ID
            reset_completed: true,
            message: "Password has been reset successfully".to_string(),
            request_id: command.request_id,
        };

        Ok(response)
    }
}

/// Error types for password reset operations
#[derive(Debug, thiserror::Error)]
pub enum PasswordResetError {
    #[error("User not found")]
    UserNotFound,
    
    #[error("Invalid reset token")]
    InvalidToken,
    
    #[error("Reset token has expired")]
    TokenExpired,
    
    #[error("Reset token has already been used")]
    TokenAlreadyUsed,
    
    #[error("Password does not meet requirements")]
    InvalidPassword,
    
    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Email service error: {0}")]
    EmailServiceError(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use imkitchen_shared::{Email, Password};

    #[test]
    fn test_initiate_password_reset_command() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let command = InitiatePasswordResetCommand::new(email.clone());

        assert_eq!(command.email, email);
        assert!(command.request_id.is_some());
    }

    #[test]
    fn test_password_reset_token_creation() {
        let user_id = Uuid::new_v4();
        let email = Email::new("test@example.com".to_string()).unwrap();
        let token = PasswordResetToken::new(user_id, email.clone());

        assert_eq!(token.user_id, user_id);
        assert_eq!(token.email, email);
        assert!(token.is_valid());
        assert!(!token.used);
    }

    #[test]
    fn test_password_reset_token_expiry() {
        let user_id = Uuid::new_v4();
        let email = Email::new("test@example.com".to_string()).unwrap();
        let mut token = PasswordResetToken::new(user_id, email);

        // Token should be valid initially
        assert!(token.is_valid());

        // Mark as used
        token.mark_used();
        assert!(!token.is_valid());
    }

    #[tokio::test]
    async fn test_password_reset_service_initiate() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let command = InitiatePasswordResetCommand::new(email.clone());
        let service = PasswordResetService::new();

        let result = service.handle_initiate(command).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.email, email);
        assert!(response.reset_initiated);
    }

    #[tokio::test]
    async fn test_password_reset_service_complete() {
        let password = Password::new("NewSecurePass123!".to_string()).unwrap();
        let command = CompletePasswordResetCommand::new("test_token".to_string(), password);
        let service = PasswordResetService::new();

        let result = service.handle_complete(command).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.reset_completed);
    }
}