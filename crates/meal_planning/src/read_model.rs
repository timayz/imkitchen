use crate::aggregate::MealPlanAggregate;
use crate::events::{MealPlanGenerated, RecipeUsedInRotation};
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
}

/// Async evento subscription handler for MealPlanGenerated events
///
/// This handler projects MealPlanGenerated events from the evento event store
/// into the meal_plans and meal_assignments read model tables for efficient querying.
#[evento::handler(MealPlanAggregate)]
pub async fn meal_plan_generated_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Archive any existing active meal plans for this user
    sqlx::query(
        r#"
        UPDATE meal_plans
        SET status = 'archived'
        WHERE user_id = ?1 AND status = 'active'
        "#,
    )
    .bind(&event.data.user_id)
    .execute(&pool)
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
    .execute(&pool)
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
        .execute(&pool)
        .await?;
    }

    Ok(())
}

/// Async evento subscription handler for RecipeUsedInRotation events
///
/// This handler is primarily for tracking and potential future analytics.
/// The rotation state is already captured in the meal_plans.rotation_state column.
#[evento::handler(MealPlanAggregate)]
pub async fn recipe_used_in_rotation_handler<E: Executor>(
    _context: &Context<'_, E>,
    _event: EventDetails<RecipeUsedInRotation>,
) -> anyhow::Result<()> {
    // Rotation state already updated in meal_plan_generated_handler
    // This handler exists for potential cross-domain event subscriptions or analytics
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
}
