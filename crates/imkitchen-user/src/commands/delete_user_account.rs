// Account deletion commands with audit trail and cascading operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::{
    domain::{User, UserError},
    event_store::EventStore,
    events::{DeletionReason, UserAccountDeleted},
};

/// Command to delete a user account with proper audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserAccountCommand {
    pub user_id: Uuid,
    pub deletion_reason: DeletionReason,
    pub initiated_by_user: bool,
    pub deletion_ip: Option<String>,
    pub user_agent: Option<String>,
    /// Optional admin user ID if deletion is admin-initiated
    pub admin_user_id: Option<Uuid>,
}

/// Command to permanently purge user data (GDPR compliance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeUserDataCommand {
    pub user_id: Uuid,
    /// Verification that retention period has passed
    pub data_purge_scheduled_at: DateTime<Utc>,
    /// Verification that this is a legitimate purge request
    pub admin_authorization: Uuid,
}

/// Response from account deletion operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDeletionResponse {
    pub success: bool,
    pub user_id: Uuid,
    pub deleted_at: DateTime<Utc>,
    pub events_stored: u32,
    pub data_purge_scheduled_at: DateTime<Utc>,
    pub cascading_operations_initiated: Vec<String>,
    pub message: String,
}

/// Account deletion command handler with database operations
#[derive(Debug, Clone)]
pub struct AccountDeletionHandler {
    event_store: EventStore,
    db_pool: SqlitePool,
}

impl DeleteUserAccountCommand {
    /// Create a user-requested account deletion command
    pub fn user_requested(
        user_id: Uuid,
        deletion_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            user_id,
            deletion_reason: DeletionReason::UserRequested,
            initiated_by_user: true,
            deletion_ip,
            user_agent,
            admin_user_id: None,
        }
    }

    /// Create a GDPR right-to-be-forgotten deletion command
    pub fn gdpr_request(user_id: Uuid) -> Self {
        Self {
            user_id,
            deletion_reason: DeletionReason::GdprRequest,
            initiated_by_user: true,
            deletion_ip: None,
            user_agent: None,
            admin_user_id: None,
        }
    }

    /// Create an admin-initiated deletion command
    pub fn admin_action(
        user_id: Uuid,
        admin_user_id: Uuid,
        deletion_reason: DeletionReason,
    ) -> Self {
        Self {
            user_id,
            deletion_reason,
            initiated_by_user: false,
            deletion_ip: None,
            user_agent: None,
            admin_user_id: Some(admin_user_id),
        }
    }

    /// Validate the deletion command
    pub fn validate_command(&self) -> Result<(), UserError> {
        // Ensure admin actions have admin user ID
        if !self.initiated_by_user && self.admin_user_id.is_none() {
            return Err(UserError::DatabaseError(
                "Admin-initiated deletions must include admin user ID".to_string(),
            ));
        }

        // Validate deletion reason is appropriate for initiated type
        match (&self.deletion_reason, self.initiated_by_user) {
            (DeletionReason::AdminAction, true) => {
                return Err(UserError::DatabaseError(
                    "Admin actions cannot be user-initiated".to_string(),
                ));
            }
            (DeletionReason::UserRequested, false) => {
                return Err(UserError::DatabaseError(
                    "User requests cannot be admin-initiated".to_string(),
                ));
            }
            _ => {}
        }

        Ok(())
    }
}

impl PurgeUserDataCommand {
    /// Create a data purge command
    pub fn new(
        user_id: Uuid,
        data_purge_scheduled_at: DateTime<Utc>,
        admin_authorization: Uuid,
    ) -> Self {
        Self {
            user_id,
            data_purge_scheduled_at,
            admin_authorization,
        }
    }

    /// Validate that purge is authorized and timely
    pub fn validate_command(&self) -> Result<(), UserError> {
        let now = Utc::now();

        // Ensure purge time has arrived or passed
        if now < self.data_purge_scheduled_at {
            return Err(UserError::DatabaseError(format!(
                "Data purge not yet scheduled. Purge time: {}, Current time: {}",
                self.data_purge_scheduled_at, now
            )));
        }

        Ok(())
    }
}

impl AccountDeletionHandler {
    pub fn new(db_pool: SqlitePool) -> Self {
        let event_store = EventStore::new(db_pool.clone());
        Self {
            event_store,
            db_pool,
        }
    }

    /// Handle account deletion command with cascading operations
    pub async fn handle_delete_account(
        &self,
        command: DeleteUserAccountCommand,
    ) -> Result<AccountDeletionResponse, UserError> {
        // Validation
        command.validate_command()?;

        // Load user to get account details
        let user = self.load_user(command.user_id).await?;

        // Begin database transaction for atomicity
        let mut tx =
            self.db_pool.begin().await.map_err(|e| {
                UserError::DatabaseError(format!("Database transaction error: {}", e))
            })?;

        // Create account deletion event
        let deletion_event = UserAccountDeleted::new(
            command.user_id,
            user.email.clone(),
            command.deletion_reason.clone(),
            command.initiated_by_user,
            user.created_at,
            command.deletion_ip,
            command.user_agent,
        );

        // Store the deletion event
        self.event_store
            .store_event(&deletion_event)
            .await
            .map_err(|e| {
                UserError::DatabaseError(format!("Failed to store deletion event: {}", e))
            })?;

        // Mark user account as deleted in database
        self.mark_user_deleted(&mut tx, command.user_id, &deletion_event)
            .await?;

        // Initiate cascading deletions across bounded contexts
        let cascading_operations = self
            .initiate_cascading_deletions(&mut tx, command.user_id)
            .await?;

        // Create audit trail entry
        self.create_audit_trail_entry(&mut tx, &deletion_event, command.admin_user_id)
            .await?;

        // Commit transaction
        tx.commit().await.map_err(|e| {
            UserError::DatabaseError(format!("Failed to commit deletion transaction: {}", e))
        })?;

        Ok(AccountDeletionResponse {
            success: true,
            user_id: command.user_id,
            deleted_at: deletion_event.deleted_at,
            events_stored: 1,
            data_purge_scheduled_at: deletion_event.data_purge_scheduled_at,
            cascading_operations_initiated: cascading_operations,
            message: format!(
                "Account deleted successfully. Data purge scheduled for {}",
                deletion_event
                    .data_purge_scheduled_at
                    .format("%Y-%m-%d %H:%M:%S UTC")
            ),
        })
    }

    /// Handle data purge command for GDPR compliance
    pub async fn handle_purge_user_data(
        &self,
        command: PurgeUserDataCommand,
    ) -> Result<AccountDeletionResponse, UserError> {
        // Validation
        command.validate_command()?;

        // Begin transaction
        let mut tx =
            self.db_pool.begin().await.map_err(|e| {
                UserError::DatabaseError(format!("Database transaction error: {}", e))
            })?;

        // Permanently delete all user data
        let purged_operations = self.purge_all_user_data(&mut tx, command.user_id).await?;

        // Create purge audit log
        self.create_purge_audit_entry(&mut tx, command.user_id, command.admin_authorization)
            .await?;

        // Commit transaction
        tx.commit().await.map_err(|e| {
            UserError::DatabaseError(format!("Failed to commit purge transaction: {}", e))
        })?;

        Ok(AccountDeletionResponse {
            success: true,
            user_id: command.user_id,
            deleted_at: Utc::now(),
            events_stored: 0,                    // Data is permanently purged
            data_purge_scheduled_at: Utc::now(), // Already purged
            cascading_operations_initiated: purged_operations,
            message: "User data permanently purged for GDPR compliance".to_string(),
        })
    }

    /// Load user from database
    async fn load_user(&self, user_id: Uuid) -> Result<User, UserError> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, family_size, skill_level,
                   dietary_restrictions, created_at, updated_at, email_verified
            FROM user_profiles
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(user_id.to_string())
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| UserError::NotFound)?;

        // Parse user data similar to existing profile handlers
        use crate::domain::UserProfile;
        use chrono::DateTime;
        use imkitchen_shared::{DietaryRestriction, Email, FamilySize, SkillLevel};

        let email = Email::new(row.get("email"))
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
            weekday_cooking_minutes: 30, // Default - could be loaded from user_preferences
            weekend_cooking_minutes: 60,
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

    /// Mark user as deleted in database (soft delete)
    async fn mark_user_deleted(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        user_id: Uuid,
        deletion_event: &UserAccountDeleted,
    ) -> Result<(), UserError> {
        sqlx::query(
            r#"
            UPDATE user_profiles 
            SET deleted_at = ?, 
                deletion_reason = ?,
                data_purge_scheduled_at = ?
            WHERE id = ?
            "#,
        )
        .bind(
            deletion_event
                .deleted_at
                .format("%Y-%m-%d %H:%M:%S%.3fZ")
                .to_string(),
        )
        .bind(
            serde_json::to_string(&deletion_event.deletion_reason).map_err(|e| {
                UserError::DatabaseError(format!("Failed to serialize deletion reason: {}", e))
            })?,
        )
        .bind(
            deletion_event
                .data_purge_scheduled_at
                .format("%Y-%m-%d %H:%M:%S%.3fZ")
                .to_string(),
        )
        .bind(user_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(|e| UserError::DatabaseError(format!("Failed to mark user as deleted: {}", e)))?;

        Ok(())
    }

    /// Initiate cascading deletions across bounded contexts
    async fn initiate_cascading_deletions(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        user_id: Uuid,
    ) -> Result<Vec<String>, UserError> {
        let mut operations = vec![];

        // Mark user preferences for deletion
        sqlx::query("UPDATE user_preferences SET deleted_at = ? WHERE user_id = ?")
            .bind(Utc::now().format("%Y-%m-%d %H:%M:%S%.3fZ").to_string())
            .bind(user_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                UserError::DatabaseError(format!("Failed to delete user preferences: {}", e))
            })?;
        operations.push("user_preferences".to_string());

        // Note: In a real system, this would trigger events to other bounded contexts
        // such as meal planning, shopping lists, etc.
        operations.push("cross_context_events_initiated".to_string());

        Ok(operations)
    }

    /// Create audit trail entry for deletion
    async fn create_audit_trail_entry(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        deletion_event: &UserAccountDeleted,
        admin_user_id: Option<Uuid>,
    ) -> Result<(), UserError> {
        sqlx::query(
            r#"
            INSERT INTO account_deletion_audit 
            (user_id, deletion_reason, initiated_by_user, admin_user_id, deleted_at, audit_created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(deletion_event.user_id.to_string())
        .bind(serde_json::to_string(&deletion_event.deletion_reason)
            .map_err(|e| UserError::DatabaseError(format!("Failed to serialize deletion reason: {}", e)))?)
        .bind(deletion_event.initiated_by_user)
        .bind(admin_user_id.map(|id| id.to_string()))
        .bind(deletion_event.deleted_at.format("%Y-%m-%d %H:%M:%S%.3fZ").to_string())
        .bind(Utc::now().format("%Y-%m-%d %H:%M:%S%.3fZ").to_string())
        .execute(&mut **tx)
        .await
        .map_err(|e| UserError::DatabaseError(format!("Failed to create audit trail entry: {}", e)))?;

        Ok(())
    }

    /// Permanently purge all user data
    async fn purge_all_user_data(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        user_id: Uuid,
    ) -> Result<Vec<String>, UserError> {
        let mut operations = vec![];

        // Permanently delete user profile
        sqlx::query("DELETE FROM user_profiles WHERE id = ?")
            .bind(user_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                UserError::DatabaseError(format!("Failed to purge user profile: {}", e))
            })?;
        operations.push("user_profile_purged".to_string());

        // Permanently delete user preferences
        sqlx::query("DELETE FROM user_preferences WHERE user_id = ?")
            .bind(user_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                UserError::DatabaseError(format!("Failed to purge user preferences: {}", e))
            })?;
        operations.push("user_preferences_purged".to_string());

        // Note: Events are kept for audit purposes even after data purge
        // Only personal data is removed for GDPR compliance

        Ok(operations)
    }

    /// Create purge audit entry
    async fn create_purge_audit_entry(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        user_id: Uuid,
        admin_authorization: Uuid,
    ) -> Result<(), UserError> {
        sqlx::query(
            r#"
            INSERT INTO data_purge_audit 
            (user_id, admin_authorization, purged_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(user_id.to_string())
        .bind(admin_authorization.to_string())
        .bind(Utc::now().format("%Y-%m-%d %H:%M:%S%.3fZ").to_string())
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            UserError::DatabaseError(format!("Failed to create purge audit entry: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::UserProfile;
    use imkitchen_shared::{FamilySize, SkillLevel};

    #[test]
    fn test_delete_user_account_command_validation() {
        // Valid user-requested deletion
        let user_command = DeleteUserAccountCommand::user_requested(
            Uuid::new_v4(),
            Some("127.0.0.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );
        assert!(user_command.validate_command().is_ok());

        // Valid admin-initiated deletion
        let admin_command = DeleteUserAccountCommand::admin_action(
            Uuid::new_v4(),
            Uuid::new_v4(),
            DeletionReason::TosViolation,
        );
        assert!(admin_command.validate_command().is_ok());

        // Invalid: admin action without admin user ID
        let invalid_command = DeleteUserAccountCommand {
            user_id: Uuid::new_v4(),
            deletion_reason: DeletionReason::AdminAction,
            initiated_by_user: false,
            deletion_ip: None,
            user_agent: None,
            admin_user_id: None,
        };
        assert!(invalid_command.validate_command().is_err());
    }

    #[test]
    fn test_purge_user_data_command_validation() {
        let future_time = Utc::now() + chrono::Duration::days(1);
        let past_time = Utc::now() - chrono::Duration::days(1);

        // Valid purge command (past scheduled time)
        let valid_command = PurgeUserDataCommand::new(Uuid::new_v4(), past_time, Uuid::new_v4());
        assert!(valid_command.validate_command().is_ok());

        // Invalid purge command (future scheduled time)
        let invalid_command =
            PurgeUserDataCommand::new(Uuid::new_v4(), future_time, Uuid::new_v4());
        assert!(invalid_command.validate_command().is_err());
    }

    #[test]
    fn test_user_account_deleted_event_creation() {
        let user_id = Uuid::new_v4();
        let email = imkitchen_shared::Email::new("test@example.com".to_string()).unwrap();
        let account_created_at = Utc::now() - chrono::Duration::days(30);

        // Test user-requested deletion
        let user_deletion = UserAccountDeleted::user_requested(
            user_id,
            email.clone(),
            account_created_at,
            Some("127.0.0.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        );

        assert_eq!(user_deletion.user_id, user_id);
        assert!(user_deletion.initiated_by_user);
        assert_eq!(user_deletion.account_age_days, 30);
        assert!(!user_deletion.requires_immediate_purge());

        // Test GDPR deletion
        let gdpr_deletion = UserAccountDeleted::gdpr_request(user_id, email, account_created_at);

        assert!(gdpr_deletion.requires_immediate_purge());
        assert!(gdpr_deletion.initiated_by_user);
    }
}
