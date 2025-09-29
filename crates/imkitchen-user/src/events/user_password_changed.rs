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
    
    /// IP address from which password change occurred (for security audit)
    pub change_ip: Option<String>,
    
    /// User agent string (for security audit)
    pub user_agent: Option<String>,
    
    /// Reason for password change (reset, voluntary, forced)
    pub change_reason: PasswordChangeReason,
    
    /// Hash of the previous password (for audit purposes, optional)
    pub previous_password_hash: Option<String>,
}

/// Reason why the password was changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PasswordChangeReason {
    /// User voluntarily changed their password
    Voluntary,
    /// Password was reset via email
    Reset,
    /// Password was forced to change by admin
    Forced,
    /// Password expired and required change
    Expired,
}

impl UserPasswordChanged {
    /// Create a new UserPasswordChanged event
    pub fn new(
        user_id: Uuid,
        password_hash: String,
        change_reason: PasswordChangeReason,
        change_ip: Option<String>,
        user_agent: Option<String>,
        previous_password_hash: Option<String>,
    ) -> Self {
        Self {
            user_id,
            password_hash,
            changed_at: Utc::now(),
            change_ip,
            user_agent,
            change_reason,
            previous_password_hash,
        }
    }

    /// Create a basic voluntary password change event
    pub fn voluntary(user_id: Uuid, password_hash: String) -> Self {
        Self::new(
            user_id,
            password_hash,
            PasswordChangeReason::Voluntary,
            None,
            None,
            None,
        )
    }

    /// Create a password reset event
    pub fn reset(user_id: Uuid, password_hash: String, previous_hash: Option<String>) -> Self {
        Self::new(
            user_id,
            password_hash,
            PasswordChangeReason::Reset,
            None,
            None,
            previous_hash,
        )
    }
}
