// Common value objects and types

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Email address value object with validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Email {
    #[validate(email)]
    pub value: String,
}

impl Email {
    pub fn new(email: String) -> Result<Self, validator::ValidationErrors> {
        let email = Self { value: email };
        email.validate()?;
        Ok(email)
    }
}

/// Family size with validation (1-8 people)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct FamilySize {
    #[validate(range(min = 1, max = 8))]
    pub value: u8,
}

impl FamilySize {
    pub fn new(size: u8) -> Result<Self, validator::ValidationErrors> {
        let family_size = Self { value: size };
        family_size.validate()?;
        Ok(family_size)
    }
}

/// Cooking skill level enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillLevel {
    Beginner,
    Intermediate,
    Advanced,
}

/// Difficulty level for recipes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}