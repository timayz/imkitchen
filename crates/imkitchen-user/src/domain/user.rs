// User aggregate with Evento event sourcing implementation

use chrono::{DateTime, Utc};
use imkitchen_shared::{DietaryRestriction, Email, FamilySize, Password, SkillLevel};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::events::{
    DietaryRestrictionsChanged, FamilySizeChanged, PasswordChangeReason, UserLoggedIn,
    UserPasswordChanged, UserProfileUpdated, UserRegistered,
};

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
    /// Create a new User and return both the user and the registration event
    pub fn new(email: Email, password: Password, profile: UserProfile) -> (Self, UserRegistered) {
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        let password_hash = password.hash();

        let user = Self {
            user_id,
            email: email.clone(),
            password_hash: password_hash.clone(),
            profile: profile.clone(),
            is_email_verified: false,
            created_at: now,
            updated_at: now,
        };

        let event = UserRegistered::basic(user_id, email, password_hash, profile);

        (user, event)
    }

    /// Create a new User with registration context (IP, user agent)
    pub fn new_with_context(
        email: Email,
        password: Password,
        profile: UserProfile,
        registration_ip: Option<String>,
        user_agent: Option<String>,
    ) -> (Self, UserRegistered) {
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        let password_hash = password.hash();

        let user = Self {
            user_id,
            email: email.clone(),
            password_hash: password_hash.clone(),
            profile: profile.clone(),
            is_email_verified: false,
            created_at: now,
            updated_at: now,
        };

        let event = UserRegistered::new(
            user_id,
            email,
            password_hash,
            profile,
            registration_ip,
            user_agent,
        );

        (user, event)
    }

    /// Record user login
    pub fn login(&self) -> Result<UserLoggedIn, UserError> {
        if !self.is_email_verified {
            return Err(UserError::EmailNotVerified);
        }

        Ok(UserLoggedIn::basic(self.user_id))
    }

    /// Record user login with context (IP, user agent, session)
    pub fn login_with_context(
        &self,
        login_ip: Option<String>,
        user_agent: Option<String>,
        session_id: Option<String>,
    ) -> Result<UserLoggedIn, UserError> {
        if !self.is_email_verified {
            return Err(UserError::EmailNotVerified);
        }

        Ok(UserLoggedIn::new(
            self.user_id,
            login_ip,
            user_agent,
            session_id,
        ))
    }

    /// Change user password (voluntary change)
    pub fn change_password(&mut self, new_password: Password) -> UserPasswordChanged {
        let previous_hash = Some(self.password_hash.clone());
        self.password_hash = new_password.hash();
        self.updated_at = Utc::now();

        UserPasswordChanged::new(
            self.user_id,
            self.password_hash.clone(),
            PasswordChangeReason::Voluntary,
            None,
            None,
            previous_hash,
        )
    }

    /// Change user password with context and reason
    pub fn change_password_with_context(
        &mut self,
        new_password: Password,
        reason: PasswordChangeReason,
        change_ip: Option<String>,
        user_agent: Option<String>,
    ) -> UserPasswordChanged {
        let previous_hash = Some(self.password_hash.clone());
        self.password_hash = new_password.hash();
        self.updated_at = Utc::now();

        UserPasswordChanged::new(
            self.user_id,
            self.password_hash.clone(),
            reason,
            change_ip,
            user_agent,
            previous_hash,
        )
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

    /// Update dietary restrictions specifically with dedicated event
    pub fn update_dietary_restrictions(
        &mut self,
        restrictions: Vec<DietaryRestriction>,
    ) -> DietaryRestrictionsChanged {
        let previous_restrictions = self.profile.dietary_restrictions.clone();
        self.profile.dietary_restrictions = restrictions.clone();
        self.updated_at = Utc::now();

        DietaryRestrictionsChanged::new(self.user_id, previous_restrictions, restrictions)
    }

    /// Update family size specifically with dedicated event
    pub fn update_family_size(&mut self, family_size: FamilySize) -> FamilySizeChanged {
        let previous_size = self.profile.family_size;
        self.profile.family_size = family_size;
        self.updated_at = Utc::now();

        FamilySizeChanged::new(self.user_id, previous_size, family_size)
    }

    /// Update cooking skill level specifically
    pub fn update_skill_level(&mut self, skill_level: SkillLevel) -> UserProfileUpdated {
        self.profile.cooking_skill_level = skill_level;
        self.updated_at = Utc::now();

        UserProfileUpdated {
            user_id: self.user_id,
            profile: self.profile.clone(),
            updated_at: self.updated_at,
        }
    }

    /// Update cooking time preferences
    pub fn update_cooking_time(
        &mut self,
        weekday_minutes: u32,
        weekend_minutes: u32,
    ) -> UserProfileUpdated {
        self.profile.weekday_cooking_minutes = weekday_minutes;
        self.profile.weekend_cooking_minutes = weekend_minutes;
        self.updated_at = Utc::now();

        UserProfileUpdated {
            user_id: self.user_id,
            profile: self.profile.clone(),
            updated_at: self.updated_at,
        }
    }

    /// Verify user email
    pub fn verify_email(&mut self) {
        self.is_email_verified = true;
        self.updated_at = Utc::now();
    }
}

// UserProfile domain methods
impl UserProfile {
    /// Create a new UserProfile with validation
    pub fn new(
        family_size: FamilySize,
        cooking_skill_level: SkillLevel,
        dietary_restrictions: Vec<DietaryRestriction>,
        weekday_cooking_minutes: u32,
        weekend_cooking_minutes: u32,
    ) -> Result<Self, UserError> {
        let profile = Self {
            family_size,
            cooking_skill_level,
            dietary_restrictions,
            weekday_cooking_minutes,
            weekend_cooking_minutes,
        };

        // Validate the profile
        profile.validate()?;
        Ok(profile)
    }

    /// Check if profile is complete for meal planning
    pub fn is_complete_for_meal_planning(&self) -> bool {
        // Family size is always valid due to type safety
        // At least one dietary preference should be specified (even if empty vector is valid)
        // Cooking times should be reasonable (at least 5 minutes)
        self.weekday_cooking_minutes >= 5 && self.weekend_cooking_minutes >= 5
    }

    /// Get recommended recipe complexity based on skill level
    pub fn get_recommended_complexity(&self) -> Vec<String> {
        match self.cooking_skill_level {
            SkillLevel::Beginner => vec!["Easy".to_string()],
            SkillLevel::Intermediate => vec!["Easy".to_string(), "Medium".to_string()],
            SkillLevel::Advanced => {
                vec!["Easy".to_string(), "Medium".to_string(), "Hard".to_string()]
            }
        }
    }

    /// Calculate portions needed based on family size
    pub fn calculate_portions(&self, base_portions: u8) -> u8 {
        let multiplier = self.family_size.value as f32 / base_portions as f32;
        (base_portions as f32 * multiplier).ceil() as u8
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

    #[error("Database error: {0}")]
    DatabaseError(String),
}
