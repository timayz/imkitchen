// User queries for retrieving user data and projections

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;
use validator::Validate;

use imkitchen_shared::Email;
use crate::commands::EmailValidationError;
use crate::domain::UserProfile;

/// Query to find user by email address
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserByEmailQuery {
    pub email: Email,
    pub request_id: Option<Uuid>,
}

impl UserByEmailQuery {
    pub fn new(email: Email) -> Self {
        Self {
            email,
            request_id: Some(Uuid::new_v4()),
        }
    }
}

/// Query to find user by ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserByIdQuery {
    pub user_id: Uuid,
    pub request_id: Option<Uuid>,
}

impl UserByIdQuery {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            request_id: Some(Uuid::new_v4()),
        }
    }
}

/// Query to find user session by session token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSessionQuery {
    pub session_token: String,
    pub request_id: Option<Uuid>,
}

impl UserSessionQuery {
    pub fn new(session_token: String) -> Self {
        Self {
            session_token,
            request_id: Some(Uuid::new_v4()),
        }
    }
}

/// User account view projection for display purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccountView {
    pub user_id: Uuid,
    pub email: Email,
    pub profile: UserProfile,
    pub is_email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub login_count: i64,
}

/// User session view projection for session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSessionView {
    pub session_id: String,
    pub user_id: Uuid,
    pub email: Email,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
    pub login_ip: Option<String>,
    pub user_agent: Option<String>,
}

/// Response for user by email query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserByEmailResponse {
    pub user: Option<UserAccountView>,
    pub found: bool,
    pub request_id: Option<Uuid>,
}

/// Response for user by ID query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserByIdResponse {
    pub user: Option<UserAccountView>,
    pub found: bool,
    pub request_id: Option<Uuid>,
}

/// Response for user session query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSessionResponse {
    pub session: Option<UserSessionView>,
    pub valid: bool,
    pub request_id: Option<Uuid>,
}

/// Query handler for user-related queries
#[derive(Clone)]
pub struct UserQueryHandler {
    db_pool: SqlitePool,
}

impl UserQueryHandler {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    /// Handle user by email query
    pub async fn handle_user_by_email(
        &self,
        query: UserByEmailQuery,
    ) -> Result<UserByEmailResponse, UserQueryError> {
        // Validate the query
        query.validate()?;

        // For now, use the existing UserRepository to find user
        // This is a simplified implementation
        let user_repository = crate::queries::UserRepository::new(self.db_pool.clone());
        let user_record = user_repository
            .find_by_email(&query.email.value)
            .await?;

        let user = if let Some(record) = user_record {
            Some(UserAccountView {
                user_id: Uuid::parse_str(&record.user_id)
                    .map_err(|_| UserQueryError::InvalidUserId)?,
                email: query.email.clone(),
                profile: UserProfile {
                    family_size: imkitchen_shared::FamilySize::new(record.family_size as u8)
                        .map_err(|_| UserQueryError::InvalidData("family_size".to_string()))?,
                    cooking_skill_level: match record.cooking_skill_level.as_str() {
                        "Beginner" => imkitchen_shared::SkillLevel::Beginner,
                        "Intermediate" => imkitchen_shared::SkillLevel::Intermediate,
                        "Advanced" => imkitchen_shared::SkillLevel::Advanced,
                        _ => imkitchen_shared::SkillLevel::Beginner,
                    },
                    dietary_restrictions: vec![], // TODO: Load from separate table
                    weekday_cooking_minutes: record.weekday_cooking_minutes.unwrap_or(30) as u32,
                    weekend_cooking_minutes: record.weekend_cooking_minutes.unwrap_or(60) as u32,
                },
                is_email_verified: true, // Placeholder
                created_at: DateTime::parse_from_rfc3339(&record.created_at)
                    .map_err(|_| UserQueryError::InvalidData("created_at".to_string()))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&record.updated_at)
                    .map_err(|_| UserQueryError::InvalidData("updated_at".to_string()))?
                    .with_timezone(&Utc),
                last_login_at: None, // TODO: Track login timestamps
                login_count: 0, // TODO: Track login counts
            })
        } else {
            None
        };

        let found = user.is_some();
        Ok(UserByEmailResponse {
            user,
            found,
            request_id: query.request_id,
        })
    }

    /// Handle user by ID query
    pub async fn handle_user_by_id(
        &self,
        query: UserByIdQuery,
    ) -> Result<UserByIdResponse, UserQueryError> {
        // TODO: Implement user by ID lookup
        // For now, return a placeholder response
        Ok(UserByIdResponse {
            user: None,
            found: false,
            request_id: query.request_id,
        })
    }

    /// Handle user session query
    pub async fn handle_user_session(
        &self,
        query: UserSessionQuery,
    ) -> Result<UserSessionResponse, UserQueryError> {
        // TODO: Query the database for session by token
        // For now, return a placeholder response
        Ok(UserSessionResponse {
            session: None,
            valid: false,
            request_id: query.request_id,
        })
    }
}

/// Error types for user queries
#[derive(Debug, thiserror::Error)]
pub enum UserQueryError {
    #[error("User not found")]
    NotFound,
    
    #[error("Invalid user ID format")]
    InvalidUserId,
    
    #[error("Invalid data format for field: {0}")]
    InvalidData(String),
    
    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Email validation error: {0}")]
    EmailValidationError(#[from] EmailValidationError),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use imkitchen_shared::Email;

    #[test]
    fn test_user_by_email_query_creation() {
        let email = Email::new("test@example.com".to_string()).unwrap();
        let query = UserByEmailQuery::new(email.clone());
        
        assert_eq!(query.email, email);
        assert!(query.request_id.is_some());
    }

    #[test]
    fn test_user_by_id_query_creation() {
        let user_id = Uuid::new_v4();
        let query = UserByIdQuery::new(user_id);
        
        assert_eq!(query.user_id, user_id);
        assert!(query.request_id.is_some());
    }

    #[test]
    fn test_user_session_query_creation() {
        let session_token = "test_session_token".to_string();
        let query = UserSessionQuery::new(session_token.clone());
        
        assert_eq!(query.session_token, session_token);
        assert!(query.request_id.is_some());
    }
}