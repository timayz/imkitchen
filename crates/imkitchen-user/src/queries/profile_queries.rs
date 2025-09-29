// Profile-specific queries with optimized projections

use chrono::{DateTime, Utc};
use imkitchen_shared::{DietaryRestriction, FamilySize, SkillLevel};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::UserProfile;

/// Query to get user profile by ID with optimized projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileByIdQuery {
    pub user_id: Uuid,
}

/// Query to get user preferences for meal planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferencesQuery {
    pub user_id: Uuid,
}

/// Optimized user profile view for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileView {
    pub user_id: Uuid,
    pub family_size: FamilySize,
    pub cooking_skill_level: SkillLevel,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub weekday_cooking_minutes: u32,
    pub weekend_cooking_minutes: u32,
    pub profile_completeness: f32, // Percentage of profile completion
    pub last_updated: DateTime<Utc>,
}

/// User preferences view optimized for meal planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferencesView {
    pub user_id: Uuid,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub family_size: u8,
    pub skill_level: SkillLevel,
    pub available_cooking_time_weekday: u32,
    pub available_cooking_time_weekend: u32,
    pub recommended_complexity: Vec<String>,
    pub portion_multiplier: f32,
}

/// Profile query handler with optimized database access
#[derive(Debug, Clone)]
pub struct ProfileQueryHandler {
    db_pool: SqlitePool,
}

impl UserProfileByIdQuery {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

impl UserPreferencesQuery {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

impl ProfileQueryHandler {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    /// Handle user profile by ID query with optimized projection
    pub async fn handle_user_profile_by_id(
        &self,
        query: UserProfileByIdQuery,
    ) -> Result<Option<UserProfileView>, ProfileQueryError> {
        let row = sqlx::query(
            r#"
            SELECT id, family_size, skill_level, dietary_restrictions, updated_at
            FROM user_profiles 
            WHERE id = ?
            "#,
        )
        .bind(query.user_id.to_string())
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(row) = row {
            let family_size = FamilySize::new(row.get::<i64, _>("family_size") as u8)
                .map_err(|_| ProfileQueryError::InvalidData("family_size".to_string()))?;

            let skill_level = match row.get::<String, _>("skill_level").as_str() {
                "Beginner" => SkillLevel::Beginner,
                "Intermediate" => SkillLevel::Intermediate,
                "Advanced" => SkillLevel::Advanced,
                _ => SkillLevel::Beginner,
            };

            let dietary_restrictions: Vec<DietaryRestriction> =
                serde_json::from_str(row.get("dietary_restrictions")).unwrap_or_default();

            let last_updated = DateTime::parse_from_rfc3339(row.get("updated_at"))
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc);

            // Calculate profile completeness
            let profile_completeness = self.calculate_profile_completeness(
                &family_size,
                &skill_level,
                &dietary_restrictions,
            );

            Ok(Some(UserProfileView {
                user_id: query.user_id,
                family_size,
                cooking_skill_level: skill_level,
                dietary_restrictions,
                weekday_cooking_minutes: 30, // Default - could be stored in DB
                weekend_cooking_minutes: 60, // Default - could be stored in DB
                profile_completeness,
                last_updated,
            }))
        } else {
            Ok(None)
        }
    }

    /// Handle user preferences query optimized for meal planning
    pub async fn handle_user_preferences(
        &self,
        query: UserPreferencesQuery,
    ) -> Result<Option<UserPreferencesView>, ProfileQueryError> {
        let row = sqlx::query(
            r#"
            SELECT id, family_size, skill_level, dietary_restrictions
            FROM user_profiles 
            WHERE id = ?
            "#,
        )
        .bind(query.user_id.to_string())
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(row) = row {
            let family_size_value = row.get::<i64, _>("family_size") as u8;
            let family_size = FamilySize::new(family_size_value)
                .map_err(|_| ProfileQueryError::InvalidData("family_size".to_string()))?;

            let skill_level = match row.get::<String, _>("skill_level").as_str() {
                "Beginner" => SkillLevel::Beginner,
                "Intermediate" => SkillLevel::Intermediate,
                "Advanced" => SkillLevel::Advanced,
                _ => SkillLevel::Beginner,
            };

            let dietary_restrictions: Vec<DietaryRestriction> =
                serde_json::from_str(row.get("dietary_restrictions")).unwrap_or_default();

            // Create optimized profile for meal planning
            let profile = UserProfile {
                family_size,
                cooking_skill_level: skill_level,
                dietary_restrictions: dietary_restrictions.clone(),
                weekday_cooking_minutes: 30,
                weekend_cooking_minutes: 60,
            };

            let recommended_complexity = profile.get_recommended_complexity();
            let portion_multiplier = profile.calculate_portions(4) as f32 / 4.0; // Base 4 servings

            Ok(Some(UserPreferencesView {
                user_id: query.user_id,
                dietary_restrictions,
                family_size: family_size_value,
                skill_level,
                available_cooking_time_weekday: 30,
                available_cooking_time_weekend: 60,
                recommended_complexity,
                portion_multiplier,
            }))
        } else {
            Ok(None)
        }
    }

    /// Calculate profile completeness percentage
    fn calculate_profile_completeness(
        &self,
        family_size: &FamilySize,
        _skill_level: &SkillLevel,
        dietary_restrictions: &[DietaryRestriction],
    ) -> f32 {
        let mut completeness = 0.0;
        let total_fields = 4.0;

        // Family size is always present (required field)
        if family_size.value > 0 {
            completeness += 1.0;
        }

        // Skill level is always present (enum with default)
        completeness += 1.0;

        // Dietary restrictions are optional but add to completeness
        if !dietary_restrictions.is_empty() {
            completeness += 1.0;
        }

        // Additional field: cooking time preferences (would be separate if stored)
        completeness += 1.0; // Assume present for now

        (completeness / total_fields) * 100.0
    }

    /// Get profile statistics for analytics
    pub async fn get_profile_statistics(&self) -> Result<ProfileStatistics, ProfileQueryError> {
        let stats = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_profiles,
                AVG(family_size) as avg_family_size,
                COUNT(CASE WHEN skill_level = 'Beginner' THEN 1 END) as beginners,
                COUNT(CASE WHEN skill_level = 'Intermediate' THEN 1 END) as intermediate,
                COUNT(CASE WHEN skill_level = 'Advanced' THEN 1 END) as advanced
            FROM user_profiles
            "#,
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(ProfileStatistics {
            total_profiles: stats.get("total_profiles"),
            average_family_size: stats.get::<f64, _>("avg_family_size") as f32,
            skill_level_distribution: SkillLevelDistribution {
                beginners: stats.get("beginners"),
                intermediate: stats.get("intermediate"),
                advanced: stats.get("advanced"),
            },
        })
    }
}

/// Profile statistics for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStatistics {
    pub total_profiles: i64,
    pub average_family_size: f32,
    pub skill_level_distribution: SkillLevelDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLevelDistribution {
    pub beginners: i64,
    pub intermediate: i64,
    pub advanced: i64,
}

/// Error types for profile queries
#[derive(Debug, thiserror::Error)]
pub enum ProfileQueryError {
    #[error("Profile not found")]
    NotFound,

    #[error("Invalid data format for field: {0}")]
    InvalidData(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

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

        pool
    }

    async fn create_test_profile(pool: &SqlitePool) -> Uuid {
        let user_id = Uuid::new_v4();
        let dietary_restrictions = serde_json::to_string(&vec![
            DietaryRestriction::Vegetarian,
            DietaryRestriction::GlutenFree,
        ])
        .unwrap();

        sqlx::query(
            r#"
            INSERT INTO user_profiles 
            (id, email, password_hash, family_size, skill_level, dietary_restrictions, email_verified)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id.to_string())
        .bind("test@example.com")
        .bind("hashed_password")
        .bind(4i64)
        .bind("Intermediate")
        .bind(dietary_restrictions)
        .bind(true)
        .execute(pool)
        .await
        .expect("Failed to create test profile");

        user_id
    }

    #[tokio::test]
    async fn test_user_profile_by_id_query() {
        let pool = setup_test_db().await;
        let handler = ProfileQueryHandler::new(pool.clone());
        let user_id = create_test_profile(&pool).await;

        let query = UserProfileByIdQuery::new(user_id);
        let result = handler.handle_user_profile_by_id(query).await.unwrap();

        assert!(result.is_some());
        let profile = result.unwrap();
        assert_eq!(profile.user_id, user_id);
        assert_eq!(profile.family_size.value, 4);
        assert_eq!(profile.cooking_skill_level, SkillLevel::Intermediate);
        assert_eq!(profile.dietary_restrictions.len(), 2);
        assert!(profile.profile_completeness > 50.0);
    }

    #[tokio::test]
    async fn test_user_preferences_query() {
        let pool = setup_test_db().await;
        let handler = ProfileQueryHandler::new(pool.clone());
        let user_id = create_test_profile(&pool).await;

        let query = UserPreferencesQuery::new(user_id);
        let result = handler.handle_user_preferences(query).await.unwrap();

        assert!(result.is_some());
        let preferences = result.unwrap();
        assert_eq!(preferences.user_id, user_id);
        assert_eq!(preferences.family_size, 4);
        assert_eq!(preferences.skill_level, SkillLevel::Intermediate);
        assert_eq!(preferences.dietary_restrictions.len(), 2);
        assert_eq!(preferences.recommended_complexity, vec!["Easy", "Medium"]);
        assert_eq!(preferences.portion_multiplier, 1.0); // 4/4 = 1.0
    }

    #[tokio::test]
    async fn test_profile_statistics() {
        let pool = setup_test_db().await;
        let handler = ProfileQueryHandler::new(pool.clone());

        // Create multiple test profiles
        create_test_profile(&pool).await;

        let stats = handler.get_profile_statistics().await.unwrap();

        assert!(stats.total_profiles > 0);
        assert!(stats.average_family_size > 0.0);
    }

    #[tokio::test]
    async fn test_profile_completeness_calculation() {
        let pool = setup_test_db().await;
        let handler = ProfileQueryHandler::new(pool);

        let family_size = FamilySize::new(4).unwrap();
        let skill_level = SkillLevel::Intermediate;
        let dietary_restrictions = vec![DietaryRestriction::Vegetarian];

        let completeness = handler.calculate_profile_completeness(
            &family_size,
            &skill_level,
            &dietary_restrictions,
        );

        assert_eq!(completeness, 100.0); // All fields present
    }
}
