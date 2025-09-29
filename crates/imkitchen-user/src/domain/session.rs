// Session aggregate for managing user authentication sessions

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::events::UserLoggedIn;

/// Simple session data structure for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub session_id: Uuid,

    /// User ID associated with this session
    pub user_id: Uuid,

    /// When the session was created
    pub created_at: DateTime<Utc>,

    /// When the session expires
    pub expires_at: DateTime<Utc>,

    /// IP address from which the session was created
    pub created_from_ip: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Whether the session is currently active
    pub is_active: bool,

    /// Last time the session was accessed (for extending expiry)
    pub last_accessed_at: DateTime<Utc>,
}

impl Session {
    /// Create a new session
    pub fn new(user_id: Uuid, login_ip: Option<String>, user_agent: Option<String>) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::days(7); // 7 days expiry

        Self {
            session_id: Uuid::new_v4(),
            user_id,
            created_at: now,
            expires_at,
            created_from_ip: login_ip,
            user_agent,
            is_active: true,
            last_accessed_at: now,
        }
    }

    /// Create a session from a login event
    pub fn from_login_event(login_event: &UserLoggedIn) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::days(7); // 7 days expiry

        // Parse session_id from the event if available
        let session_id = login_event
            .session_id
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::new_v4);

        Self {
            session_id,
            user_id: login_event.user_id,
            created_at: now,
            expires_at,
            created_from_ip: login_event.login_ip.clone(),
            user_agent: login_event.user_agent.clone(),
            is_active: true,
            last_accessed_at: now,
        }
    }

    /// Check if the session is valid (not expired and active)
    pub fn is_valid(&self) -> bool {
        self.is_active && Utc::now() < self.expires_at
    }

    /// Mark session as accessed (for extending expiry)
    pub fn touch(&mut self) {
        self.last_accessed_at = Utc::now();
    }

    /// Invalidate the session
    pub fn invalidate(&mut self) {
        self.is_active = false;
    }
}

/// Session-related errors
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session has expired")]
    SessionExpired,

    #[error("Session is not active")]
    SessionInactive,

    #[error("Invalid session ID")]
    InvalidSessionId,

    #[error("Event processing error: {0}")]
    EventProcessingError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UserLoggedIn;

    #[test]
    fn test_session_creation() {
        let user_id = Uuid::new_v4();
        let session = Session::new(
            user_id,
            Some("192.168.1.1".to_string()),
            Some("test-agent".to_string()),
        );

        assert_eq!(session.user_id, user_id);
        assert!(session.is_active);
        assert!(session.is_valid());
        assert_eq!(session.created_from_ip, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_session_invalidation() {
        let user_id = Uuid::new_v4();
        let mut session = Session::new(user_id, None, None);

        assert!(session.is_valid());

        session.invalidate();
        assert!(!session.is_valid());
        assert!(!session.is_active);
    }

    #[test]
    fn test_session_from_login_event() {
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        let login_event = UserLoggedIn::new(
            user_id,
            Some("192.168.1.1".to_string()),
            Some("test-agent".to_string()),
            Some(session_id.to_string()),
        );

        let session = Session::from_login_event(&login_event);

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.session_id, session_id);
        assert!(session.is_active);
        assert!(session.is_valid());
    }
}
