// CQRS commands for user operations

pub mod email_validation;
pub mod email_validation_handlers;
pub mod register_user;
pub mod reset_password;

use imkitchen_shared::{Email, FamilySize, SkillLevel};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// Re-export email validation commands and handlers
pub use email_validation::*;
pub use email_validation_handlers::*;

// Re-export complex process commands
pub use register_user::*;
pub use reset_password::*;

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
