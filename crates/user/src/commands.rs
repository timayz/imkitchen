use chrono::Utc;
use evento::Sqlite;
use sqlx::SqlitePool;
use validator::Validate;

use crate::aggregate::UserAggregate;
use crate::error::{UserError, UserResult};
use crate::events::{
    DietaryRestrictionsSet, HouseholdSizeSet, PasswordChanged, ProfileCompleted, SkillLevelSet,
    UserCreated, WeeknightAvailabilitySet,
};
use crate::password::hash_password;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterUserCommand {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

/// Register a new user using evento event sourcing pattern with consistent email validation
///
/// 1. Validates email format and password length
/// 2. Checks email uniqueness via dedicated user_email_uniqueness table (consistent check)
/// 3. Hashes password with Argon2
/// 4. Inserts into user_email_uniqueness table within transaction
/// 5. Creates and commits UserCreated event to evento event store
/// 6. Event automatically projected to read model via async subscription handler
/// 7. Returns user ID (evento aggregator_id)
///
/// Consistency guarantee: Email uniqueness is enforced via UNIQUE constraint on
/// user_email_uniqueness table, preventing race conditions.
pub async fn register_user(
    command: RegisterUserCommand,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> UserResult<String> {
    // Validate command
    command
        .validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    // Hash password using Argon2id with OWASP parameters
    let password_hash = hash_password(&command.password)?;

    // Generate user ID
    let user_id = Uuid::new_v4();
    let created_at = Utc::now();

    // Check and insert into user_email_uniqueness table
    // This enforces email uniqueness at database level with UNIQUE constraint
    // This is NOT the read model - it's a write-side uniqueness constraint table
    let insert_result = sqlx::query(
        "INSERT INTO user_email_uniqueness (email, user_id, created_at) VALUES (?, ?, ?)",
    )
    .bind(&command.email)
    .bind(user_id.to_string())
    .bind(created_at.to_rfc3339())
    .execute(pool)
    .await;

    match insert_result {
        Ok(_) => {
            // Email is unique, continue
        }
        Err(e) => {
            // Check if it's a unique constraint violation
            if let Some(db_err) = e.as_database_error() {
                let is_unique_violation = if let Some(code) = db_err.code() {
                    code.as_ref() == "2067" || code.as_ref() == "1555"
                } else {
                    false
                };

                if is_unique_violation || db_err.message().contains("UNIQUE constraint failed") {
                    return Err(UserError::EmailAlreadyExists);
                }
            }
            return Err(UserError::DatabaseError(e));
        }
    }

    // Create UserCreated event and commit to evento event store
    // The async subscription handler (on_user_created) will project to read model
    // evento::create() generates a ULID for the aggregator_id
    let aggregator_id = evento::create::<UserAggregate>()
        .data(&UserCreated {
            email: command.email,
            password_hash,
            created_at: created_at.to_rfc3339(),
        })
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| UserError::EventStoreError(e.to_string()))?;

    // Return the generated aggregator_id as the user_id
    Ok(aggregator_id)
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResetPasswordCommand {
    pub user_id: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub new_password: String,
}

/// Reset user password using evento event sourcing pattern
///
/// 1. Validates password length
/// 2. Hashes new password with Argon2
/// 3. Creates and commits PasswordChanged event to evento event store
/// 4. Event automatically projected to read model via async subscription handler
///
/// This invalidates the old password and all existing JWT sessions (stateless JWT limitation)
pub async fn reset_password(command: ResetPasswordCommand, executor: &Sqlite) -> UserResult<()> {
    // Validate command
    command
        .validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    // Hash new password using Argon2id with OWASP parameters
    let password_hash = hash_password(&command.new_password)?;

    let changed_at = Utc::now();

    // Create PasswordChanged event and commit to evento event store
    // evento::save() automatically loads the aggregate before appending the event
    // The async subscription handler (password_changed_handler) will project to read model
    evento::save::<UserAggregate>(command.user_id.clone())
        .data(&PasswordChanged {
            password_hash,
            changed_at: changed_at.to_rfc3339(),
        })
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| UserError::EventStoreError(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetDietaryRestrictionsCommand {
    pub user_id: String,
    pub dietary_restrictions: Vec<String>, // e.g., ["vegetarian", "peanuts"]
}

pub async fn set_dietary_restrictions(
    command: SetDietaryRestrictionsCommand,
    executor: &Sqlite,
) -> UserResult<()> {
    let set_at = Utc::now();

    evento::save::<UserAggregate>(command.user_id.clone())
        .data(&DietaryRestrictionsSet {
            dietary_restrictions: command.dietary_restrictions,
            set_at: set_at.to_rfc3339(),
        })
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| UserError::EventStoreError(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SetHouseholdSizeCommand {
    pub user_id: String,

    #[validate(range(min = 1, max = 10, message = "Household size must be between 1 and 10"))]
    pub household_size: u8,
}

pub async fn set_household_size(
    command: SetHouseholdSizeCommand,
    executor: &Sqlite,
) -> UserResult<()> {
    command
        .validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    let set_at = Utc::now();

    evento::save::<UserAggregate>(command.user_id.clone())
        .data(&HouseholdSizeSet {
            household_size: command.household_size,
            set_at: set_at.to_rfc3339(),
        })
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| UserError::EventStoreError(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSkillLevelCommand {
    pub user_id: String,
    pub skill_level: String, // "beginner", "intermediate", "expert"
}

pub async fn set_skill_level(command: SetSkillLevelCommand, executor: &Sqlite) -> UserResult<()> {
    let set_at = Utc::now();

    evento::save::<UserAggregate>(command.user_id.clone())
        .data(&SkillLevelSet {
            skill_level: command.skill_level,
            set_at: set_at.to_rfc3339(),
        })
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| UserError::EventStoreError(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetWeeknightAvailabilityCommand {
    pub user_id: String,
    pub weeknight_availability: String, // JSON: {"start":"18:00","duration_minutes":45}
}

pub async fn set_weeknight_availability(
    command: SetWeeknightAvailabilityCommand,
    executor: &Sqlite,
) -> UserResult<()> {
    let set_at = Utc::now();

    evento::save::<UserAggregate>(command.user_id.clone())
        .data(&WeeknightAvailabilitySet {
            weeknight_availability: command.weeknight_availability,
            set_at: set_at.to_rfc3339(),
        })
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| UserError::EventStoreError(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteProfileCommand {
    pub user_id: String,
}

/// Complete user profile (mark onboarding as done)
///
/// This should be called AFTER all individual step events have been emitted.
/// It simply marks the onboarding process as completed.
pub async fn complete_profile(
    command: CompleteProfileCommand,
    executor: &Sqlite,
) -> UserResult<()> {
    let completed_at = Utc::now();

    evento::save::<UserAggregate>(command.user_id.clone())
        .data(&ProfileCompleted {
            completed_at: completed_at.to_rfc3339(),
        })
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| UserError::EventStoreError(e.to_string()))?;

    Ok(())
}
