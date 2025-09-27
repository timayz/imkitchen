// UserProfileUpdated event for profile modification tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::UserProfile;

/// Event fired when a user updates their profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileUpdated {
    pub user_id: Uuid,
    pub profile: UserProfile,
    pub updated_at: DateTime<Utc>,
}
