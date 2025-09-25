use crate::models::user::{CreateUserRequest, User, UserSession};
use chrono::{DateTime, Duration, Utc};
use sqlx::{Row, SqlitePool};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum UserRepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("User not found")]
    NotFound,
    #[error("Email already exists")]
    EmailExists,
    #[error("Invalid session token")]
    InvalidSession,
}

#[derive(Clone)]
pub struct UserRepository {
    pool: SqlitePool,
}

impl UserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new user
    pub async fn create_user(
        &self,
        request: CreateUserRequest,
        password_hash: String,
    ) -> Result<User, UserRepositoryError> {
        let user_id = Uuid::new_v4().to_string();
        let family_size = request.family_size.unwrap_or(1);
        let dietary_restrictions = "[]".to_string();

        // Check if email already exists
        let existing = sqlx::query("SELECT COUNT(*) as count FROM users WHERE email = ?")
            .bind(&request.email)
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = existing.get("count");
        if count > 0 {
            return Err(UserRepositoryError::EmailExists);
        }

        // Generate email verification token
        let verification_token = Uuid::new_v4().to_string();
        let verification_expires = Utc::now() + Duration::days(7); // 7 days to verify

        // Insert user
        sqlx::query(
            r#"
            INSERT INTO users (
                id, email, password_hash, name, family_size, dietary_restrictions,
                cooking_skill_level, cooking_time_preferences, email_verified, email_verification_token, 
                email_verification_expires_at, created_at, updated_at, last_active
            )
            VALUES (?, ?, ?, ?, ?, ?, 'beginner', '{"weekdayMaxMinutes": 30, "weekendMaxMinutes": 60}', 0, ?, ?, datetime('now'), datetime('now'), datetime('now'))
            "#,
        )
        .bind(&user_id)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&request.name)
        .bind(family_size)
        .bind(&dietary_restrictions)
        .bind(&verification_token)
        .bind(verification_expires.format("%Y-%m-%d %H:%M:%S").to_string())
        .execute(&self.pool)
        .await?;

        // Return the created user
        self.find_by_id(&user_id)
            .await?
            .ok_or(UserRepositoryError::NotFound)
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, UserRepositoryError> {
        let row = sqlx::query("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(Some(self.parse_user_row(&row)?)),
            None => Ok(None),
        }
    }

    /// Find user by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<User>, UserRepositoryError> {
        let row = sqlx::query("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(Some(self.parse_user_row(&row)?)),
            None => Ok(None),
        }
    }

    /// Parse a database row into a User struct
    fn parse_user_row(&self, row: &sqlx::sqlite::SqliteRow) -> Result<User, UserRepositoryError> {
        let id: String = row.try_get("id")?;
        let email: String = row.try_get("email")?;
        let password_hash: String = row.try_get("password_hash")?;
        let name: String = row.try_get("name")?;
        let family_size: Option<i64> = row.try_get("family_size")?;
        let dietary_restrictions: Option<String> = row.try_get("dietary_restrictions")?;
        let cooking_skill_level: Option<String> = row.try_get("cooking_skill_level")?;
        let email_verified: Option<i64> = row.try_get("email_verified")?;
        let email_verification_token: Option<String> = row.try_get("email_verification_token")?;
        let email_verification_expires_at: Option<String> =
            row.try_get("email_verification_expires_at")?;
        let password_reset_token: Option<String> = row.try_get("password_reset_token")?;
        let password_reset_expires_at: Option<String> = row.try_get("password_reset_expires_at")?;
        let cooking_time_preferences: Option<String> = row.try_get("cooking_time_preferences")?;
        let created_at: Option<String> = row.try_get("created_at")?;
        let last_active: Option<String> = row.try_get("last_active")?;

        // Parse datetime strings to DateTime<Utc>
        let parse_datetime = |dt_str: Option<String>| -> DateTime<Utc> {
            dt_str
                .and_then(|s| {
                    DateTime::parse_from_str(&format!("{} +00:00", s), "%Y-%m-%d %H:%M:%S %z").ok()
                })
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now)
        };

        let parse_optional_datetime = |dt_str: Option<String>| -> Option<DateTime<Utc>> {
            dt_str
                .and_then(|s| {
                    DateTime::parse_from_str(&format!("{} +00:00", s), "%Y-%m-%d %H:%M:%S %z").ok()
                })
                .map(|dt| dt.with_timezone(&Utc))
        };

        Ok(User {
            id,
            email,
            password_hash,
            name,
            family_size: family_size.unwrap_or(1) as i32,
            dietary_restrictions: dietary_restrictions.unwrap_or_else(|| "[]".to_string()),
            cooking_skill_level: cooking_skill_level.unwrap_or_else(|| "beginner".to_string()),
            email_verified: email_verified.unwrap_or(0) != 0,
            email_verification_token,
            email_verification_expires_at: parse_optional_datetime(email_verification_expires_at),
            password_reset_token,
            password_reset_expires_at: parse_optional_datetime(password_reset_expires_at),
            cooking_time_preferences: cooking_time_preferences.unwrap_or_else(|| {
                r#"{"weekdayMaxMinutes": 30, "weekendMaxMinutes": 60}"#.to_string()
            }),
            created_at: parse_datetime(created_at),
            last_active: parse_datetime(last_active),
        })
    }

    /// Update user's last active timestamp
    pub async fn update_last_active(&self, user_id: &str) -> Result<(), UserRepositoryError> {
        sqlx::query("UPDATE users SET last_active = datetime('now') WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update user profile
    pub async fn update_profile(&self, user: &User) -> Result<(), UserRepositoryError> {
        sqlx::query(
            r#"
            UPDATE users SET
                name = ?,
                family_size = ?,
                dietary_restrictions = ?,
                cooking_skill_level = ?,
                cooking_time_preferences = ?,
                last_active = datetime('now')
            WHERE id = ?
            "#,
        )
        .bind(&user.name)
        .bind(user.family_size)
        .bind(&user.dietary_restrictions)
        .bind(&user.cooking_skill_level)
        .bind(&user.cooking_time_preferences)
        .bind(&user.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete user account and cascade delete related data
    pub async fn delete_user(&self, user_id: &str) -> Result<(), UserRepositoryError> {
        // Start a transaction for cascade delete
        let mut tx = self.pool.begin().await?;

        // Delete user sessions (will cascade automatically due to FK constraints)
        sqlx::query("DELETE FROM user_sessions WHERE user_id = ?")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        // Delete the user (this will cascade delete other related data)
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(UserRepositoryError::NotFound);
        }

        tx.commit().await?;
        Ok(())
    }

    /// Verify email with token
    pub async fn verify_email(&self, token: &str) -> Result<bool, UserRepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE users 
            SET email_verified = 1, 
                email_verification_token = NULL, 
                email_verification_expires_at = NULL
            WHERE email_verification_token = ? 
            AND email_verification_expires_at > datetime('now')
            "#,
        )
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Create user session
    pub async fn create_session(&self, user_id: &str) -> Result<UserSession, UserRepositoryError> {
        let session_id = Uuid::new_v4().to_string();
        let session_token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::days(30);

        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, session_token, expires_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&session_id)
        .bind(user_id)
        .bind(&session_token)
        .bind(expires_at.format("%Y-%m-%d %H:%M:%S").to_string())
        .execute(&self.pool)
        .await?;

        Ok(UserSession {
            id: session_id,
            user_id: user_id.to_string(),
            session_token,
            expires_at,
            created_at: Utc::now(),
        })
    }

    /// Find session by token
    pub async fn find_session(
        &self,
        token: &str,
    ) -> Result<Option<UserSession>, UserRepositoryError> {
        let row = sqlx::query("SELECT * FROM user_sessions WHERE session_token = ?")
            .bind(token)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let id: String = row.try_get("id")?;
                let user_id: String = row.try_get("user_id")?;
                let session_token: String = row.try_get("session_token")?;
                let expires_at_str: String = row.try_get("expires_at")?;
                let created_at_str: Option<String> = row.try_get("created_at")?;

                // Parse datetime strings
                let expires_at = DateTime::parse_from_str(
                    &format!("{} +00:00", expires_at_str),
                    "%Y-%m-%d %H:%M:%S %z",
                )
                .map_err(|_| {
                    UserRepositoryError::Database(sqlx::Error::Decode(
                        "Invalid datetime format".into(),
                    ))
                })?
                .with_timezone(&Utc);

                let created_at = created_at_str
                    .and_then(|s| {
                        DateTime::parse_from_str(&format!("{} +00:00", s), "%Y-%m-%d %H:%M:%S %z")
                            .ok()
                    })
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);

                Ok(Some(UserSession {
                    id,
                    user_id,
                    session_token,
                    expires_at,
                    created_at,
                }))
            }
            None => Ok(None),
        }
    }

    /// Delete session
    pub async fn delete_session(&self, token: &str) -> Result<(), UserRepositoryError> {
        sqlx::query("DELETE FROM user_sessions WHERE session_token = ?")
            .bind(token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<u64, UserRepositoryError> {
        let result = sqlx::query("DELETE FROM user_sessions WHERE expires_at <= datetime('now')")
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Set password reset token
    pub async fn set_password_reset_token(
        &self,
        email: &str,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<bool, UserRepositoryError> {
        let expires_at_str = expires_at.format("%Y-%m-%d %H:%M:%S").to_string();
        let result = sqlx::query(
            "UPDATE users SET password_reset_token = ?, password_reset_expires_at = ? WHERE email = ?",
        )
        .bind(token)
        .bind(expires_at_str)
        .bind(email)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Reset password with token
    pub async fn reset_password(
        &self,
        token: &str,
        new_password_hash: &str,
    ) -> Result<bool, UserRepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE users 
            SET password_hash = ?, 
                password_reset_token = NULL, 
                password_reset_expires_at = NULL
            WHERE password_reset_token = ? 
            AND password_reset_expires_at > datetime('now')
            "#,
        )
        .bind(new_password_hash)
        .bind(token)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        // Run migrations
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_create_and_find_user() {
        let pool = setup_test_db().await;
        let repo = UserRepository::new(pool);

        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            family_size: Some(4),
        };

        // Create user
        let user = repo
            .create_user(request, "hashed_password".to_string())
            .await
            .unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "Test User");
        assert_eq!(user.family_size, 4);
        assert!(!user.email_verified);

        // Find user by email
        let found_user = repo
            .find_by_email("test@example.com")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found_user.id, user.id);

        // Find user by ID
        let found_user_by_id = repo.find_by_id(&user.id).await.unwrap().unwrap();
        assert_eq!(found_user_by_id.email, user.email);
    }

    #[tokio::test]
    async fn test_duplicate_email() {
        let pool = setup_test_db().await;
        let repo = UserRepository::new(pool);

        let request = CreateUserRequest {
            email: "duplicate@example.com".to_string(),
            password: "password123".to_string(),
            name: "First User".to_string(),
            family_size: None,
        };

        // Create first user
        repo.create_user(request.clone(), "hashed_password".to_string())
            .await
            .unwrap();

        // Try to create second user with same email
        let result = repo
            .create_user(request, "hashed_password".to_string())
            .await;
        assert!(matches!(
            result.unwrap_err(),
            UserRepositoryError::EmailExists
        ));
    }

    #[tokio::test]
    async fn test_session_management() {
        let pool = setup_test_db().await;
        let repo = UserRepository::new(pool);

        // Create a user first
        let request = CreateUserRequest {
            email: "session_test@example.com".to_string(),
            password: "password123".to_string(),
            name: "Session User".to_string(),
            family_size: None,
        };
        let user = repo
            .create_user(request, "hashed_password".to_string())
            .await
            .unwrap();

        // Create session
        let session = repo.create_session(&user.id).await.unwrap();
        assert_eq!(session.user_id, user.id);
        assert!(!session.is_expired());

        // Find session
        let found_session = repo
            .find_session(&session.session_token)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found_session.id, session.id);

        // Delete session
        repo.delete_session(&session.session_token).await.unwrap();
        let deleted_session = repo.find_session(&session.session_token).await.unwrap();
        assert!(deleted_session.is_none());
    }
}
