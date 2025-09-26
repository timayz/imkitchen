// CQRS commands for recipe operations

use crate::domain::{Ingredient, Instruction};
use imkitchen_shared::Difficulty;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRecipeCommand {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1))]
    pub ingredients: Vec<Ingredient>,
    #[validate(length(min = 1))]
    pub instructions: Vec<Instruction>,
    #[validate(range(min = 1))]
    pub prep_time_minutes: u32,
    #[validate(range(min = 1))]
    pub cook_time_minutes: u32,
    pub difficulty: Difficulty,
    pub created_by: Uuid,
}
