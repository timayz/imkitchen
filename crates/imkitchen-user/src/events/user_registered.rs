// UserRegistered event for user registration workflow

use chrono::{DateTime, Utc};
use imkitchen_shared::Email;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::UserProfile;

/// Event fired when a new user successfully registers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegistered {
    pub user_id: Uuid,
    pub email: Email,
    pub password_hash: String,
    pub profile: UserProfile,
    pub created_at: DateTime<Utc>,
    
    /// IP address from which registration occurred (for security)
    pub registration_ip: Option<String>,
    
    /// User agent string (for security and analytics)
    pub user_agent: Option<String>,
}

impl UserRegistered {
    /// Create a new UserRegistered event
    pub fn new(
        user_id: Uuid,
        email: Email,
        password_hash: String,
        profile: UserProfile,
        registration_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            user_id,
            email,
            password_hash,
            profile,
            created_at: Utc::now(),
            registration_ip,
            user_agent,
        }
    }

    /// Create a basic UserRegistered event with minimal information
    pub fn basic(
        user_id: Uuid,
        email: Email,
        password_hash: String,
        profile: UserProfile,
    ) -> Self {
        Self::new(user_id, email, password_hash, profile, None, None)
    }
}
