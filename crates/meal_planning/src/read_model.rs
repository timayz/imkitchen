use crate::aggregate::MealPlanAggregate;
use crate::events::{
    MealPlanGenerated, MealPlanRegenerated, MultiWeekMealPlanGenerated, RecipeUsedInRotation,
    RotationCycleReset,
};
use evento::{AggregatorName, Context, EventDetails, Executor};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

/// MealPlan data from read model (meal_plans table)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MealPlanReadModel {
    pub id: String,
    pub user_id: String,
    pub start_date: String,
    pub status: String,         // "active" or "archived"
    pub rotation_state: String, // JSON
    pub created_at: String,
    pub updated_at: Option<String>, // Updated on regeneration
}

/// MealAssignment data from read model (meal_assignments table)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MealAssignmentReadModel {
    pub id: String,
    pub meal_plan_id: String,
    pub date: String,
    pub course_type: String, // AC-5: "appetizer", "main_course", "dessert" (renamed from meal_type)
    pub recipe_id: String,
    pub prep_required: bool,
    pub assignment_reasoning: Option<String>, // Story 3.8: Human-readable assignment explanation
}

/// MealPlanWithAssignments combines meal plan with its assignments for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealPlanWithAssignments {
    pub meal_plan: MealPlanReadModel,
    pub assignments: Vec<MealAssignmentReadModel>,
}

/// RecipeRotationState data from read model (recipe_rotation_state table)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecipeRotationStateRow {
    pub id: String,
    pub user_id: String,
    pub cycle_number: i64,
    pub recipe_id: String,
    pub used_at: String,
}

/// Query methods for meal plan read models
pub struct MealPlanQueries;

impl MealPlanQueries {
    /// Get active meal plan for user
    pub async fn get_active_meal_plan(
        user_id: &str,
        pool: &SqlitePool,
    ) -> Result<Option<MealPlanReadModel>, sqlx::Error> {
        sqlx::query_as::<_, MealPlanReadModel>(
            r#"
            SELECT id, user_id, start_date, status, rotation_state, created_at, updated_at
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

    /// Get meal plan by ID
    pub async fn get_meal_plan_by_id(
        meal_plan_id: &str,
        pool: &SqlitePool,
    ) -> Result<Option<MealPlanReadModel>, sqlx::Error> {
        sqlx::query_as::<_, MealPlanReadModel>(
            r#"
            SELECT id, user_id, start_date, status, rotation_state, created_at, updated_at
            FROM meal_plans
            WHERE id = ?1
            "#,
        )
        .bind(meal_plan_id)
        .fetch_optional(pool)
        .await
    }

    /// Get meal assignments for a meal plan
    pub async fn get_meal_assignments(
        meal_plan_id: &str,
        pool: &SqlitePool,
    ) -> Result<Vec<MealAssignmentReadModel>, sqlx::Error> {
        sqlx::query_as::<_, MealAssignmentReadModel>(
            r#"
            SELECT id, meal_plan_id, date, course_type, recipe_id, prep_required, assignment_reasoning
            FROM meal_assignments
            WHERE meal_plan_id = ?1
            ORDER BY date, course_type
            "#,
        )
        .bind(meal_plan_id)
        .fetch_all(pool)
        .await
    }

    /// Get active meal plan with assignments for user
    pub async fn get_active_meal_plan_with_assignments(
        user_id: &str,
        pool: &SqlitePool,
    ) -> Result<Option<MealPlanWithAssignments>, sqlx::Error> {
        let meal_plan = Self::get_active_meal_plan(user_id, pool).await?;

        match meal_plan {
            Some(plan) => {
                let assignments = Self::get_meal_assignments(&plan.id, pool).await?;
                Ok(Some(MealPlanWithAssignments {
                    meal_plan: plan,
                    assignments,
                }))
            }
            None => Ok(None),
        }
    }

    /// Query rotation state for a user's current cycle
    ///
    /// Returns a RotationState struct with the current cycle number and used recipe IDs.
    /// If no rotation state exists, returns a fresh RotationState::new().
    pub async fn query_rotation_state(
        user_id: &str,
        pool: &SqlitePool,
    ) -> Result<crate::rotation::RotationState, sqlx::Error> {
        // Get the max cycle number for this user
        let max_cycle: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT MAX(cycle_number) as max_cycle
            FROM recipe_rotation_state
            WHERE user_id = ?1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let current_cycle = max_cycle.map(|(c,)| c).unwrap_or(1);

        // Get all used recipe IDs for the current cycle
        let rows: Vec<RecipeRotationStateRow> = sqlx::query_as(
            r#"
            SELECT id, user_id, cycle_number, recipe_id, used_at
            FROM recipe_rotation_state
            WHERE user_id = ?1 AND cycle_number = ?2
            "#,
        )
        .bind(user_id)
        .bind(current_cycle)
        .fetch_all(pool)
        .await?;

        // Build RotationState from query results
        let mut rotation_state = crate::rotation::RotationState::new();
        rotation_state.cycle_number = current_cycle as u32;

        for row in rows {
            rotation_state.mark_recipe_used(row.recipe_id);
        }

        Ok(rotation_state)
    }

    /// Query available recipes for rotation (unused in current cycle)
    ///
    /// Returns a list of recipe IDs that are favorite recipes but not yet used
    /// in the current rotation cycle.
    pub async fn query_available_recipes_for_rotation(
        user_id: &str,
        pool: &SqlitePool,
    ) -> Result<Vec<String>, sqlx::Error> {
        // Get current cycle number
        let max_cycle: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT MAX(cycle_number) as max_cycle
            FROM recipe_rotation_state
            WHERE user_id = ?1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let current_cycle = max_cycle.map(|(c,)| c).unwrap_or(1);

        // Get favorite recipes NOT IN current rotation state used set
        let available_recipe_ids: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT r.id
            FROM recipes r
            WHERE r.user_id = ?1
              AND r.is_favorite = TRUE
              AND r.id NOT IN (
                  SELECT recipe_id
                  FROM recipe_rotation_state
                  WHERE user_id = ?1 AND cycle_number = ?2
              )
            ORDER BY r.title
            "#,
        )
        .bind(user_id)
        .bind(current_cycle)
        .fetch_all(pool)
        .await?;

        Ok(available_recipe_ids.into_iter().map(|(id,)| id).collect())
    }

    /// Query rotation progress for display (AC-8)
    ///
    /// Returns (used_count, total_favorites) tuple for showing progress like:
    /// "Recipe variety: 12 of 20 favorites used this cycle"
    pub async fn query_rotation_progress(
        user_id: &str,
        pool: &SqlitePool,
    ) -> Result<(usize, usize), sqlx::Error> {
        // Get current cycle number
        let max_cycle: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT MAX(cycle_number) as max_cycle
            FROM recipe_rotation_state
            WHERE user_id = ?1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let current_cycle = max_cycle.map(|(c,)| c).unwrap_or(1);

        // Count recipes used in current cycle
        let used_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM recipe_rotation_state
            WHERE user_id = ?1 AND cycle_number = ?2
            "#,
        )
        .bind(user_id)
        .bind(current_cycle)
        .fetch_one(pool)
        .await?;

        // Count total favorite recipes
        let total_favorites: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM recipes
            WHERE user_id = ?1 AND is_favorite = TRUE AND deleted_at IS NULL
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok((used_count.0 as usize, total_favorites.0 as usize))
    }

    /// Get today's meals with recipe details (Story 3.9)
    ///
    /// Returns meal assignments for the current date with recipe details via JOIN.
    /// Used for dashboard "Today's Meals" section display.
    ///
    /// Query returns assignments ordered by meal_type (breakfast, lunch, dinner).
    pub async fn get_todays_meals(
        user_id: &str,
        pool: &SqlitePool,
    ) -> Result<Vec<MealAssignmentWithRecipe>, sqlx::Error> {
        sqlx::query_as::<_, MealAssignmentWithRecipe>(
            r#"
            SELECT
                ma.id,
                ma.meal_plan_id,
                ma.date,
                ma.course_type,
                ma.recipe_id,
                ma.prep_required,
                ma.assignment_reasoning,
                r.title as recipe_title,
                r.prep_time_min,
                r.cook_time_min,
                r.advance_prep_hours,
                r.complexity
            FROM meal_assignments ma
            INNER JOIN recipes r ON ma.recipe_id = r.id
            INNER JOIN meal_plans mp ON ma.meal_plan_id = mp.id
            WHERE mp.user_id = ?1
              AND mp.status = 'active'
              AND ma.date = DATE('now')
            ORDER BY
              CASE ma.course_type
                WHEN 'appetizer' THEN 1
                WHEN 'main_course' THEN 2
                WHEN 'dessert' THEN 3
                WHEN 'breakfast' THEN 1
                WHEN 'lunch' THEN 2
                WHEN 'dinner' THEN 3
                ELSE 4
              END
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// Get all active weeks for multi-week calendar view (Story 9.1, Epic 8)
    ///
    /// Returns a list of WeekReadModel for all active weeks in the meal plan batch,
    /// ordered by start_date ascending (Week 1, Week 2, etc.).
    ///
    /// This query is used to populate the week tabs and carousel navigation
    /// in the multi-week calendar template.
    pub async fn get_active_weeks(
        user_id: &str,
        pool: &SqlitePool,
    ) -> Result<Vec<WeekReadModel>, sqlx::Error> {
        sqlx::query_as::<_, WeekReadModel>(
            r#"
            SELECT id, start_date, end_date, status, is_locked
            FROM meal_plans
            WHERE user_id = ?1 AND status = 'active'
            ORDER BY start_date ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// Get a specific week by ID for partial week content rendering (Story 9.1, Epic 8)
    ///
    /// Used for TwinSpark partial updates when user clicks a different week tab.
    pub async fn get_week_by_id(
        week_id: &str,
        pool: &SqlitePool,
    ) -> Result<Option<WeekReadModel>, sqlx::Error> {
        sqlx::query_as::<_, WeekReadModel>(
            r#"
            SELECT id, start_date, end_date, status, is_locked
            FROM meal_plans
            WHERE id = ?1
            "#,
        )
        .bind(week_id)
        .fetch_optional(pool)
        .await
    }
}

/// MealAssignment with Recipe details for today's meals display (Story 3.9)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MealAssignmentWithRecipe {
    pub id: String,
    pub meal_plan_id: String,
    pub date: String,
    pub course_type: String, // AC-5: Renamed from meal_type
    pub recipe_id: String,
    pub prep_required: bool,
    pub assignment_reasoning: Option<String>,
    pub recipe_title: String,
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub advance_prep_hours: Option<i32>,
    pub complexity: Option<String>,
}

/// Week data for multi-week calendar view (Story 9.1, Epic 8)
///
/// Represents a single week in the multi-week calendar, containing
/// week metadata (dates, status, lock state) for template rendering.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WeekReadModel {
    pub id: String,
    pub start_date: String, // Format: YYYY-MM-DD
    pub end_date: String,   // Format: YYYY-MM-DD
    pub status: String,     // "active" or "archived"
    pub is_locked: bool,    // true if current week (locked from regeneration)
}

/// Async evento subscription handler for MealPlanGenerated events
///
/// This handler projects MealPlanGenerated events from the evento event store
/// into the meal_plans and meal_assignments read model tables for efficient querying.
///
/// **Critical Fix 1.1:** Added database transaction and idempotency check to ensure
/// data consistency and prevent duplicate processing on event replay.
#[evento::handler(MealPlanAggregate)]
pub async fn meal_plan_generated_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Idempotency check: Skip if this event has already been processed
    let exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM meal_plans WHERE id = ?1")
        .bind(&event.aggregator_id)
        .fetch_optional(&pool)
        .await?;

    if exists.is_some() {
        // Event already processed, skip to maintain idempotency
        return Ok(());
    }

    // Begin transaction for atomic updates
    let mut tx = pool.begin().await?;

    // Archive any existing active meal plans for this user
    sqlx::query(
        r#"
        UPDATE meal_plans
        SET status = 'archived'
        WHERE user_id = ?1 AND status = 'active'
        "#,
    )
    .bind(&event.data.user_id)
    .execute(&mut *tx)
    .await?;

    // Insert meal plan into read model
    // Use event.timestamp for created_at and updated_at (both same on initial generation)
    let timestamp_rfc3339 = chrono::DateTime::from_timestamp(event.timestamp, 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?
        .to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO meal_plans (id, user_id, start_date, status, rotation_state, created_at, updated_at)
        VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?5)
        "#,
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.user_id)
    .bind(&event.data.start_date)
    .bind(&event.data.rotation_state_json)
    .bind(&timestamp_rfc3339)
    .execute(&mut *tx)
    .await?;

    // Insert meal assignments
    for assignment in &event.data.meal_assignments {
        let assignment_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO meal_assignments (id, meal_plan_id, date, course_type, recipe_id, prep_required, assignment_reasoning)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(assignment_id)
        .bind(&event.aggregator_id)
        .bind(&assignment.date)
        .bind(&assignment.course_type) // AC-4: Changed from meal_type
        .bind(&assignment.recipe_id)
        .bind(assignment.prep_required)
        .bind(&assignment.assignment_reasoning)
        .execute(&mut *tx)
        .await?;
    }

    // Commit transaction - all or nothing
    tx.commit().await?;

    Ok(())
}

/// Async evento subscription handler for RecipeUsedInRotation events
///
/// This handler projects RecipeUsedInRotation events into the recipe_rotation_state
/// read model table for rotation tracking queries and analytics.
#[evento::handler(MealPlanAggregate)]
pub async fn recipe_used_in_rotation_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeUsedInRotation>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Get user_id from the meal plan by querying the aggregate's user_id
    // We need to derive user_id from the aggregator_id (meal_plan_id)
    let user_id: Option<(String,)> = sqlx::query_as("SELECT user_id FROM meal_plans WHERE id = ?1")
        .bind(&event.aggregator_id)
        .fetch_optional(&pool)
        .await?;

    let user_id = match user_id {
        Some((uid,)) => uid,
        None => {
            // If meal plan not found yet (race condition), skip this event
            // It will be reprocessed on next subscription run
            return Ok(());
        }
    };

    // Insert into recipe_rotation_state table
    // Use ON CONFLICT DO NOTHING for idempotency (unique constraint on user_id, cycle_number, recipe_id)
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO recipe_rotation_state (id, user_id, cycle_number, recipe_id, used_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(user_id, cycle_number, recipe_id) DO NOTHING
        "#,
    )
    .bind(id)
    .bind(&user_id)
    .bind(event.data.cycle_number as i64)
    .bind(&event.data.recipe_id)
    .bind(&event.data.used_at)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Async evento subscription handler for RotationCycleReset events
///
/// This handler clears rotation state for the old cycle when a reset occurs.
/// It deletes all recipe_rotation_state entries for the old cycle number.
#[evento::handler(MealPlanAggregate)]
pub async fn rotation_cycle_reset_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RotationCycleReset>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Delete all rotation state entries for the old cycle
    sqlx::query(
        r#"
        DELETE FROM recipe_rotation_state
        WHERE user_id = ?1 AND cycle_number = ?2
        "#,
    )
    .bind(&event.data.user_id)
    .bind(event.data.old_cycle_number as i64)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Async evento subscription handler for MealPlanRegenerated events (Story 3.7)
///
/// This handler projects MealPlanRegenerated events to replace all meal_assignments
/// for the meal plan with freshly generated assignments.
///
/// **Critical Operations:**
/// 1. DELETE all existing assignments for the meal plan
/// 2. INSERT new assignments from event
/// 3. UPDATE rotation_state in meal_plans table
/// 4. All in a single atomic transaction
#[evento::handler(MealPlanAggregate)]
pub async fn meal_plan_regenerated_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanRegenerated>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Begin transaction for atomic updates
    let mut tx = pool.begin().await?;

    // DELETE all existing meal assignments for this meal plan
    sqlx::query(
        r#"
        DELETE FROM meal_assignments
        WHERE meal_plan_id = ?1
        "#,
    )
    .bind(&event.aggregator_id)
    .execute(&mut *tx)
    .await?;

    // INSERT new meal assignments from regeneration
    for assignment in &event.data.new_assignments {
        let assignment_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO meal_assignments (id, meal_plan_id, date, course_type, recipe_id, prep_required, assignment_reasoning)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(assignment_id)
        .bind(&event.aggregator_id)
        .bind(&assignment.date)
        .bind(&assignment.course_type) // AC-4: Changed from meal_type
        .bind(&assignment.recipe_id)
        .bind(assignment.prep_required)
        .bind(&assignment.assignment_reasoning)
        .execute(&mut *tx)
        .await?;
    }

    // UPDATE rotation_state and updated_at in meal_plans table (cycle preserved)
    // Use event.timestamp for updated_at to track when regeneration occurred
    let timestamp_rfc3339 = chrono::DateTime::from_timestamp(event.timestamp, 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?
        .to_rfc3339();

    sqlx::query(
        r#"
        UPDATE meal_plans
        SET rotation_state = ?1, updated_at = ?2
        WHERE id = ?3
        "#,
    )
    .bind(&event.data.rotation_state_json)
    .bind(&timestamp_rfc3339)
    .bind(&event.aggregator_id)
    .execute(&mut *tx)
    .await?;

    // Commit transaction - all or nothing
    tx.commit().await?;

    Ok(())
}

/// Async evento subscription handler for MultiWeekMealPlanGenerated events (Story 6.6 AC-1, AC-2)
///
/// This handler projects MultiWeekMealPlanGenerated events into meal_plans and meal_assignments
/// read model tables. Unlike MealPlanGenerated (single week), this handler processes multiple weeks
/// (1-5 maximum) in a single batch with rotation tracking across weeks.
///
/// **Key Operations:**
/// 1. Iterate over weeks in event.data.weeks vector
/// 2. Calculate week status from dates (Future/Current/Past)
/// 3. Insert each week into meal_plans table with generation_batch_id
/// 4. Insert 21 meal assignments per week (7 days × 3 courses)
/// 5. Store accompaniment_recipe_id for main courses
/// 6. Serialize rotation_state to JSON for persistence
#[evento::handler(MealPlanAggregate)]
pub async fn multi_week_meal_plan_generated_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MultiWeekMealPlanGenerated>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Begin transaction for atomic updates across all weeks
    let mut tx = pool.begin().await?;

    // Archive any existing active meal plans for this user
    sqlx::query(
        r#"
        UPDATE meal_plans
        SET status = 'archived'
        WHERE user_id = ?1 AND status = 'active'
        "#,
    )
    .bind(&event.data.user_id)
    .execute(&mut *tx)
    .await?;

    // Serialize rotation_state to JSON
    let rotation_state_json = serde_json::to_string(&event.data.rotation_state)?;

    // Get current date for status calculation
    let now = chrono::Utc::now().date_naive();

    // Process each week in the batch
    for week_data in &event.data.weeks {
        // Calculate week status from dates
        let start_date = chrono::NaiveDate::parse_from_str(&week_data.start_date, "%Y-%m-%d")
            .map_err(|e| anyhow::anyhow!("Invalid start_date format: {}", e))?;
        let end_date = chrono::NaiveDate::parse_from_str(&week_data.end_date, "%Y-%m-%d")
            .map_err(|e| anyhow::anyhow!("Invalid end_date format: {}", e))?;

        // Calculate actual status (Future/Current/Past for business logic)
        let actual_status = if start_date > now {
            "future"
        } else if end_date < now {
            "past"
        } else {
            "current"
        };

        // Map to database status (only 'active' or 'archived' allowed by CHECK constraint)
        // Future and Current weeks are 'active', Past weeks are 'archived'
        let db_status = if actual_status == "past" {
            "archived"
        } else {
            "active"
        };

        // Determine if week is locked (current week is locked)
        let is_locked = actual_status == "current";

        // Use event timestamp for created_at
        let timestamp_rfc3339 = chrono::DateTime::from_timestamp(event.timestamp, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?
            .to_rfc3339();

        // Insert week into meal_plans table
        sqlx::query(
            r#"
            INSERT INTO meal_plans (
                id, user_id, start_date, end_date, status, is_locked,
                generation_batch_id, rotation_state_json, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)
            "#,
        )
        .bind(&week_data.id)
        .bind(&event.data.user_id)
        .bind(&week_data.start_date)
        .bind(&week_data.end_date)
        .bind(db_status)
        .bind(is_locked)
        .bind(&event.data.generation_batch_id)
        .bind(&rotation_state_json)
        .bind(&timestamp_rfc3339)
        .execute(&mut *tx)
        .await?;

        // Insert 21 meal assignments for this week
        for assignment in &week_data.meal_assignments {
            let assignment_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO meal_assignments (
                    id, meal_plan_id, date, course_type, recipe_id, prep_required,
                    assignment_reasoning, accompaniment_recipe_id
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
            )
            .bind(assignment_id)
            .bind(&week_data.id)
            .bind(&assignment.date)
            .bind(&assignment.course_type)
            .bind(&assignment.recipe_id)
            .bind(assignment.prep_required)
            .bind(&assignment.assignment_reasoning)
            .bind(&assignment.accompaniment_recipe_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    // Commit transaction - all weeks and assignments inserted atomically
    tx.commit().await?;

    Ok(())
}

/// Create and configure the meal plan projection subscription
///
/// This function sets up evento subscriptions for meal plan read model projections.
pub fn meal_plan_projection(pool: SqlitePool) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("meal-plan-read-model")
        .data(pool)
        .handler(meal_plan_generated_handler())
        .handler(recipe_used_in_rotation_handler())
        .handler(rotation_cycle_reset_handler())
        .handler(meal_plan_regenerated_handler())
        .handler(multi_week_meal_plan_generated_handler())
}

// NOTE: Integration tests for multi-week meal plan projections (Story 6.6 AC-8)
//
// Comprehensive integration tests for the multi_week_meal_plan_generated_handler()
// are located in tests/multi_week_projection_tests.rs (TODO: to be created)
//
// These tests should verify:
// 1. Multiple weeks (1-5) inserted atomically into meal_plans table
// 2. All weeks share same generation_batch_id for batch tracking
// 3. 21 meal assignments per week inserted into meal_assignments table
// 4. Week status (Future/Current/Past) calculated correctly from dates
// 5. is_locked field set correctly (current week locked, future weeks unlocked)
// 6. accompaniment_recipe_id stored for main courses (Story 6.3 AC-8)
// 7. rotation_state serialized to JSON and can be deserialized
// 8. Transaction rollback on error (all-or-nothing semantics)
// 9. Previous active meal plans archived before inserting new batch
//
// Test pattern should follow existing tests in tests/persistence_tests.rs using unsafe_oneshot()
// for synchronous evento processing during tests.

// NOTE: Task 7 (AC-6, AC-7) - Favorite Recipe Changes Mid-Rotation
//
// To handle favorite recipe changes mid-rotation, we need cross-domain event subscriptions:
//
// 1. Subscribe to RecipeFavorited events from Recipe aggregate:
//    - When a new recipe is favorited, it becomes immediately available in rotation pool
//    - No need to mark as "used" - it starts fresh in the current cycle
//    - Update total_favorite_count in active rotation state
//
// 2. Subscribe to RecipeUnfavorited events from Recipe aggregate:
//    - Remove recipe from recipe_rotation_state if present in current cycle
//    - Update total_favorite_count in active rotation state
//    - If recipe is currently assigned in active meal plan, keep the assignment
//      but it won't appear in future meal plan generations
//
// Implementation pattern (when Recipe domain events are available):
//
// #[evento::handler(RecipeAggregate)]
// pub async fn recipe_favorited_handler<E: Executor>(
//     context: &Context<'_, E>,
//     event: EventDetails<RecipeFavorited>,
// ) -> anyhow::Result<()> {
//     let pool: SqlitePool = context.extract();
//     let user_id = event.metadata.clone();
//
//     // Recipe automatically available in rotation - no action needed in recipe_rotation_state
//     // The query_available_recipes_for_rotation() will include it automatically
//
//     Ok(())
// }
//
// #[evento::handler(RecipeAggregate)]
// pub async fn recipe_unfavorited_handler<E: Executor>(
//     context: &Context<'_, E>,
//     event: EventDetails<RecipeUnfavorited>,
// ) -> anyhow::Result<()> {
//     let pool: SqlitePool = context.extract();
//     let user_id = event.metadata.clone();
//
//     // Remove from rotation state if present
//     sqlx::query(
//         r#"
//         DELETE FROM recipe_rotation_state
//         WHERE user_id = ?1 AND recipe_id = ?2
//         "#,
//     )
//     .bind(&user_id)
//     .bind(&event.data.recipe_id)
//     .execute(&pool)
//     .await?;
//
//     Ok(())
// }
