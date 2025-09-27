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
}