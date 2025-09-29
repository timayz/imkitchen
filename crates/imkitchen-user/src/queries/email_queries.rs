// Email-related queries for async validation

use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::commands::email_validation::{
    CheckEmailExistsResponse, EmailValidationError, ValidateUsernameAvailabilityResponse,
};

/// Query handler for checking if an email exists in the database
pub struct EmailExistsQueryHandler {
    db_pool: SqlitePool,
}

impl EmailExistsQueryHandler {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    /// Check if the given email address already exists in the user_profiles table
    pub async fn handle_email_exists_check(
        &self,
        email: &str,
        request_id: Option<Uuid>,
    ) -> Result<CheckEmailExistsResponse, EmailValidationError> {
        // Query the database to check if email exists
        let row = sqlx::query("SELECT COUNT(*) as count FROM user_profiles WHERE email = ?")
            .bind(email)
            .fetch_one(&self.db_pool)
            .await?;

        let count: i64 = row.get("count");
        let exists = count > 0;

        Ok(CheckEmailExistsResponse {
            email: email.to_string(),
            exists,
            request_id,
        })
    }

    /// Check username availability (email-based) and provide suggestions
    pub async fn handle_username_availability_check(
        &self,
        username: &str,
        request_id: Option<Uuid>,
    ) -> Result<ValidateUsernameAvailabilityResponse, EmailValidationError> {
        // Check if the username (email) is available
        let row = sqlx::query("SELECT COUNT(*) as count FROM user_profiles WHERE email = ?")
            .bind(username)
            .fetch_one(&self.db_pool)
            .await?;

        let count: i64 = row.get("count");
        let exists = count > 0;
        let available = !exists;

        let suggestions = if !available {
            generate_email_suggestions(username)
        } else {
            vec![]
        };

        Ok(ValidateUsernameAvailabilityResponse {
            username: username.to_string(),
            available,
            request_id,
            suggestions,
        })
    }
}

/// Generate alternative email suggestions if the original is not available
fn generate_email_suggestions(email: &str) -> Vec<String> {
    if let Some((local, domain)) = email.split_once('@') {
        let base_suggestions = vec![
            format!("{}1@{}", local, domain),
            format!("{}2@{}", local, domain),
            format!("{}.user@{}", local, domain),
            format!("{}2024@{}", local, domain),
        ];

        // Filter out invalid suggestions using email regex
        let email_regex =
            regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        base_suggestions
            .into_iter()
            .filter(|s| email_regex.is_match(s))
            .take(3) // Limit to 3 suggestions
            .collect()
    } else {
        vec![]
    }
}

/// User repository for database operations
#[derive(Clone)]
pub struct UserRepository {
    db_pool: SqlitePool,
}

impl UserRepository {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    /// Find user by email address
    pub async fn find_by_email(
        &self,
        email: &str,
    ) -> Result<Option<UserRecord>, EmailValidationError> {
        let row = sqlx::query(
            "SELECT id, email, password_hash, family_size, skill_level, 
             created_at, updated_at 
             FROM user_profiles WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(row) = row {
            let user = UserRecord {
                user_id: row.get("id"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                family_size: row.get("family_size"),
                cooking_skill_level: row.get("skill_level"),
                weekday_cooking_minutes: None, // Not stored in current schema
                weekend_cooking_minutes: None, // Not stored in current schema
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Check if email exists (simpler version)
    pub async fn email_exists(&self, email: &str) -> Result<bool, EmailValidationError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM user_profiles WHERE email = ?")
            .bind(email)
            .fetch_one(&self.db_pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }
}

/// Database record structure for user_profiles table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub user_id: String,
    pub email: String,
    pub password_hash: String,
    pub family_size: i64,
    pub cooking_skill_level: String,
    pub weekday_cooking_minutes: Option<i64>,
    pub weekend_cooking_minutes: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_suggestions_generation() {
        let suggestions = generate_email_suggestions("john@example.com");
        assert!(suggestions.len() <= 3);
        assert!(suggestions.iter().all(|s| s.contains("@example.com")));
        let email_regex =
            regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        assert!(suggestions.iter().all(|s| email_regex.is_match(s)));
    }

    #[test]
    fn test_invalid_email_suggestions() {
        let suggestions = generate_email_suggestions("invalid-email");
        assert!(suggestions.is_empty());
    }
}
