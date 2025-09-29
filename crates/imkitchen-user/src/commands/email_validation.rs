// Email validation commands using Evento for async database checks

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Command to check if an email address already exists in the database
/// This command uses Evento for async database queries
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CheckEmailExistsCommand {
    #[validate(email)]
    pub email: String,

    /// Optional request ID for tracking async validation requests
    pub request_id: Option<Uuid>,
}

impl CheckEmailExistsCommand {
    pub fn new(email: String) -> Self {
        Self {
            email,
            request_id: Some(Uuid::new_v4()),
        }
    }

    pub fn with_request_id(email: String, request_id: Uuid) -> Self {
        Self {
            email,
            request_id: Some(request_id),
        }
    }
}

/// Response for email existence check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckEmailExistsResponse {
    pub email: String,
    pub exists: bool,
    pub request_id: Option<Uuid>,
}

/// Command to validate username availability
/// Note: In this system, email serves as the username
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidateUsernameAvailabilityCommand {
    #[validate(email)]
    pub username: String, // This is the email address

    /// Optional request ID for tracking async validation requests
    pub request_id: Option<Uuid>,
}

impl ValidateUsernameAvailabilityCommand {
    pub fn new(username: String) -> Self {
        Self {
            username,
            request_id: Some(Uuid::new_v4()),
        }
    }
}

/// Response for username availability check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateUsernameAvailabilityResponse {
    pub username: String,
    pub available: bool,
    pub request_id: Option<Uuid>,
    pub suggestions: Vec<String>, // Alternative usernames if not available
}

/// Error types for email validation commands
#[derive(Debug, thiserror::Error)]
pub enum EmailValidationError {
    #[error("Invalid email format: {0}")]
    InvalidFormat(String),

    #[error("Database query failed: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),

    #[error("Request timeout")]
    Timeout,
}

// TODO: Implement Evento command handlers for these commands
// These will be implemented when integrating with the database layer
