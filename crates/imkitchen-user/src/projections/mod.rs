// Evento projections for user views

use chrono::{DateTime, Utc};
use imkitchen_shared::{Email, FamilySize, SkillLevel};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Read model for user profile view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileView {
    pub user_id: Uuid,
    pub email: String,
    pub family_size: u8,
    pub cooking_skill_level: SkillLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}