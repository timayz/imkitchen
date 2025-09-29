// UserLoggedIn event for login tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event fired when a user successfully logs in
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedIn {
    pub user_id: Uuid,
    pub logged_in_at: DateTime<Utc>,

    /// IP address from which login occurred (for security)
    pub login_ip: Option<String>,

    /// User agent string (for security and analytics)
    pub user_agent: Option<String>,

    /// Session ID for tracking user sessions
    pub session_id: Option<String>,
}

impl UserLoggedIn {
    /// Create a new UserLoggedIn event
    pub fn new(
        user_id: Uuid,
        login_ip: Option<String>,
        user_agent: Option<String>,
        session_id: Option<String>,
    ) -> Self {
        Self {
            user_id,
            logged_in_at: Utc::now(),
            login_ip,
            user_agent,
            session_id,
        }
    }

    /// Create a basic UserLoggedIn event with minimal information
    pub fn basic(user_id: Uuid) -> Self {
        Self::new(user_id, None, None, None)
    }
}
