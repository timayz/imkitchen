// RegisterUserCommand for complex user registration with email verification

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::{User, UserProfile};
use imkitchen_shared::{Email, FamilySize, Password, SkillLevel};

/// Command for registering a new user with email verification workflow
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterUserCommand {
    pub email: Email,

    pub password: Password,

    pub family_size: FamilySize,

    pub cooking_skill_level: SkillLevel,

    /// Optional meal planning preferences
    pub weekday_cooking_minutes: Option<u32>,
    pub weekend_cooking_minutes: Option<u32>,

    /// Registration context for security audit
    pub registration_ip: Option<String>,
    pub user_agent: Option<String>,

    /// Optional request ID for tracking
    pub request_id: Option<Uuid>,
}

impl RegisterUserCommand {
    /// Create a new RegisterUserCommand
    pub fn new(
        email: Email,
        password: Password,
        family_size: FamilySize,
        cooking_skill_level: SkillLevel,
    ) -> Self {
        Self {
            email,
            password,
            family_size,
            cooking_skill_level,
            weekday_cooking_minutes: None,
            weekend_cooking_minutes: None,
            registration_ip: None,
            user_agent: None,
            request_id: Some(Uuid::new_v4()),
        }
    }

    /// Create a RegisterUserCommand with full details
    pub fn with_details(
        email: Email,
        password: Password,
        family_size: FamilySize,
        cooking_skill_level: SkillLevel,
        weekday_cooking_minutes: Option<u32>,
        weekend_cooking_minutes: Option<u32>,
        registration_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            email,
            password,
            family_size,
            cooking_skill_level,
            weekday_cooking_minutes,
            weekend_cooking_minutes,
            registration_ip,
            user_agent,
            request_id: Some(Uuid::new_v4()),
        }
    }
}

/// Response from user registration command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterUserResponse {
    pub user_id: Uuid,
    pub email: Email,
    pub verification_required: bool,
    pub verification_token: Option<String>,
    pub request_id: Option<Uuid>,
}

/// Service for handling user registration commands
#[derive(Clone)]
pub struct RegisterUserService {
    // This would typically contain dependencies like:
    // - Email service for verification emails
    // - User repository for persistence
    // - Event store for storing events
    db_pool: Option<sqlx::SqlitePool>,
}

impl Default for RegisterUserService {
    fn default() -> Self {
        Self::new()
    }
}

impl RegisterUserService {
    pub fn new() -> Self {
        Self { db_pool: None }
    }

    pub fn with_database(db_pool: sqlx::SqlitePool) -> Self {
        Self {
            db_pool: Some(db_pool),
        }
    }

    /// Handle user registration command
    pub async fn handle(
        &self,
        command: RegisterUserCommand,
    ) -> Result<RegisterUserResponse, RegisterUserError> {
        // Validate the command
        command.validate()?;

        // Create user profile from command
        let profile = UserProfile {
            family_size: command.family_size,
            cooking_skill_level: command.cooking_skill_level,
            dietary_restrictions: vec![], // Default empty
            weekday_cooking_minutes: command.weekday_cooking_minutes.unwrap_or(30),
            weekend_cooking_minutes: command.weekend_cooking_minutes.unwrap_or(60),
        };

        // Create user and registration event
        let (user, _registration_event) = User::new_with_context(
            command.email.clone(),
            command.password,
            profile,
            command.registration_ip,
            command.user_agent,
        );

        // Check if email already exists and store user if database is available
        if let Some(ref pool) = self.db_pool {
            // Check if email already exists
            let existing_user = sqlx::query!(
                "SELECT id FROM user_profiles WHERE email = ?",
                command.email.value
            )
            .fetch_optional(pool)
            .await
            .map_err(|e| RegisterUserError::DatabaseError(e.to_string()))?;

            if existing_user.is_some() {
                return Err(RegisterUserError::EmailAlreadyExists);
            }

            // Store user in database - prepare values to avoid borrow checker issues
            let user_id_str = user.user_id.to_string();
            let family_size_i64 = user.profile.family_size.value as i64;
            let skill_level_str = user.profile.cooking_skill_level.to_string();
            let dietary_restrictions_json =
                serde_json::to_string(&user.profile.dietary_restrictions).unwrap_or_default();
            let created_at_str = user.created_at.to_rfc3339();
            let updated_at_str = user.updated_at.to_rfc3339();

            sqlx::query!(
                r#"
                INSERT INTO user_profiles (
                    id, email, password_hash, family_size, 
                    skill_level, dietary_restrictions,
                    email_verified, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                user_id_str,
                user.email.value,
                user.password_hash,
                family_size_i64,
                skill_level_str,
                dietary_restrictions_json,
                user.is_email_verified,
                created_at_str,
                updated_at_str
            )
            .execute(pool)
            .await
            .map_err(|e| RegisterUserError::DatabaseError(e.to_string()))?;
        }

        // TODO: Store registration event in event store
        // TODO: Send verification email via email service
        // TODO: Generate verification token

        // Return success response
        let response = RegisterUserResponse {
            user_id: user.user_id,
            email: command.email,
            verification_required: false, // For now, skip email verification
            verification_token: None,
            request_id: command.request_id,
        };

        Ok(response)
    }
}

/// Error types for user registration
#[derive(Debug, thiserror::Error)]
pub enum RegisterUserError {
    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Invalid email format")]
    InvalidEmail,

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
    use imkitchen_shared::{Email, FamilySize, Password, SkillLevel};

    #[test]
    fn test_register_user_command_creation() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let password = Password::new("SecurePass123!".to_string()).unwrap();
        let family_size = FamilySize::Family2;
        let skill_level = SkillLevel::Beginner;

        let command = RegisterUserCommand::new(email.clone(), password, family_size, skill_level);

        assert_eq!(command.email, email);
        assert_eq!(command.family_size, family_size);
        assert_eq!(command.cooking_skill_level, skill_level);
        assert!(command.request_id.is_some());
    }

    #[tokio::test]
    async fn test_register_user_service_handle() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let password = Password::new("SecurePass123!".to_string()).unwrap();
        let family_size = FamilySize::Family4;
        let skill_level = SkillLevel::Intermediate;

        let command = RegisterUserCommand::new(email.clone(), password, family_size, skill_level);
        let service = RegisterUserService::new();

        let result = service.handle(command).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.email, email);
        assert!(response.verification_required);
        assert!(response.verification_token.is_some());
    }
}
