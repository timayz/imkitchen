/// Page-Specific Projection Handlers for Meal Planning Domain
///
/// This module contains evento projection handlers that populate page-specific read model tables
/// (dashboard_meals, dashboard_prep_tasks, dashboard_metrics, calendar_view) from meal planning events.
use crate::aggregate::MealPlanAggregate;
use crate::events::{
    MealPlanArchived, MealPlanGenerated, MealPlanRegenerated, RecipeUsedInRotation,
    RotationCycleReset,
};
use evento::{AggregatorName, Context, EventDetails, Executor};
use sqlx::SqlitePool;

/// Create evento subscription for dashboard page projections
///
/// Registers handlers for dashboard_meals, dashboard_prep_tasks, and dashboard_metrics tables.
pub fn meal_plan_dashboard_projections(pool: SqlitePool) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("meal-plan-dashboard-projections")
        .aggregator::<MealPlanAggregate>()
        .data(pool)
        .handler(handle_meal_plan_generated_for_dashboard())
        .handler(reproject_dashboard_meals_on_regenerate())
        .skip::<MealPlanAggregate, RecipeUsedInRotation>()
        .skip::<MealPlanAggregate, RotationCycleReset>()
        .skip::<MealPlanAggregate, MealPlanArchived>()
}

/// Create evento subscription for calendar page projections
///
/// Registers handlers for calendar_view table.
pub fn meal_plan_calendar_projections(pool: SqlitePool) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("meal-plan-calendar-projections")
        .aggregator::<MealPlanAggregate>()
        .data(pool)
        .handler(project_meal_plan_to_calendar_view())
        .handler(reproject_calendar_view_on_regenerate())
        .skip::<MealPlanAggregate, RecipeUsedInRotation>()
        .skip::<MealPlanAggregate, RotationCycleReset>()
        .skip::<MealPlanAggregate, MealPlanArchived>()
}

// =============================================================================
// DASHBOARD PAGE PROJECTIONS
// =============================================================================

/// Handle MealPlanGenerated event for dashboard: update meals, prep tasks, and metrics
///
/// This handler combines 3 operations that all need to run on MealPlanGenerated:
/// 1. Insert/update dashboard_meals (today's 3 meals)
/// 2. Insert dashboard_prep_tasks (today's prep tasks)
/// 3. Update dashboard_metrics (last_plan_generated_at)
///
/// Note: evento only allows ONE handler per event type per subscription
#[evento::handler(MealPlanAggregate)]
pub async fn handle_meal_plan_generated_for_dashboard<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let today_date = chrono::Utc::now().date_naive();

    // Operation 1: Insert/update dashboard_meals (TODAY'S meals only)
    for assignment in &event.data.meal_assignments {
#[allow(clippy::type_complexity)]
        if assignment.date == today {
            let recipe: Option<(String, Option<String>, Option<i32>, Option<i32>)> = sqlx::query_as(
                "SELECT title, image_url, prep_time_min, cook_time_min FROM recipe_list WHERE id = ?",
            )
            .bind(&assignment.recipe_id)
            .fetch_optional(&pool)
            .await?;

            if let Some((title, image_url, prep_time_min, cook_time_min)) = recipe {
                sqlx::query(
                    r#"
                    INSERT INTO dashboard_meals (
                        id, user_id, date, course_type, recipe_id,
                        recipe_title, recipe_image_url, prep_time_min, cook_time_min,
                        prep_required, created_at, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
                    ON CONFLICT(id) DO UPDATE SET
                        recipe_id = ?5,
                        recipe_title = ?6,
                        recipe_image_url = ?7,
                        prep_time_min = ?8,
                        cook_time_min = ?9,
                        prep_required = ?10,
                        updated_at = ?11
                    "#,
                )
                .bind(format!("{}-{}", assignment.date, assignment.course_type))
                .bind(&event.data.user_id)
                .bind(&assignment.date)
                .bind(&assignment.course_type)
                .bind(&assignment.recipe_id)
                .bind(&title)
                .bind(&image_url)
                .bind(prep_time_min)
                .bind(cook_time_min)
                .bind(if assignment.prep_required { 1 } else { 0 })
                .bind(&event.data.generated_at)
                .execute(&pool)
                .await?;
            }
        }
    }

    // Operation 2: Insert dashboard_prep_tasks (TODAY'S tasks only)
    for assignment in &event.data.meal_assignments {
        if assignment.prep_required {
            let meal_date = chrono::NaiveDate::parse_from_str(&assignment.date, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid date format: {}", e))?;

            let recipe: Option<(String, Option<i32>)> = sqlx::query_as(
                "SELECT title, advance_prep_hours FROM recipe_detail WHERE id = ?",
            )
            .bind(&assignment.recipe_id)
            .fetch_optional(&pool)
            .await?;

            if let Some((title, Some(advance_prep_hours))) = recipe {
                let task_date = meal_date - chrono::Duration::hours(advance_prep_hours as i64);

                if task_date == today_date {
                    sqlx::query(
                        r#"
                        INSERT INTO dashboard_prep_tasks (
                            id, user_id, task_date, meal_date, recipe_id,
                            recipe_title, prep_description, advance_prep_hours, created_at
                        )
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                        ON CONFLICT(id) DO UPDATE SET
                            recipe_id = ?5,
                            recipe_title = ?6,
                            prep_description = ?7,
                            advance_prep_hours = ?8
                        "#,
                    )
                    .bind(format!("{}-{}", task_date, assignment.recipe_id))
                    .bind(&event.data.user_id)
                    .bind(task_date.to_string())
                    .bind(&assignment.date)
                    .bind(&assignment.recipe_id)
                    .bind(&title)
                    .bind(format!("Prep {} hours before meal", advance_prep_hours))
                    .bind(advance_prep_hours)
                    .bind(&event.data.generated_at)
                    .execute(&pool)
                    .await?;
                }
            }
        }
    }

    // Operation 3: Update dashboard_metrics (last_plan_generated_at)
    sqlx::query(
        r#"
        INSERT INTO dashboard_metrics (
            user_id, recipe_count, favorite_count, cuisine_variety_count,
            last_plan_generated_at, updated_at
        )
        VALUES (?1, 0, 0, 0, ?2, ?2)
        ON CONFLICT(user_id) DO UPDATE SET
            last_plan_generated_at = ?2,
            updated_at = ?2
        "#,
    )
    .bind(&event.data.user_id)
    .bind(&event.data.generated_at)
    .execute(&pool)
    .await?;

    Ok(())
}

/// DEPRECATED: Split into handle_meal_plan_generated_for_dashboard
#[allow(dead_code)]
#[evento::handler(MealPlanAggregate)]
pub async fn project_meal_plan_to_dashboard_meals<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    for assignment in &event.data.meal_assignments {
        if assignment.date == today {
            // Fetch recipe metadata from recipe_list (denormalized for dashboard)
#[allow(clippy::type_complexity)]
            let recipe: Option<(String, Option<String>, Option<i32>, Option<i32>)> = sqlx::query_as(
                "SELECT title, image_url, prep_time_min, cook_time_min FROM recipe_list WHERE id = ?",
            )
            .bind(&assignment.recipe_id)
            .fetch_optional(&pool)
            .await?;

            if let Some((title, image_url, prep_time_min, cook_time_min)) = recipe {
                sqlx::query(
                    r#"
                    INSERT INTO dashboard_meals (
                        id, user_id, date, course_type, recipe_id,
                        recipe_title, recipe_image_url, prep_time_min, cook_time_min,
                        prep_required, created_at, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
                    ON CONFLICT(id) DO UPDATE SET
                        recipe_id = ?5,
                        recipe_title = ?6,
                        recipe_image_url = ?7,
                        prep_time_min = ?8,
                        cook_time_min = ?9,
                        prep_required = ?10,
                        updated_at = ?11
                    "#,
                )
                .bind(format!("{}-{}", assignment.date, assignment.course_type))
                .bind(&event.data.user_id)
                .bind(&assignment.date)
                .bind(&assignment.course_type)
                .bind(&assignment.recipe_id)
                .bind(&title)
                .bind(&image_url)
                .bind(prep_time_min)
                .bind(cook_time_min)
                .bind(if assignment.prep_required { 1 } else { 0 })
                .bind(&event.data.generated_at)
                .execute(&pool)
                .await?;
            }
        }
    }

    Ok(())
}

/// Project MealPlanGenerated event to dashboard_prep_tasks table (TODAY'S tasks only)
///
#[allow(dead_code)]
/// Extract prep tasks for today from meals that require advance prep.
#[evento::handler(MealPlanAggregate)]
pub async fn project_meal_plan_to_dashboard_prep_tasks<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    let today = chrono::Utc::now().date_naive();

    for assignment in &event.data.meal_assignments {
        if assignment.prep_required {
            // Parse meal date
            let meal_date = chrono::NaiveDate::parse_from_str(&assignment.date, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid date format: {}", e))?;

            // Fetch recipe metadata and advance_prep_hours
            let recipe: Option<(String, Option<i32>)> = sqlx::query_as(
                "SELECT title, advance_prep_hours FROM recipe_detail WHERE id = ?",
            )
            .bind(&assignment.recipe_id)
            .fetch_optional(&pool)
            .await?;

            if let Some((title, Some(advance_prep_hours))) = recipe {
                // Calculate task date (when prep should be done)
                let task_date = meal_date - chrono::Duration::hours(advance_prep_hours as i64);

                // Only insert if task_date == today
                if task_date == today {
                    sqlx::query(
                        r#"
                        INSERT INTO dashboard_prep_tasks (
                            id, user_id, task_date, meal_date, recipe_id,
                            recipe_title, prep_description, advance_prep_hours, created_at
                        )
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                        ON CONFLICT(id) DO UPDATE SET
                            recipe_id = ?5,
                            recipe_title = ?6,
                            prep_description = ?7,
                            advance_prep_hours = ?8
                        "#,
                    )
                    .bind(format!("{}-{}", task_date, assignment.recipe_id))
                    .bind(&event.data.user_id)
                    .bind(task_date.to_string())
                    .bind(&assignment.date)
                    .bind(&assignment.recipe_id)
                    .bind(&title)
                    .bind(format!("Prep {} hours before meal", advance_prep_hours))
                    .bind(advance_prep_hours)
                    .bind(&event.data.generated_at)
                    .execute(&pool)
                    .await?;
                }
            }
        }
    }

    Ok(())
}

/// Update dashboard_metrics when MealPlanGenerated fires (update last_plan_generated_at)
#[allow(dead_code)]
#[evento::handler(MealPlanAggregate)]
pub async fn update_dashboard_metrics_on_plan_generated<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query(
        r#"
        INSERT INTO dashboard_metrics (
            user_id, recipe_count, favorite_count, cuisine_variety_count,
            last_plan_generated_at, updated_at
        )
        VALUES (?1, 0, 0, 0, ?2, ?2)
        ON CONFLICT(user_id) DO UPDATE SET
            last_plan_generated_at = ?2,
            updated_at = ?2
        "#,
    )
    .bind(&event.data.user_id)
    .bind(&event.data.generated_at)
    .execute(&pool)
    .await?;

    Ok(())
}

// =============================================================================
// CALENDAR PAGE PROJECTIONS
// =============================================================================

/// Project MealPlanGenerated event to calendar_view table (FULL WEEK)
///
/// Calendar page shows all 7 days × 3 courses = 21 assignments.
#[evento::handler(MealPlanAggregate)]
pub async fn project_meal_plan_to_calendar_view<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    for assignment in &event.data.meal_assignments {
#[allow(clippy::type_complexity)]
        // Fetch recipe metadata from recipe_list
        let recipe: Option<(String, Option<String>, Option<i32>, Option<i32>)> = sqlx::query_as(
            "SELECT title, image_url, prep_time_min, cook_time_min FROM recipe_list WHERE id = ?",
        )
        .bind(&assignment.recipe_id)
        .fetch_optional(&pool)
        .await?;

        if let Some((title, image_url, prep_time_min, cook_time_min)) = recipe {
            sqlx::query(
                r#"
                INSERT INTO calendar_view (
                    id, user_id, meal_plan_id, date, course_type, recipe_id,
                    recipe_title, recipe_image_url, prep_time_min, cook_time_min,
                    assignment_reasoning, created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ON CONFLICT(id) DO UPDATE SET
                    recipe_id = ?6,
                    recipe_title = ?7,
                    recipe_image_url = ?8,
                    prep_time_min = ?9,
                    cook_time_min = ?10,
                    assignment_reasoning = ?11,
                    created_at = ?12
                "#,
            )
            .bind(format!("{}-{}", assignment.date, assignment.course_type))
            .bind(&event.data.user_id)
            .bind(&event.aggregator_id)
            .bind(&assignment.date)
            .bind(&assignment.course_type)
            .bind(&assignment.recipe_id)
            .bind(&title)
            .bind(&image_url)
            .bind(prep_time_min)
            .bind(cook_time_min)
            .bind(&assignment.assignment_reasoning)
            .bind(&event.data.generated_at)
            .execute(&pool)
            .await?;
        }
    }

    Ok(())
}

/// Handle MealPlanRegenerated event - delete old calendar entries and re-project
#[evento::handler(MealPlanAggregate)]
pub async fn reproject_calendar_view_on_regenerate<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanRegenerated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Query user_id from meal_plans table
    let user_id: String = sqlx::query_scalar("SELECT user_id FROM meal_plans WHERE id = ?")
        .bind(&event.aggregator_id)
        .fetch_one(&pool)
        .await?;

    // Delete old calendar entries for this meal plan
    sqlx::query("DELETE FROM calendar_view WHERE meal_plan_id = ?")
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

#[allow(clippy::type_complexity)]
    // Re-project new assignments
    for assignment in &event.data.new_assignments {
        let recipe: Option<(String, Option<String>, Option<i32>, Option<i32>)> = sqlx::query_as(
            "SELECT title, image_url, prep_time_min, cook_time_min FROM recipe_list WHERE id = ?",
        )
        .bind(&assignment.recipe_id)
        .fetch_optional(&pool)
        .await?;

        if let Some((title, image_url, prep_time_min, cook_time_min)) = recipe {
            sqlx::query(
                r#"
                INSERT INTO calendar_view (
                    id, user_id, meal_plan_id, date, course_type, recipe_id,
                    recipe_title, recipe_image_url, prep_time_min, cook_time_min,
                    assignment_reasoning, created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
            )
            .bind(format!("{}-{}", assignment.date, assignment.course_type))
            .bind(&user_id)
            .bind(&event.aggregator_id)
            .bind(&assignment.date)
            .bind(&assignment.course_type)
            .bind(&assignment.recipe_id)
            .bind(&title)
            .bind(&image_url)
            .bind(prep_time_min)
            .bind(cook_time_min)
            .bind(&assignment.assignment_reasoning)
            .bind(&event.data.regenerated_at)
            .execute(&pool)
            .await?;
        }
    }

    Ok(())
}

/// Handle MealPlanRegenerated event - re-project dashboard_meals
#[evento::handler(MealPlanAggregate)]
pub async fn reproject_dashboard_meals_on_regenerate<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanRegenerated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Query user_id from meal_plans table
    let user_id: String = sqlx::query_scalar("SELECT user_id FROM meal_plans WHERE id = ?")
        .bind(&event.aggregator_id)
        .fetch_one(&pool)
        .await?;

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Delete old dashboard meals for this user
    sqlx::query("DELETE FROM dashboard_meals WHERE user_id = ?")
        .bind(&user_id)
        .execute(&pool)
        .await?;

#[allow(clippy::type_complexity)]
    // Re-project today's meals
    for assignment in &event.data.new_assignments {
        if assignment.date == today {
            let recipe: Option<(String, Option<String>, Option<i32>, Option<i32>)> = sqlx::query_as(
                "SELECT title, image_url, prep_time_min, cook_time_min FROM recipe_list WHERE id = ?",
            )
            .bind(&assignment.recipe_id)
            .fetch_optional(&pool)
            .await?;

            if let Some((title, image_url, prep_time_min, cook_time_min)) = recipe {
                sqlx::query(
                    r#"
                    INSERT INTO dashboard_meals (
                        id, user_id, date, course_type, recipe_id,
                        recipe_title, recipe_image_url, prep_time_min, cook_time_min,
                        prep_required, created_at, updated_at
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)
                    "#,
                )
                .bind(format!("{}-{}", assignment.date, assignment.course_type))
                .bind(&user_id)
                .bind(&assignment.date)
                .bind(&assignment.course_type)
                .bind(&assignment.recipe_id)
                .bind(&title)
                .bind(&image_url)
                .bind(prep_time_min)
                .bind(cook_time_min)
                .bind(if assignment.prep_required { 1 } else { 0 })
                .bind(&event.data.regenerated_at)
                .execute(&pool)
                .await?;
            }
        }
    }

    Ok(())
}
