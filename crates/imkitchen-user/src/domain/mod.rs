// User domain logic and aggregates

use chrono::{DateTime, Utc};
use imkitchen_shared::{Email, FamilySize, SkillLevel};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// User profile aggregate root
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub email: Email,
    pub family_size: FamilySize,
    pub cooking_skill_level: SkillLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserProfile {
    pub fn new(email: Email, family_size: FamilySize, skill_level: SkillLevel) -> Self {
        let now = Utc::now();
        Self {
            user_id: Uuid::new_v4(),
            email,
            family_size,
            cooking_skill_level: skill_level,
            created_at: now,
            updated_at: now,
        }
    }
}