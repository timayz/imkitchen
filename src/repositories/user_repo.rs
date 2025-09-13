use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::{CreateUserRequest, User};

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: CreateUserRequest, password_hash: String, verification_token: Option<String>) -> Result<User, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        
        let dietary_preferences = request.dietary_preferences.unwrap_or_default();
        let skill_level = request.skill_level.map(|s| s.to_string()).unwrap_or_else(|| "beginner".to_string());
        let household_size = request.household_size.unwrap_or(1);
        let kitchen_equipment = request.kitchen_equipment
            .map(|eq| serde_json::to_value(eq).unwrap_or(serde_json::json!([])))
            .unwrap_or(serde_json::json!([]));

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, dietary_preferences, skill_level, household_size, kitchen_equipment, verification_token, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, email, password_hash, dietary_preferences, skill_level, household_size, kitchen_equipment, email_verified, verification_token, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(&request.email)
        .bind(&password_hash)
        .bind(&dietary_preferences)
        .bind(&skill_level)
        .bind(household_size)
        .bind(&kitchen_equipment)
        .bind(&verification_token)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, dietary_preferences, skill_level, household_size, kitchen_equipment, email_verified, verification_token, created_at, updated_at
            FROM users 
            WHERE email = $1
            "#
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, dietary_preferences, skill_level, household_size, kitchen_equipment, email_verified, verification_token, created_at, updated_at
            FROM users 
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_verification_token(&self, token: &str) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, dietary_preferences, skill_level, household_size, kitchen_equipment, email_verified, verification_token, created_at, updated_at
            FROM users 
            WHERE verification_token = $1
            "#
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update(&self, id: Uuid, user: &User) -> Result<User, sqlx::Error> {
        let updated_user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET email = $2, dietary_preferences = $3, skill_level = $4, household_size = $5, kitchen_equipment = $6, email_verified = $7, verification_token = $8, updated_at = NOW()
            WHERE id = $1
            RETURNING id, email, password_hash, dietary_preferences, skill_level, household_size, kitchen_equipment, email_verified, verification_token, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(&user.email)
        .bind(&user.dietary_preferences)
        .bind(&user.skill_level)
        .bind(user.household_size)
        .bind(&user.kitchen_equipment)
        .bind(user.email_verified)
        .bind(&user.verification_token)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_user)
    }

    pub async fn verify_email(&self, id: Uuid) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET email_verified = TRUE, verification_token = NULL, updated_at = NOW()
            WHERE id = $1
            RETURNING id, email, password_hash, dietary_preferences, skill_level, household_size, kitchen_equipment, email_verified, verification_token, created_at, updated_at
            "#
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_password(&self, id: Uuid, password_hash: String) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET password_hash = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, email, password_hash, dietary_preferences, skill_level, household_size, kitchen_equipment, email_verified, verification_token, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(&password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM users WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}