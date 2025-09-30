use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::value_objects::{Difficulty, Ingredient, Instruction, RecipeCategory};

/// Parameters for creating a new recipe
#[derive(Debug, Clone)]
pub struct RecipeParams {
    pub title: String,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
    pub prep_time_minutes: u32,
    pub cook_time_minutes: u32,
    pub difficulty: Difficulty,
    pub category: RecipeCategory,
    pub created_by: Uuid,
    pub is_public: bool,
    pub tags: Vec<String>,
}

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
    pub category: RecipeCategory,
    #[validate(range(min = 0.0, max = 5.0))]
    pub rating: f32,
    pub review_count: u32,
    pub created_by: Uuid,
    pub is_public: bool,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl Recipe {
    pub fn new(params: RecipeParams) -> Result<Self, validator::ValidationErrors> {
        let recipe = Self {
            recipe_id: Uuid::new_v4(),
            title: params.title,
            ingredients: params.ingredients,
            instructions: params.instructions,
            prep_time_minutes: params.prep_time_minutes,
            cook_time_minutes: params.cook_time_minutes,
            difficulty: params.difficulty,
            category: params.category,
            rating: 0.0,
            review_count: 0,
            created_by: params.created_by,
            is_public: params.is_public,
            tags: params.tags,
            created_at: Utc::now(),
        };

        recipe.validate()?;
        Ok(recipe)
    }

    pub fn total_time_minutes(&self) -> u32 {
        self.prep_time_minutes + self.cook_time_minutes
    }

    pub fn ingredient_count(&self) -> usize {
        self.ingredients.len()
    }

    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }
}
