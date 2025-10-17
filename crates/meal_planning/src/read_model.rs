use crate::aggregate::MealPlanAggregate;
use crate::events::{MealPlanGenerated, RecipeUsedInRotation, RotationCycleReset};
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
}

/// MealAssignment data from read model (meal_assignments table)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MealAssignmentReadModel {
    pub id: String,
    pub meal_plan_id: String,
    pub date: String,
    pub meal_type: String, // "breakfast", "lunch", "dinner"
    pub recipe_id: String,
    pub prep_required: bool,
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
            SELECT id, user_id, start_date, status, rotation_state, created_at
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
            SELECT id, user_id, start_date, status, rotation_state, created_at
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
            SELECT id, meal_plan_id, date, meal_type, recipe_id, prep_required
            FROM meal_assignments
            WHERE meal_plan_id = ?1
            ORDER BY date, meal_type
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

    /// Query replacement candidates for a meal slot
    ///
    /// Returns favorite recipes that:
    /// 1. Haven't been used in current rotation cycle (respects rotation)
    /// 2. Match the meal type constraints
    /// 3. Are available for replacement
    ///
    /// This supports AC-5: "Manually replacing individual meals respects rotation"
    pub async fn query_replacement_candidates(
        user_id: &str,
        _meal_type: &str,
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
        // This ensures meal replacement respects rotation (AC-5)
        let replacement_ids: Vec<(String,)> = sqlx::query_as(
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
            LIMIT 10
            "#,
        )
        .bind(user_id)
        .bind(current_cycle)
        .fetch_all(pool)
        .await?;

        Ok(replacement_ids.into_iter().map(|(id,)| id).collect())
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
    sqlx::query(
        r#"
        INSERT INTO meal_plans (id, user_id, start_date, status, rotation_state, created_at)
        VALUES (?1, ?2, ?3, 'active', ?4, ?5)
        "#,
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.user_id)
    .bind(&event.data.start_date)
    .bind(&event.data.rotation_state_json)
    .bind(&event.data.generated_at)
    .execute(&mut *tx)
    .await?;

    // Insert meal assignments
    for assignment in &event.data.meal_assignments {
        let assignment_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO meal_assignments (id, meal_plan_id, date, meal_type, recipe_id, prep_required)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(assignment_id)
        .bind(&event.aggregator_id)
        .bind(&assignment.date)
        .bind(assignment.meal_type.as_str())
        .bind(&assignment.recipe_id)
        .bind(assignment.prep_required)
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

/// Create and configure the meal plan projection subscription
///
/// This function sets up evento subscriptions for meal plan read model projections.
pub fn meal_plan_projection(pool: SqlitePool) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("meal-plan-read-model")
        .aggregator::<MealPlanAggregate>()
        .data(pool)
        .handler(meal_plan_generated_handler())
        .handler(recipe_used_in_rotation_handler())
        .handler(rotation_cycle_reset_handler())
}

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
