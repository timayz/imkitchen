// UserLoggedIn event for login tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event fired when a user successfully logs in
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedIn {
    pub user_id: Uuid,
    pub logged_in_at: DateTime<Utc>,
}
