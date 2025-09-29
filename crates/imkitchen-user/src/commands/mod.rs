// CQRS commands for user operations

pub mod delete_user_account;
pub mod email_validation;
pub mod email_validation_handlers;
pub mod register_user;
pub mod reset_password;
pub mod update_user_profile;

use imkitchen_shared::{Email, FamilySize, SkillLevel};
use serde::{Deserialize, Serialize};
use validator::Validate;

// Re-export email validation commands and handlers
pub use email_validation::*;
pub use email_validation_handlers::*;

// Re-export complex process commands
pub use delete_user_account::*;
pub use register_user::*;
pub use reset_password::*;
pub use update_user_profile::*;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterUserCommand {
    pub email: Email,
    pub family_size: FamilySize,
    pub cooking_skill_level: SkillLevel,
}
