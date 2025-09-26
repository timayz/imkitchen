// Recipe domain logic and aggregates

use chrono::{DateTime, Utc};
use imkitchen_shared::Difficulty;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Recipe aggregate root
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Recipe {
    pub recipe_id: Uuid,
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
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Ingredient {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub quantity: String,
    #[validate(length(min = 1))]
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Instruction {
    pub step_number: u32,
    #[validate(length(min = 1))]
    pub description: String,
}