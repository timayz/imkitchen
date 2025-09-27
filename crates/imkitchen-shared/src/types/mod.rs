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

// Password validation regex: basic pattern (will be enhanced with custom validation)
lazy_static::lazy_static! {
    static ref PASSWORD_REGEX: regex::Regex = regex::Regex::new(
        r"^[A-Za-z\d@$!%*?&]{8,128}$"
    ).unwrap();
}

/// Password value object with strength validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Password {
    #[validate(length(
        min = 8,
        max = 128,
        message = "Password must be between 8 and 128 characters"
    ))]
    #[validate(regex(path = *PASSWORD_REGEX, message = "Password must contain at least one uppercase letter, one lowercase letter, one digit, and one special character"))]
    pub value: String,
}

impl Password {
    pub fn new(password: String) -> Result<Self, validator::ValidationErrors> {
        let pwd = Self { value: password };

        // First run basic validator checks
        pwd.validate()?;

        // Then run custom strength validation
        if !Self::has_required_complexity(&pwd.value) {
            let mut errors = validator::ValidationErrors::new();
            let error = validator::ValidationError::new("password_complexity");
            errors.add("value", error);
            return Err(errors);
        }

        Ok(pwd)
    }

    fn has_required_complexity(password: &str) -> bool {
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| "@$!%*?&".contains(c));

        has_lower && has_upper && has_digit && has_special
    }

    pub fn hash(&self) -> String {
        // TODO: Implement bcrypt hashing in the authentication layer
        format!("HASH:{}", self.value)
    }
}

/// Dietary restriction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DietaryRestriction {
    Vegetarian,
    Vegan,
    GlutenFree,
    DairyFree,
    NutFree,
    SoyFree,
    LowSodium,
    LowCarb,
    Keto,
    Paleo,
}
