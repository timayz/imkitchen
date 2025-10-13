use chrono::Utc;
use evento::Sqlite;
use sqlx::SqlitePool;
use uuid::Uuid;
use validator::Validate;

use crate::aggregate::UserAggregate;
use crate::error::{UserError, UserResult};
use crate::events::{PasswordChanged, UserCreated};
use crate::password::hash_password;
use serde::{Deserialize, Serialize};

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

    // Start transaction for consistent email validation
    let mut tx = pool.begin().await.map_err(UserError::DatabaseError)?;

    // Check and insert into user_email_uniqueness table
    // This enforces email uniqueness at database level with UNIQUE constraint
    let insert_result = sqlx::query(
        "INSERT INTO user_email_uniqueness (email, user_id, created_at) VALUES (?, ?, ?)",
    )
    .bind(&command.email)
    .bind(user_id.to_string())
    .bind(created_at.to_rfc3339())
    .execute(&mut *tx)
    .await;

    match insert_result {
        Ok(_) => {
            // Email is unique, commit transaction
            tx.commit().await.map_err(UserError::DatabaseError)?;
        }
        Err(e) => {
            // Check if it's a unique constraint violation
            // SQLite UNIQUE constraint violations can have multiple error codes
            if let Some(db_err) = e.as_database_error() {
                // Check both error code and message for UNIQUE constraint violations
                let is_unique_violation = if let Some(code) = db_err.code() {
                    // SQLite error code 2067 = SQLITE_CONSTRAINT_UNIQUE
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
    let aggregator_id = evento::create::<UserAggregate>()
        .data(&UserCreated {
            user_id: user_id.to_string(),
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
pub async fn reset_password(
    command: ResetPasswordCommand,
    executor: &Sqlite,
) -> UserResult<()> {
    // Validate command
    command
        .validate()
        .map_err(|e| UserError::ValidationError(e.to_string()))?;

    // Hash new password using Argon2id with OWASP parameters
    let password_hash = hash_password(&command.new_password)?;

    let changed_at = Utc::now();

    // Create PasswordChanged event and commit to evento event store
    // The async subscription handler (password_changed_handler) will project to read model
    evento::save::<UserAggregate>(command.user_id.clone())
        .data(&PasswordChanged {
            user_id: command.user_id.clone(),
            password_hash,
            changed_at: changed_at.to_rfc3339(),
        })
        .map_err(|e| UserError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| UserError::EventStoreError(e.to_string()))?;

    Ok(())
}
