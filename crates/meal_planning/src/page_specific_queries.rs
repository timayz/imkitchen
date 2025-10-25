/// Query functions for page-specific read models (Dashboard and Calendar pages)
///
/// These functions query the new page-specific tables (dashboard_meals, calendar_view)
/// instead of the old domain-centric tables (meal_assignments + JOIN recipes).
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Dashboard meal data (denormalized, no JOINs needed)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DashboardMeal {
    pub id: String,
    pub date: String,
    pub course_type: String,
    pub recipe_id: String,
    pub recipe_title: String,
    pub recipe_image_url: Option<String>,
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub prep_required: i32, // SQLite boolean (0 or 1)
}

/// Calendar view meal data (denormalized, no JOINs needed)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CalendarMeal {
    pub id: String,
    pub meal_plan_id: String,
    pub date: String,
    pub course_type: String,
    pub recipe_id: String,
    pub recipe_title: String,
    pub recipe_image_url: Option<String>,
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub assignment_reasoning: Option<String>,
}

/// Dashboard metrics (recipe counts, last plan date)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DashboardMetrics {
    pub recipe_count: i32,
    pub favorite_count: i32,
    pub cuisine_variety_count: i32,
    pub last_plan_generated_at: Option<String>,
}

/// Query today's meals for Dashboard page
///
/// Returns 3 meals (appetizer, main_course, dessert) for today's date.
/// No JOINs - all recipe metadata is denormalized in dashboard_meals table.
pub async fn get_todays_meals(
    user_id: &str,
    pool: &SqlitePool,
) -> Result<Vec<DashboardMeal>, sqlx::Error> {
    sqlx::query_as::<_, DashboardMeal>(
        r#"
        SELECT id, date, course_type, recipe_id, recipe_title, recipe_image_url,
               prep_time_min, cook_time_min, prep_required
        FROM dashboard_meals
        WHERE user_id = ?1 AND date = DATE('now')
        ORDER BY
            CASE course_type
                WHEN 'appetizer' THEN 1
                WHEN 'main_course' THEN 2
                WHEN 'dessert' THEN 3
            END
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Query dashboard metrics for user
///
/// Returns recipe counts and last plan generation timestamp.
pub async fn get_dashboard_metrics(
    user_id: &str,
    pool: &SqlitePool,
) -> Result<Option<DashboardMetrics>, sqlx::Error> {
    sqlx::query_as::<_, DashboardMetrics>(
        r#"
        SELECT recipe_count, favorite_count, cuisine_variety_count, last_plan_generated_at
        FROM dashboard_metrics
        WHERE user_id = ?1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Query week view meals for Calendar page
///
/// Returns all 21 meals (7 days × 3 courses) for the user's active meal plan.
/// No JOINs - all recipe metadata is denormalized in calendar_view table.
pub async fn get_calendar_week_view(
    user_id: &str,
    pool: &SqlitePool,
) -> Result<Vec<CalendarMeal>, sqlx::Error> {
    sqlx::query_as::<_, CalendarMeal>(
        r#"
        SELECT id, meal_plan_id, date, course_type, recipe_id, recipe_title,
               recipe_image_url, prep_time_min, cook_time_min, assignment_reasoning
        FROM calendar_view
        WHERE user_id = ?1
        ORDER BY date,
            CASE course_type
                WHEN 'appetizer' THEN 1
                WHEN 'main_course' THEN 2
                WHEN 'dessert' THEN 3
            END
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Query meal plan metadata (start date, meal_plan_id)
///
/// Returns the active meal plan ID and start date for the user.
/// This uses the meal_plans table (still domain-centric, used for plan metadata).
pub async fn get_active_meal_plan_metadata(
    user_id: &str,
    pool: &SqlitePool,
) -> Result<Option<(String, String)>, sqlx::Error> {
    sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT id, start_date
        FROM meal_plans
        WHERE user_id = ?1 AND status = 'active'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}
