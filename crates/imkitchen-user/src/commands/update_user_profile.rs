// Profile update commands with validation middleware and command bus integration

use chrono::{DateTime, Utc};
use imkitchen_shared::{DietaryRestriction, FamilySize, SkillLevel};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;
use validator::Validate;

use crate::{
    domain::{User, UserError, UserProfile},
    event_store::EventStore,
};

/// Command to update user profile with validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUserProfileCommand {
    pub user_id: Uuid,
    pub family_size: FamilySize,
    pub cooking_skill_level: SkillLevel,
    #[validate(range(
        min = 5,
        max = 480,
        message = "Weekday cooking time must be between 5 and 480 minutes"
    ))]
    pub weekday_cooking_minutes: u32,
    #[validate(range(
        min = 5,
        max = 480,
        message = "Weekend cooking time must be between 5 and 480 minutes"
    ))]
    pub weekend_cooking_minutes: u32,
}

/// Command to change dietary restrictions specifically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDietaryRestrictionsCommand {
    pub user_id: Uuid,
    pub new_restrictions: Vec<DietaryRestriction>,
}

/// Profile update command handler with database operations
#[derive(Debug, Clone)]
pub struct ProfileCommandHandler {
    event_store: EventStore,
    db_pool: SqlitePool,
}

/// Response from profile update operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUpdateResponse {
    pub success: bool,
    pub user_id: Uuid,
    pub updated_at: DateTime<Utc>,
    pub events_stored: u32,
    pub message: String,
}

impl UpdateUserProfileCommand {
    pub fn new(
        user_id: Uuid,
        family_size: FamilySize,
        cooking_skill_level: SkillLevel,
        weekday_cooking_minutes: u32,
        weekend_cooking_minutes: u32,
    ) -> Self {
        Self {
            user_id,
            family_size,
            cooking_skill_level,
            weekday_cooking_minutes,
            weekend_cooking_minutes,
        }
    }

    /// Validate the command using the validation middleware
    pub fn validate_command(&self) -> Result<(), UserError> {
        self.validate()?;

        // Additional business rule validation
        if self.weekday_cooking_minutes > self.weekend_cooking_minutes {
            return Err(UserError::DatabaseError(
                "Weekday cooking time cannot exceed weekend cooking time".to_string(),
            ));
        }

        Ok(())
    }
}

impl ChangeDietaryRestrictionsCommand {
    pub fn new(user_id: Uuid, new_restrictions: Vec<DietaryRestriction>) -> Self {
        Self {
            user_id,
            new_restrictions,
        }
    }

    /// Validate dietary restrictions for conflicts
    pub fn validate_command(&self) -> Result<(), UserError> {
        // Check for conflicting restrictions
        if self
            .new_restrictions
            .contains(&DietaryRestriction::Vegetarian)
            && self.new_restrictions.contains(&DietaryRestriction::Vegan)
        {
            return Err(UserError::DatabaseError(
                "Cannot be both Vegetarian and Vegan - Vegan includes Vegetarian".to_string(),
            ));
        }

        // Check for excessive restrictions (more than 5 might be impractical)
        if self.new_restrictions.len() > 5 {
            return Err(UserError::DatabaseError(
                "Too many dietary restrictions selected (maximum 5)".to_string(),
            ));
        }

        Ok(())
    }
}

impl ProfileCommandHandler {
    pub fn new(db_pool: SqlitePool) -> Self {
        let event_store = EventStore::new(db_pool.clone());
        Self {
            event_store,
            db_pool,
        }
    }

    /// Handle profile update command with transaction support
    pub async fn handle_update_profile(
        &self,
        command: UpdateUserProfileCommand,
    ) -> Result<ProfileUpdateResponse, UserError> {
        // Validation middleware
        command.validate_command()?;

        // Begin database transaction
        let _tx =
            self.db_pool.begin().await.map_err(|e| {
                UserError::DatabaseError(format!("Database transaction error: {}", e))
            })?;

        // Load current user profile
        let current_user = self.load_user_profile(command.user_id).await?;
        let mut user = current_user.clone();

        // Validate new profile structure (not used directly but validates constraints)
        let _new_profile = UserProfile::new(
            command.family_size,
            command.cooking_skill_level,
            current_user.profile.dietary_restrictions.clone(), // Keep existing restrictions
            command.weekday_cooking_minutes,
            command.weekend_cooking_minutes,
        )?;

        // Update user profile and generate events
        let mut events_stored = 0;

        // Check for family size change
        if user.profile.family_size != command.family_size {
            let family_size_event = user.update_family_size(command.family_size);
            self.event_store
                .store_event(&family_size_event)
                .await
                .map_err(|e| {
                    UserError::DatabaseError(format!("Failed to store family size event: {}", e))
                })?;
            events_stored += 1;
        }

        // Check for skill level change
        if user.profile.cooking_skill_level != command.cooking_skill_level {
            let skill_update_event = user.update_skill_level(command.cooking_skill_level);
            self.event_store
                .store_event(&skill_update_event)
                .await
                .map_err(|e| {
                    UserError::DatabaseError(format!("Failed to store skill level event: {}", e))
                })?;
            events_stored += 1;
        }

        // Check for cooking time changes
        if user.profile.weekday_cooking_minutes != command.weekday_cooking_minutes
            || user.profile.weekend_cooking_minutes != command.weekend_cooking_minutes
        {
            let time_update_event = user.update_cooking_time(
                command.weekday_cooking_minutes,
                command.weekend_cooking_minutes,
            );
            self.event_store
                .store_event(&time_update_event)
                .await
                .map_err(|e| {
                    UserError::DatabaseError(format!("Failed to store cooking time event: {}", e))
                })?;
            events_stored += 1;
        }

        // Update user profile in database
        self.update_user_profile_in_db(&user).await?;

        // Commit transaction
        _tx.commit().await.map_err(|e| {
            UserError::DatabaseError(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(ProfileUpdateResponse {
            success: true,
            user_id: command.user_id,
            updated_at: user.updated_at,
            events_stored,
            message: "Profile updated successfully".to_string(),
        })
    }

    /// Handle dietary restrictions change command
    pub async fn handle_dietary_restrictions_change(
        &self,
        command: ChangeDietaryRestrictionsCommand,
    ) -> Result<ProfileUpdateResponse, UserError> {
        // Validation middleware
        command.validate_command()?;

        // Load current user
        let mut user = self.load_user_profile(command.user_id).await?;

        // Update dietary restrictions and store event
        let dietary_event = user.update_dietary_restrictions(command.new_restrictions);
        self.event_store
            .store_event(&dietary_event)
            .await
            .map_err(|e| {
                UserError::DatabaseError(format!(
                    "Failed to store dietary restrictions event: {}",
                    e
                ))
            })?;

        // Update user profile in database
        self.update_user_profile_in_db(&user).await?;

        Ok(ProfileUpdateResponse {
            success: true,
            user_id: command.user_id,
            updated_at: user.updated_at,
            events_stored: 1,
            message: "Dietary restrictions updated successfully".to_string(),
        })
    }

    /// Load user profile from database
    async fn load_user_profile(&self, user_id: Uuid) -> Result<User, UserError> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, family_size, skill_level, 
                   dietary_restrictions, weekday_cooking_minutes, weekend_cooking_minutes,
                   created_at, updated_at, email_verified
            FROM user_profiles 
            WHERE id = ?
            "#,
        )
        .bind(user_id.to_string())
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| UserError::NotFound)?;

        // Parse the data
        let email = imkitchen_shared::Email::new(row.get("email"))
            .map_err(|_| UserError::DatabaseError("Invalid email in database".to_string()))?;

        let family_size = FamilySize::new(row.get::<i64, _>("family_size") as u8)
            .map_err(|_| UserError::DatabaseError("Invalid family size in database".to_string()))?;

        let skill_level = match row.get::<String, _>("skill_level").as_str() {
            "Beginner" => SkillLevel::Beginner,
            "Intermediate" => SkillLevel::Intermediate,
            "Advanced" => SkillLevel::Advanced,
            _ => {
                return Err(UserError::DatabaseError(
                    "Invalid skill level in database".to_string(),
                ))
            }
        };

        let dietary_restrictions: Vec<DietaryRestriction> =
            serde_json::from_str(row.get("dietary_restrictions")).unwrap_or_default();

        let profile = UserProfile {
            family_size,
            cooking_skill_level: skill_level,
            dietary_restrictions,
            weekday_cooking_minutes: row
                .get::<Option<i64>, _>("weekday_cooking_minutes")
                .unwrap_or(30) as u32,
            weekend_cooking_minutes: row
                .get::<Option<i64>, _>("weekend_cooking_minutes")
                .unwrap_or(60) as u32,
        };

        let created_at = DateTime::parse_from_rfc3339(row.get("created_at"))
            .unwrap_or_else(|_| Utc::now().into())
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(row.get("updated_at"))
            .unwrap_or_else(|_| Utc::now().into())
            .with_timezone(&Utc);

        Ok(User {
            user_id,
            email,
            password_hash: row.get("password_hash"),
            profile,
            is_email_verified: row.get("email_verified"),
            created_at,
            updated_at,
        })
    }

    /// Update user profile in database
    async fn update_user_profile_in_db(&self, user: &User) -> Result<(), UserError> {
        let dietary_restrictions_json = serde_json::to_string(&user.profile.dietary_restrictions)
            .map_err(|e| {
            UserError::DatabaseError(format!("Failed to serialize dietary restrictions: {}", e))
        })?;

        sqlx::query(
            r#"
            UPDATE user_profiles 
            SET family_size = ?, skill_level = ?, dietary_restrictions = ?, 
                weekday_cooking_minutes = ?, weekend_cooking_minutes = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(user.profile.family_size.value as i64)
        .bind(user.profile.cooking_skill_level.to_string())
        .bind(dietary_restrictions_json)
        .bind(user.profile.weekday_cooking_minutes as i64)
        .bind(user.profile.weekend_cooking_minutes as i64)
        .bind(user.updated_at.format("%Y-%m-%d %H:%M:%S%.3fZ").to_string())
        .bind(user.user_id.to_string())
        .execute(&self.db_pool)
        .await
        .map_err(|e| UserError::DatabaseError(format!("Failed to update user profile: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use imkitchen_shared::{DietaryRestriction, FamilySize, SkillLevel};

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:")
            .await
            .expect("Failed to create in-memory database");

        // Create user_profiles table
        sqlx::query(
            r#"
            CREATE TABLE user_profiles (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                email_verified BOOLEAN NOT NULL DEFAULT FALSE,
                password_hash TEXT NOT NULL,
                family_size INTEGER NOT NULL,
                skill_level TEXT NOT NULL,
                dietary_restrictions TEXT DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("Failed to create user_profiles table");

        // Create user_events table
        sqlx::query(
            r#"
            CREATE TABLE user_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                aggregate_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                event_data TEXT NOT NULL,
                version INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(aggregate_id, version)
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("Failed to create user_events table");

        pool
    }

    async fn create_test_user(pool: &SqlitePool) -> Uuid {
        let user_id = Uuid::new_v4();
        let email = "test@example.com";
        let password_hash = "hashed_password";

        sqlx::query(
            r#"
            INSERT INTO user_profiles 
            (id, email, password_hash, family_size, skill_level, dietary_restrictions, email_verified)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id.to_string())
        .bind(email)
        .bind(password_hash)
        .bind(2i64)
        .bind("Beginner")
        .bind("[]")
        .bind(true)
        .execute(pool)
        .await
        .expect("Failed to create test user");

        user_id
    }

    #[tokio::test]
    async fn test_update_user_profile_command_validation() {
        let user_id = Uuid::new_v4();
        let family_size = FamilySize::new(4).unwrap();

        // Valid command
        let valid_command =
            UpdateUserProfileCommand::new(user_id, family_size, SkillLevel::Intermediate, 30, 60);
        assert!(valid_command.validate_command().is_ok());

        // Invalid command - weekday > weekend
        let invalid_command =
            UpdateUserProfileCommand::new(user_id, family_size, SkillLevel::Intermediate, 90, 60);
        assert!(invalid_command.validate_command().is_err());
    }

    #[tokio::test]
    async fn test_dietary_restrictions_command_validation() {
        let user_id = Uuid::new_v4();

        // Valid command
        let valid_command = ChangeDietaryRestrictionsCommand::new(
            user_id,
            vec![
                DietaryRestriction::Vegetarian,
                DietaryRestriction::GlutenFree,
            ],
        );
        assert!(valid_command.validate_command().is_ok());

        // Invalid command - conflicting restrictions
        let invalid_command = ChangeDietaryRestrictionsCommand::new(
            user_id,
            vec![DietaryRestriction::Vegetarian, DietaryRestriction::Vegan],
        );
        assert!(invalid_command.validate_command().is_err());

        // Invalid command - too many restrictions
        let too_many_command = ChangeDietaryRestrictionsCommand::new(
            user_id,
            vec![
                DietaryRestriction::Vegetarian,
                DietaryRestriction::GlutenFree,
                DietaryRestriction::DairyFree,
                DietaryRestriction::NutFree,
                DietaryRestriction::SoyFree,
                DietaryRestriction::LowSodium,
            ],
        );
        assert!(too_many_command.validate_command().is_err());
    }

    #[tokio::test]
    async fn test_profile_command_handler_update() {
        let pool = setup_test_db().await;
        let handler = ProfileCommandHandler::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        let command = UpdateUserProfileCommand::new(
            user_id,
            FamilySize::new(4).unwrap(),
            SkillLevel::Advanced,
            45,
            90,
        );

        let response = handler.handle_update_profile(command).await.unwrap();

        assert!(response.success);
        assert_eq!(response.user_id, user_id);
        assert!(response.events_stored > 0);
    }

    #[tokio::test]
    async fn test_dietary_restrictions_command_handler() {
        let pool = setup_test_db().await;
        let handler = ProfileCommandHandler::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        let command = ChangeDietaryRestrictionsCommand::new(
            user_id,
            vec![
                DietaryRestriction::Vegetarian,
                DietaryRestriction::GlutenFree,
            ],
        );

        let response = handler
            .handle_dietary_restrictions_change(command)
            .await
            .unwrap();

        assert!(response.success);
        assert_eq!(response.user_id, user_id);
        assert_eq!(response.events_stored, 1);
    }
}
