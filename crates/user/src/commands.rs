use chrono::Utc;
use evento::Sqlite;
use sqlx::{Row, SqlitePool};
use validator::Validate;

use crate::aggregate::UserAggregate;
use crate::error::{UserError, UserResult};
use crate::events::{
    DietaryRestrictionsSet, HouseholdSizeSet, NotificationPermissionChanged, PasswordChanged,
    ProfileCompleted, ProfileUpdated, SkillLevelSet, SubscriptionUpgraded, UserCreated,
    WeeknightAvailabilitySet,
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

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateProfileCommand {
    pub user_id: String,
    pub dietary_restrictions: Option<Vec<String>>,

    #[validate(range(min = 1, max = 20, message = "Household size must be between 1 and 20"))]
    pub household_size: Option<u8>,

    pub skill_level: Option<String>, // "beginner", "intermediate", "expert"
    pub weeknight_availability: Option<String>, // JSON: {"start":"18:00","duration_minutes":45}
}

/// Update user profile using evento event sourcing pattern
///
/// Supports partial updates - only provided fields (Some) are included in the ProfileUpdated event.
/// Uses COALESCE logic: None fields preserve existing values, Some fields update.
/// Emits ProfileUpdated event with timestamp for audit trail (AC-7).
pub async fn update_profile(command: UpdateProfileCommand, executor: &Sqlite) -> UserResult<()> {
    // Validate command
    command
        .validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    // Validate skill_level enum if provided
    if let Some(ref skill_level) = command.skill_level {
        if !["beginner", "intermediate", "expert"].contains(&skill_level.as_str()) {
            return Err(UserError::ValidationError(
                "skill_level must be one of: beginner, intermediate, expert".to_string(),
            ));
        }
    }

    let updated_at = Utc::now();

    // Emit ProfileUpdated event with only changed fields
    evento::save::<UserAggregate>(command.user_id.clone())
        .data(&ProfileUpdated {
            dietary_restrictions: command.dietary_restrictions,
            household_size: command.household_size,
            skill_level: command.skill_level,
            weeknight_availability: command.weeknight_availability,
            updated_at: updated_at.to_rfc3339(),
        })
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| UserError::EventStoreError(e.to_string()))?;

    Ok(())
}

/// Validate whether a user can create a new recipe based on tier and recipe_count
///
/// This function enforces the freemium tier limit: free users are limited to 10 recipes maximum.
/// Premium users have unlimited recipe creation.
///
/// Returns Ok(()) if user can create a recipe (premium or under limit)
/// Returns Err(UserError::RecipeLimitReached) if free user has reached the 10 recipe limit
///
/// This validation should be called BEFORE emitting RecipeCreated event in the recipe domain.
pub async fn validate_recipe_creation(user_id: &str, pool: &SqlitePool) -> UserResult<()> {
    // Query user tier and recipe_count from read model
    let result = sqlx::query("SELECT tier, recipe_count FROM users WHERE id = ?1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    match result {
        Some(row) => {
            let tier: String = row.get("tier");
            let recipe_count: i32 = row.get("recipe_count");

            // Premium users bypass all limits
            if tier == "premium" {
                return Ok(());
            }

            // Free tier users limited to 10 recipes
            if tier == "free" && recipe_count >= 10 {
                return Err(UserError::RecipeLimitReached);
            }

            Ok(())
        }
        None => {
            // User not found - should not happen in normal flow
            Err(UserError::ValidationError("User not found".to_string()))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpgradeSubscriptionCommand {
    pub user_id: String,
    pub new_tier: String, // "free" or "premium"
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
}

/// Upgrade user subscription tier via evento event sourcing pattern
///
/// This command is invoked by the Stripe webhook handler after successful payment.
/// 1. Emits SubscriptionUpgraded event with tier change and Stripe metadata
/// 2. evento::save() automatically loads the aggregate before appending the event
/// 3. Event automatically projected to read model via async subscription handler
///
/// The SubscriptionUpgraded event stores Stripe Customer ID and Subscription ID
/// for future subscription management (cancellation, billing updates).
pub async fn upgrade_subscription(
    command: UpgradeSubscriptionCommand,
    executor: &Sqlite,
) -> UserResult<()> {
    // Validate command
    command
        .validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    let upgraded_at = Utc::now();

    // Emit SubscriptionUpgraded event
    // evento::save() automatically loads the aggregate before appending the event
    evento::save::<UserAggregate>(command.user_id.clone())
        .data(&SubscriptionUpgraded {
            new_tier: command.new_tier,
            stripe_customer_id: command.stripe_customer_id,
            stripe_subscription_id: command.stripe_subscription_id,
            upgraded_at: upgraded_at.to_rfc3339(),
        })
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| UserError::EventStoreError(e.to_string()))?;

    Ok(())
}

/// Command to change notification permission status
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ChangeNotificationPermissionCommand {
    #[validate(length(min = 1))]
    pub user_id: String,

    #[validate(custom(function = "validate_permission_status"))]
    pub permission_status: String, // "granted", "denied", "skipped"
}

/// Validate permission_status is one of allowed values
fn validate_permission_status(status: &str) -> Result<(), validator::ValidationError> {
    if !["granted", "denied", "skipped"].contains(&status) {
        return Err(validator::ValidationError::new("invalid_permission_status"));
    }
    Ok(())
}

/// Change notification permission status via evento event sourcing pattern
///
/// AC #3, #5, #8: Track user's permission decision and denial timestamp for grace period
///
/// This command:
/// 1. Validates permission_status is "granted", "denied", or "skipped"
/// 2. Emits NotificationPermissionChanged event
/// 3. If denied, stores timestamp for 30-day grace period tracking
/// 4. Event automatically projected to read model via async subscription handler
pub async fn change_notification_permission(
    command: ChangeNotificationPermissionCommand,
    executor: &Sqlite,
) -> UserResult<()> {
    // Validate command
    command
        .validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    let changed_at = Utc::now();

    // Calculate denial timestamp (only if status is "denied")
    let last_permission_denial_at = if command.permission_status == "denied" {
        Some(changed_at.to_rfc3339())
    } else {
        None
    };

    // Emit NotificationPermissionChanged event
    // evento::save() automatically loads the aggregate before appending the event
    evento::save::<UserAggregate>(command.user_id.clone())
        .data(&NotificationPermissionChanged {
            permission_status: command.permission_status,
            last_permission_denial_at,
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
