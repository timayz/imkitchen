use serde::{Deserialize, Serialize};
use validator::Validate;

// Re-export Difficulty from shared crate
pub use imkitchen_shared::Difficulty;

/// Ingredient value object with quantity validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Ingredient {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(range(min = 0.01))]
    pub quantity: f64,
    #[validate(length(min = 1))]
    pub unit: String,
    pub notes: Option<String>,
}

impl Ingredient {
    pub fn new(
        name: String,
        quantity: f64,
        unit: String,
        notes: Option<String>,
    ) -> Result<Self, validator::ValidationErrors> {
        let ingredient = Self {
            name,
            quantity,
            unit,
            notes,
        };

        ingredient.validate()?;
        Ok(ingredient)
    }
}

/// Instruction value object with step validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Instruction {
    pub step_number: u32,
    #[validate(length(min = 1))]
    #[validate(custom(function = "validate_non_whitespace"))]
    pub text: String,
    pub estimated_minutes: Option<u32>,
}

impl Instruction {
    pub fn new(
        step_number: u32,
        text: String,
        estimated_minutes: Option<u32>,
    ) -> Result<Self, validator::ValidationErrors> {
        let instruction = Self {
            step_number,
            text,
            estimated_minutes,
        };

        instruction.validate()?;
        Ok(instruction)
    }
}

/// Recipe category enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RecipeCategory {
    Appetizer,
    Main,
    Dessert,
    Beverage,
    Bread,
    Soup,
    Salad,
}

impl std::fmt::Display for RecipeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecipeCategory::Appetizer => write!(f, "Appetizer"),
            RecipeCategory::Main => write!(f, "Main"),
            RecipeCategory::Dessert => write!(f, "Dessert"),
            RecipeCategory::Beverage => write!(f, "Beverage"),
            RecipeCategory::Bread => write!(f, "Bread"),
            RecipeCategory::Soup => write!(f, "Soup"),
            RecipeCategory::Salad => write!(f, "Salad"),
        }
    }
}

/// Nutritional information value object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutritionalInfo {
    pub calories: f64,
    pub protein: f64,       // grams
    pub carbohydrates: f64, // grams
    pub fat: f64,           // grams
    pub fiber: f64,         // grams
    pub sugar: f64,         // grams
    pub sodium: f64,        // milligrams
}

impl Default for NutritionalInfo {
    fn default() -> Self {
        Self {
            calories: 0.0,
            protein: 0.0,
            carbohydrates: 0.0,
            fat: 0.0,
            fiber: 0.0,
            sugar: 0.0,
            sodium: 0.0,
        }
    }
}

// Custom validator function for non-whitespace text
fn validate_non_whitespace(text: &str) -> Result<(), validator::ValidationError> {
    if text.trim().is_empty() {
        return Err(validator::ValidationError::new("text_empty_or_whitespace"));
    }
    Ok(())
}
