// UserPasswordChanged event for password change tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event fired when a user changes their password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPasswordChanged {
    pub user_id: Uuid,
    pub password_hash: String,
    pub changed_at: DateTime<Utc>,
}