// CQRS commands for user operations

use imkitchen_shared::{Email, FamilySize, SkillLevel};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterUserCommand {
    pub email: Email,
    pub family_size: FamilySize,
    pub cooking_skill_level: SkillLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUserProfileCommand {
    pub user_id: Uuid,
    pub family_size: Option<FamilySize>,
    pub cooking_skill_level: Option<SkillLevel>,
}
