// User aggregate with Evento event sourcing implementation

use chrono::{DateTime, Utc};
use imkitchen_shared::{DietaryRestriction, Email, FamilySize, Password, SkillLevel};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::events::{
    PasswordChangeReason, UserLoggedIn, UserPasswordChanged, UserProfileUpdated, UserRegistered,
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
