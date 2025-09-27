// User aggregate with Evento event sourcing implementation

use chrono::{DateTime, Utc};
use imkitchen_shared::{Email, Password, FamilySize, SkillLevel, DietaryRestriction};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::events::{UserLoggedIn, UserPasswordChanged, UserProfileUpdated};

/// User aggregate root with Evento event sourcing
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct User {
    pub user_id: Uuid,
    pub email: Email,
    pub password_hash: String,
    pub profile: UserProfile,
    pub is_email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for User {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            user_id: Uuid::new_v4(),
            email: Email::new("default@example.com".to_string()).unwrap(),
            password_hash: String::new(),
            profile: UserProfile::default(),
            is_email_verified: false,
            created_at: now,
            updated_at: now,
        }
    }
}

/// User profile containing meal planning preferences
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserProfile {
    pub family_size: FamilySize,
    pub cooking_skill_level: SkillLevel,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub weekday_cooking_minutes: u32,
    pub weekend_cooking_minutes: u32,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            family_size: FamilySize::new(2).unwrap(), // Default to 2 people
            cooking_skill_level: SkillLevel::Beginner,
            dietary_restrictions: vec![],
            weekday_cooking_minutes: 30,
            weekend_cooking_minutes: 60,
        }
    }
}

// Core User domain methods
impl User {
    /// Create a new User - this will later be integrated with Evento
    pub fn new(
        email: Email,
        password: Password,
        profile: UserProfile,
    ) -> Self {
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        
        Self {
            user_id,
            email,
            password_hash: password.hash(),
            profile,
            is_email_verified: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Record user login
    pub fn login(&self) -> Result<UserLoggedIn, UserError> {
        if !self.is_email_verified {
            return Err(UserError::EmailNotVerified);
        }

        Ok(UserLoggedIn {
            user_id: self.user_id,
            logged_in_at: Utc::now(),
        })
    }

    /// Change user password
    pub fn change_password(&mut self, new_password: Password) -> UserPasswordChanged {
        self.password_hash = new_password.hash();
        self.updated_at = Utc::now();
        
        UserPasswordChanged {
            user_id: self.user_id,
            password_hash: self.password_hash.clone(),
            changed_at: self.updated_at,
        }
    }

    /// Update user profile
    pub fn update_profile(&mut self, new_profile: UserProfile) -> UserProfileUpdated {
        self.profile = new_profile.clone();
        self.updated_at = Utc::now();
        
        UserProfileUpdated {
            user_id: self.user_id,
            profile: new_profile,
            updated_at: self.updated_at,
        }
    }
    
    /// Verify user email
    pub fn verify_email(&mut self) {
        self.is_email_verified = true;
        self.updated_at = Utc::now();
    }
}

// Error types for User domain
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("Email address has not been verified")]
    EmailNotVerified,
    
    #[error("User not found")]
    NotFound,
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Email already exists")]
    EmailAlreadyExists,
    
    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
}