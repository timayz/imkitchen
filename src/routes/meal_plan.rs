use crate::routes::meal_planning_api::load_user_preferences;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    Extension,
};
use chrono::{Datelike, NaiveDate, Utc};
use meal_planning::{
    algorithm::{RecipeForPlanning, UserConstraints},
    read_model::{MealAssignmentReadModel, MealPlanQueries},
};
use recipe::read_model::{query_recipes_by_user, RecipeReadModel};
use serde::{Deserialize, Serialize};
use shopping;
use sqlx::{Row, SqlitePool};

use crate::error::AppError;
use crate::middleware::auth::Auth;
use crate::routes::AppState;

/// RAII guard for generation lock
/// Automatically releases the lock when dropped (on function return or panic)
struct GenerationLockGuard {
    user_id: String,
    locks: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, ()>>>,
}

impl Drop for GenerationLockGuard {
    fn drop(&mut self) {
        let user_id = self.user_id.clone();
        let locks = self.locks.clone();

        // Spawn a task to release the lock since Drop is sync but Mutex is async
        tokio::spawn(async move {
            let mut map = locks.lock().await;
            map.remove(&user_id);
            tracing::debug!("Released generation lock for user: {}", user_id);
        });
    }
}

/// Accompaniment data for template rendering (Story 9.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccompanimentView {
    pub id: String,
    pub title: String,
}

/// Recipe with meal assignment for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealSlotData {
    pub assignment_id: String, // Story 3.4: Needed for "Replace This Meal" button
    pub date: String,
    pub course_type: String, // AC-5: Renamed from meal_type
    pub recipe_id: String,
    pub recipe_title: String,
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub prep_required: bool,
    pub complexity: Option<String>,
    pub assignment_reasoning: Option<String>, // Story 3.8: Assignment reasoning tooltip
    pub accompaniment: Option<AccompanimentView>, // Story 9.2: Accompaniment display
}

/// Template for meal plan loading state with TwinSpark polling
#[derive(Template, Debug)]
#[template(path = "pages/meal-plan-loading.html")]
pub struct MealPlanLoadingTemplate {
    pub user: Option<()>,
    pub meal_plan_id: String,
    pub current_path: String,
}

/// Template for meal plan polling continuation (partial HTML)
#[derive(Template, Debug)]
#[template(path = "components/meal-plan-polling.html")]
pub struct MealPlanPollingTemplate {
    pub meal_plan_id: String,
}

/// Template for meal plan polling page (full HTML with base.html)
#[derive(Template, Debug)]
#[template(path = "pages/meal-plan-polling-page.html")]
pub struct MealPlanPollingPageTemplate {
    pub user: Option<()>,
    pub meal_plan_id: String,
    pub current_path: String,
}

/// Template for meal plan error state (partial HTML)
#[derive(Template, Debug)]
#[template(path = "components/meal-plan-error.html")]
pub struct MealPlanErrorTemplate;

/// Helper function to load user profile constraints from database
///
/// Loads dietary restrictions, skill level, and weeknight availability from the users table
/// and constructs a UserConstraints struct for the meal planning algorithm.
async fn _load_user_constraints(
    user_id: &str,
    db_pool: &SqlitePool,
) -> Result<UserConstraints, AppError> {
    let user_profile =
        sqlx::query("SELECT dietary_restrictions, weeknight_availability FROM users WHERE id = ?1")
            .bind(user_id)
            .fetch_optional(db_pool)
            .await?;

    match user_profile {
        Some(row) => {
            // Parse dietary restrictions from JSON
            let dietary_str: Option<String> = row.get("dietary_restrictions");
            let dietary_restrictions: Vec<String> = dietary_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            // Parse weeknight availability JSON to extract duration_minutes
            let availability_json: Option<String> = row.get("weeknight_availability");
            let weeknight_availability_minutes = availability_json
                .as_ref()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok())
                .and_then(|v| v["duration_minutes"].as_u64())
                .map(|mins| mins as u32);

            Ok(UserConstraints {
                weeknight_availability_minutes,
                dietary_restrictions,
            })
        }
        None => {
            tracing::warn!("User profile not found for {}, using defaults", user_id);
            Ok(UserConstraints::default())
        }
    }
}

/// Day with 3 meal slots for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayData {
    pub date: String,
    pub day_name: String,                  // "Monday", "Tuesday", etc.
    pub is_today: bool,                    // AC-6: Today's date highlighted
    pub is_past: bool,                     // AC-7: Past dates dimmed
    pub meal_plan_id: String,              // Story 3.5: Needed for calendar context links
    pub appetizer: Option<MealSlotData>,   // AC-5: Course-based model (renamed from breakfast)
    pub main_course: Option<MealSlotData>, // AC-5: Course-based model (renamed from lunch)
    pub dessert: Option<MealSlotData>,     // AC-5: Course-based model (renamed from dinner)
}

#[derive(Template)]
#[template(path = "pages/meal-calendar.html")]
pub struct MealCalendarTemplate {
    pub user: Option<()>,
    pub days: Vec<DayData>,
    pub start_date: String,
    pub end_date: String, // Story 3.13: End date for "Week of {start} - {end}" display
    pub has_meal_plan: bool,
    pub rotation_used: usize,  // AC (Story 3.3): Rotation progress display
    pub rotation_total: usize, // AC (Story 3.3): Total favorites
    pub current_path: String,
    pub error_message: Option<String>, // Issue #130: Display inline error messages
    pub current_week_index: usize,     // Multi-week navigation: current week (0-indexed)
    pub total_weeks: usize,            // Multi-week navigation: total number of weeks
    pub is_current_week: bool,         // Whether this is the current week
}

/// Multi-Week Calendar Template (Story 9.1, Epic 8)
///
/// Renders the multi-week meal plan calendar with week tabs (desktop)
/// and carousel navigation (mobile). Supports TwinSpark partial updates
/// for week navigation without full page reload.
#[derive(Template)]
#[template(path = "meal_plan/multi_week_calendar.html")]
pub struct MultiWeekCalendarTemplate {
    pub user: Option<()>,
    pub weeks: Vec<meal_planning::read_model::WeekReadModel>,
    pub current_week_id: String,
    pub current_week_index: usize, // Index of current week in weeks vec (for carousel navigation)
    pub current_week_start_date: String, // For shopping list link
    pub has_meal_plan: bool,
    pub days: Vec<DayData>, // 7-day grid for the current week
    pub error_message: Option<String>,
    pub current_path: String,
    pub future_weeks_count: usize, // Count of unlocked (future) weeks for regeneration
}

/// Week Calendar Content Partial Template (Story 9.1, Epic 8)
///
/// Renders just the 7-day meal grid for a single week.
/// Used for TwinSpark partial HTML updates when user navigates between weeks.
#[derive(Template)]
#[template(path = "meal_plan/week_calendar_content.html")]
pub struct WeekCalendarContentTemplate {
    pub days: Vec<DayData>, // 7-day grid for the selected week
}

/// GET /plan/check-ready/:meal_plan_id - Check if meal plan read model is ready
///
/// This endpoint is used by TwinSpark polling to check if evento projections
/// have completed for a newly generated or regenerated meal plan.
///
/// Verifies that:
/// 1. Meal plan has 21 assignments in read model
/// 2. Read model updated_at matches latest event timestamp from aggregate
///
/// Returns polling HTML until ready, then returns full meal calendar page HTML
/// for TwinSpark to swap into <body>.
pub async fn get_meal_plan_check_ready(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
    axum::extract::Path(meal_plan_id): axum::extract::Path<String>,
) -> axum::response::Response {
    tracing::debug!("Checking meal plan readiness for id: {}", meal_plan_id);

    // Check if ALL weeks in the batch have complete assignments in read model
    // Each week needs 7 assignments (one per day)
    //
    // Note: This works for both initial generation and regeneration because:
    // - Initial generation: creates new meal_plan records with new IDs
    // - Regeneration: archives old meal_plans and creates new ones with new IDs
    // In both cases, we wait for the new meal_plan records and their assignments to appear
    let batch_check: Result<(i64, i64), sqlx::Error> = sqlx::query_as(
        r#"
        SELECT
            COUNT(DISTINCT mp.id) as total_weeks,
            COUNT(ma.id) as total_assignments
        FROM meal_plans mp
        LEFT JOIN meal_assignments ma ON mp.id = ma.meal_plan_id
        WHERE mp.generation_batch_id = (
            SELECT generation_batch_id FROM meal_plans WHERE id = ?1
        )
        AND mp.user_id = ?2
        "#,
    )
    .bind(&meal_plan_id)
    .bind(&auth.user_id)
    .fetch_one(&state.db_pool)
    .await;

    let is_ready = match &batch_check {
        Ok((total_weeks, total_assignments)) => {
            tracing::debug!(
                "Batch check: total_weeks={}, total_assignments={}, expected={}",
                total_weeks,
                total_assignments,
                total_weeks * 7
            );
            // Each week needs 7 assignments, so total should be total_weeks * 7
            *total_weeks > 0 && *total_assignments == *total_weeks * 7
        }
        Err(e) => {
            tracing::debug!("Batch check query failed (projection likely not complete yet): {}", e);
            false
        }
    };

    if is_ready {
        // All weeks are ready - return ts-location header to redirect to /plan
        tracing::info!("Meal plan batch ready, redirecting to /plan");
        (
            axum::http::StatusCode::OK,
            [("ts-location", "/plan")],
            Html(String::new()),
        )
            .into_response()
    } else {
        // Not all weeks ready yet - return self polling element to continue polling
        let polling_html = format!(
            r##"<div ts-req="/plan/check-ready/{}" ts-trigger="load delay:500ms"></div>"##,
            meal_plan_id
        );
        (axum::http::StatusCode::OK, Html(polling_html)).into_response()
    }
}

/// GET /plan - Display meal calendar view
///
/// Supports multi-week navigation via ?week=N query parameter (0-indexed)
///
/// AC-5: Week-view calendar displays generated plan with breakfast/lunch/dinner slots filled
/// AC-9: User redirected to calendar view after successful generation
pub async fn get_meal_plan(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Html<String>, AppError> {
    use chrono::{Local, NaiveDate};
    let today = Local::now().date_naive();

    // Query active meal plan for user to get generation_batch_id
    let first_meal_plan =
        MealPlanQueries::get_active_meal_plan(&auth.user_id, &state.db_pool).await?;

    if first_meal_plan.is_none() {
        // No meal plan exists, show empty calendar or prompt
        let template = MealCalendarTemplate {
            user: Some(()),
            days: Vec::new(),
            start_date: String::new(),
            end_date: String::new(),
            has_meal_plan: false,
            rotation_used: 0,
            rotation_total: 0,
            current_path: "/plan".to_string(),
            error_message: None,
            current_week_index: 0,
            total_weeks: 0,
            is_current_week: false,
        };
        return template.render().map(Html).map_err(|e| {
            tracing::error!("Failed to render empty multi-week calendar template: {:?}", e);
            AppError::InternalError("Failed to render page".to_string())
        });
    }

    let first_plan = first_meal_plan.unwrap();

    // Query ALL meal plans in the same generation batch (for multi-week support)
    let all_meal_plans: Vec<_> = sqlx::query_as::<_, meal_planning::read_model::MealPlanReadModel>(
        r#"
        SELECT id, user_id, start_date, status, rotation_state, created_at, updated_at
        FROM meal_plans
        WHERE user_id = ?1 AND generation_batch_id = (
            SELECT generation_batch_id FROM meal_plans WHERE id = ?2
        )
        ORDER BY start_date
        "#,
    )
    .bind(&auth.user_id)
    .bind(&first_plan.id)
    .fetch_all(&state.db_pool)
    .await?;

    let _total_weeks = all_meal_plans.len();

    tracing::info!(
        "Loaded {} meal plans for batch_id from plan {}",
        all_meal_plans.len(),
        first_plan.id
    );

    // Find the current week index (week containing today's date)
    let current_week_idx = all_meal_plans
        .iter()
        .position(|plan| {
            if let Ok(start_date) = NaiveDate::parse_from_str(&plan.start_date, "%Y-%m-%d") {
                let end_date = start_date + chrono::Duration::days(6);
                today >= start_date && today <= end_date
            } else {
                false
            }
        });

    // Get week index from query parameter, or default to current week or first week
    let week_index: usize = params
        .get("week")
        .and_then(|w| w.parse().ok())
        .or(current_week_idx)
        .unwrap_or(0); // Default to first week if current week not found

    // Get the selected week's meal plan
    let selected_plan = all_meal_plans.get(week_index).unwrap_or(&first_plan);

    // Query all assignments for ALL weeks (to enable correct week calculation)
    let _all_assignments: Vec<_> = {
        let mut assignments = Vec::new();
        for plan in &all_meal_plans {
            let plan_assignments = MealPlanQueries::get_meal_assignments(&plan.id, &state.db_pool).await?;
            assignments.extend(plan_assignments);
        }
        assignments
    };

    // Filter assignments to only the selected week
    let assignments: Vec<_> = MealPlanQueries::get_meal_assignments(&selected_plan.id, &state.db_pool).await?;

    let meal_plan_with_assignments = if !assignments.is_empty() {
        Some(assignments)
    } else {
        None
    };

    match meal_plan_with_assignments {
        Some(assignments) => {
            // Calculate the start and end dates for the selected week
            use chrono::NaiveDate;
            let _week_start = NaiveDate::parse_from_str(&selected_plan.start_date, "%Y-%m-%d")
                .map_err(|_| AppError::InternalError("Invalid start date".to_string()))?;

            // Fetch recipe details for assignments in this week
            let recipe_ids: Vec<String> = assignments.iter().map(|a| a.recipe_id.clone()).collect();

            let recipes = fetch_recipes_by_ids(&recipe_ids, &state.db_pool).await?;

            // Story 9.2: Fetch accompaniment recipes
            let accompaniment_ids: Vec<String> = assignments
                .iter()
                .filter_map(|a| a.accompaniment_recipe_id.clone())
                .collect();

            let accompaniment_recipes = if !accompaniment_ids.is_empty() {
                fetch_recipes_by_ids(&accompaniment_ids, &state.db_pool).await?
            } else {
                Vec::new()
            };

            // AC (Story 3.3): Query rotation progress for display (not used in multi-week template)
            let (_rotation_used, _rotation_total) =
                MealPlanQueries::query_rotation_progress(&auth.user_id, &state.db_pool).await?;

            // Group assignments by date into DayData with today/past flags
            let days = build_day_data(
                &assignments,
                &recipes,
                &accompaniment_recipes,
                &selected_plan.id,
            );

            let start_date = selected_plan.start_date.clone();
            let end_date = if let Ok(start) = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d") {
                (start + chrono::Duration::days(6)).format("%Y-%m-%d").to_string()
            } else {
                String::new()
            };

            let is_current_week = current_week_idx.map_or(false, |idx| week_index == idx);

            let template = MealCalendarTemplate {
                user: Some(()),
                days,
                start_date,
                end_date,
                has_meal_plan: true,
                rotation_used: _rotation_used,
                rotation_total: _rotation_total,
                current_path: "/plan".to_string(),
                error_message: None,
                current_week_index: week_index,
                total_weeks: all_meal_plans.len(),
                is_current_week,
            };
            template.render().map(Html).map_err(|e| {
                tracing::error!("Failed to render meal calendar template: {:?}", e);
                AppError::InternalError("Failed to render page".to_string())
            })
        }
        None => {
            // No assignments for this meal plan, show empty template
            let start_date = selected_plan.start_date.clone();
            let end_date = if let Ok(start) = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d") {
                (start + chrono::Duration::days(6)).format("%Y-%m-%d").to_string()
            } else {
                String::new()
            };

            let is_current_week = current_week_idx.map_or(false, |idx| week_index == idx);

            let template = MealCalendarTemplate {
                user: Some(()),
                days: Vec::new(),
                start_date,
                end_date,
                has_meal_plan: true,
                rotation_used: 0,
                rotation_total: 0,
                current_path: "/plan".to_string(),
                error_message: None,
                current_week_index: week_index,
                total_weeks: all_meal_plans.len(),
                is_current_week,
            };
            template.render().map(Html).map_err(|e| {
                tracing::error!("Failed to render meal calendar template: {:?}", e);
                AppError::InternalError("Failed to render page".to_string())
            })
        }
    }
}

/// POST /plan/generate - Generate new meal plan
///
/// AC-1: Home dashboard displays "Generate Meal Plan" button prominently
/// AC-2: Clicking button triggers meal planning algorithm
/// AC-3: System analyzes all favorited recipes against user profile constraints
/// AC-4: Algorithm generates single meal plan with recipes organized by week
/// AC-6: Generation completes within 5 seconds for up to 50 favorite recipes
/// AC-7: Progress indicator shown during generation (future enhancement)
/// AC-8: Generated plan automatically becomes active
/// AC-9: User redirected to calendar view after successful generation
/// AC-10: If insufficient recipes (<7 favorites), display helpful error
pub async fn post_generate_meal_plan(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    // **Critical Fix 1.4:** Acquire generation lock to prevent concurrent generation
    // This prevents race conditions when multiple requests try to generate plans simultaneously
    let _lock_guard = {
        let mut locks = state.generation_locks.lock().await;

        // Check if lock already exists for this user
        if locks.contains_key(&auth.user_id) {
            tracing::warn!(
                "Concurrent generation attempt detected for user: {}",
                auth.user_id
            );
            return Err(AppError::ConcurrentGenerationInProgress);
        }

        // Acquire lock by inserting user_id
        locks.insert(auth.user_id.clone(), ());
        tracing::debug!("Acquired generation lock for user: {}", auth.user_id);

        // Create guard that will auto-release on drop
        GenerationLockGuard {
            user_id: auth.user_id.clone(),
            locks: state.generation_locks.clone(),
        }
    };
    // Lock is now held via _lock_guard RAII - will auto-release on function exit

    // Query user's favorited recipes
    let favorites = query_recipes_by_user(&auth.user_id, true, &state.db_pool).await?;

    // AC-10: Validate minimum 7 favorite recipes
    // Issue #130: Return inline error message instead of separate error page
    if favorites.len() < 7 {
        let error_msg = format!(
            "You need at least 7 favorite recipes to generate a meal plan. You currently have {}. Add {} more recipe{} to get started!",
            favorites.len(),
            7 - favorites.len(),
            if 7 - favorites.len() > 1 { "s" } else { "" }
        );

        let template = MealCalendarTemplate {
            user: Some(()),
            days: Vec::new(),
            start_date: String::new(),
            end_date: String::new(),
            has_meal_plan: false,
            rotation_used: 0,
            rotation_total: favorites.len(),
            current_path: "/plan".to_string(),
            error_message: Some(error_msg),
            current_week_index: 0,
            total_weeks: 0,
            is_current_week: false,
        };

        return template.render().map(Html).map_err(|e| {
            tracing::error!(
                "Failed to render meal calendar template with error: {:?}",
                e
            );
            AppError::InternalError("Failed to render page".to_string())
        });
    }

    // Convert RecipeReadModel to RecipeForPlanning
    let recipes_for_planning: Vec<RecipeForPlanning> = favorites
        .into_iter()
        .map(|r| {
            let ingredients: Vec<serde_json::Value> = serde_json::from_str(&r.ingredients)
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        "Failed to parse ingredients JSON for recipe {}: {}",
                        r.id,
                        e
                    );
                    Vec::new()
                });
            let instructions: Vec<serde_json::Value> = serde_json::from_str(&r.instructions)
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        "Failed to parse instructions JSON for recipe {}: {}",
                        r.id,
                        e
                    );
                    Vec::new()
                });

            // Parse dietary_tags from JSON
            let dietary_tags: Vec<String> = r
                .dietary_tags
                .as_ref()
                .and_then(|tags_json| serde_json::from_str(tags_json).ok())
                .unwrap_or_default();

            // Parse cuisine from string to enum (Story 7.2)
            let cuisine = r
                .cuisine
                .as_ref()
                .and_then(|c| serde_json::from_str(&format!("\"{}\"", c)).ok())
                .unwrap_or(recipe::Cuisine::Italian); // Default fallback

            RecipeForPlanning {
                id: r.id,
                title: r.title,
                recipe_type: r.recipe_type, // AC-4: Add recipe_type for course matching
                ingredients_count: ingredients.len(),
                instructions_count: instructions.len(),
                prep_time_min: r.prep_time_min.map(|v| v as u32),
                cook_time_min: r.cook_time_min.map(|v| v as u32),
                advance_prep_hours: r.advance_prep_hours.map(|v| v as u32),
                complexity: r.complexity,
                dietary_tags,
                cuisine,
                accepts_accompaniment: false, // Story 7.3: Default for now, will be set from DB later
                preferred_accompaniments: vec![],
                accompaniment_category: None,
            }
        })
        .collect();

    // Load user profile preferences from database (for multi-week generation)
    let preferences = load_user_preferences(&auth.user_id, &state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to load user preferences: {:?}", e);
            AppError::InternalError("Failed to load user preferences".to_string())
        })?;

    tracing::info!(
        "Generating multi-week meal plan for user {} with {} favorite recipes",
        auth.user_id,
        recipes_for_planning.len()
    );

    // AC-2, AC-3, AC-4, AC-6: Generate multi-week meal plan (up to 5 weeks)
    // Issue #130: Handle algorithm errors inline
    let multi_week_plan = match meal_planning::generate_multi_week_meal_plans(
        auth.user_id.clone(),
        recipes_for_planning.clone(),
        preferences,
    )
    .await
    {
        Ok(plan) => plan,
        Err(meal_planning::MealPlanningError::InsufficientRecipes { minimum, current }) => {
            let error_msg = format!(
                "Cannot generate meal plan: Need at least {} main course recipe(s), but found only {} total recipes. Please add at least one recipe with recipe_type='main_course'.",
                minimum,
                current
            );

            let template = MealCalendarTemplate {
                user: Some(()),
                days: Vec::new(),
                start_date: String::new(),
                end_date: String::new(),
                has_meal_plan: false,
                rotation_used: 0,
                rotation_total: current,
                current_path: "/plan".to_string(),
                error_message: Some(error_msg),
                current_week_index: 0,
                total_weeks: 0,
                is_current_week: false,
            };

            return template.render().map(Html).map_err(|e| {
                tracing::error!(
                    "Failed to render meal calendar template with error: {:?}",
                    e
                );
                AppError::InternalError("Failed to render page".to_string())
            });
        }
        Err(e) => {
            tracing::error!("Multi-week meal planning algorithm error: {:?}", e);
            return Err(AppError::MealPlanningError(e));
        }
    };

    tracing::info!(
        "Multi-week generation successful: {} weeks generated",
        multi_week_plan.generated_weeks.len()
    );

    // AC-8: Emit MultiWeekMealPlanGenerated event via evento (this will trigger read model projection)
    let generation_batch_id = multi_week_plan.generation_batch_id.clone();
    let weeks_data: Vec<meal_planning::WeekMealPlanData> = multi_week_plan
        .generated_weeks
        .iter()
        .map(|week| meal_planning::WeekMealPlanData {
            id: week.id.clone(),
            start_date: week.start_date.clone(),
            end_date: week.end_date.clone(),
            status: week.status,
            is_locked: week.is_locked,
            meal_assignments: week.meal_assignments.clone(),
            shopping_list_id: week.shopping_list_id.clone(),
        })
        .collect();

    let event = meal_planning::MultiWeekMealPlanGenerated {
        generation_batch_id: generation_batch_id.clone(),
        user_id: auth.user_id.clone(),
        weeks: weeks_data,
        rotation_state: multi_week_plan.rotation_state.clone(),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<meal_planning::MealPlanAggregate>()
        .data(&event)
        .map_err(|e| {
            tracing::error!("Failed to encode MultiWeekMealPlanGenerated event: {:?}", e);
            anyhow::anyhow!("Failed to encode event: {}", e)
        })?
        .metadata(&true)
        .map_err(|e| {
            tracing::error!("Failed to encode metadata: {:?}", e);
            anyhow::anyhow!("Failed to encode metadata: {}", e)
        })?
        .commit(&state.evento_executor)
        .await
        .map_err(|e| {
            tracing::error!("Failed to commit MultiWeekMealPlanGenerated event: {:?}", e);
            anyhow::anyhow!("Failed to commit event: {}", e)
        })?;

    tracing::info!(
        "MultiWeekMealPlanGenerated event emitted successfully for batch {}",
        generation_batch_id
    );

    // BUSINESS RULE: Auto-generate shopping lists for each week
    tracing::info!(
        "Auto-generating shopping lists for {} weeks",
        multi_week_plan.generated_weeks.len()
    );

    for (week_index, week) in multi_week_plan.generated_weeks.iter().enumerate() {
        tracing::debug!(
            "Generating shopping list for week {} ({}): meal_plan_id={}",
            week_index + 1,
            week.start_date,
            week.id
        );

        // Collect all recipe IDs from this week's meal assignments
        let recipe_ids: Vec<String> = week
            .meal_assignments
            .iter()
            .map(|assignment| assignment.recipe_id.clone())
            .collect();

        // Collect ingredients from all recipes
        let ingredients = collect_ingredients_from_recipes(&recipe_ids, &state.db_pool).await?;

        tracing::info!(
            "Generating shopping list for week {} with {} ingredients from {} recipes",
            week.start_date,
            ingredients.len(),
            recipe_ids.len()
        );

        // Generate shopping list
        let shopping_list_cmd = shopping::GenerateShoppingListCommand {
            user_id: auth.user_id.clone(),
            meal_plan_id: week.id.clone(),
            week_start_date: week.start_date.clone(),
            ingredients,
        };

        match shopping::generate_shopping_list(shopping_list_cmd, &state.evento_executor).await {
            Ok(shopping_list_id) => {
                tracing::info!(
                    "Shopping list generated successfully: id={}, week={}, meal_plan_id={}",
                    shopping_list_id,
                    week.start_date,
                    week.id
                );
            }
            Err(e) => {
                tracing::error!(
                    "Failed to generate shopping list for week {}: {:?}",
                    week.start_date,
                    e
                );
                // Continue processing other weeks even if one fails
            }
        }
    }

    // Get the first week ID for polling redirect
    let first_week_id = multi_week_plan
        .generated_weeks
        .first()
        .map(|w| w.id.clone())
        .ok_or_else(|| anyhow::anyhow!("No weeks generated in multi-week plan"))?;

    // Return loading state that polls for read model completion
    // TwinSpark will poll /plan/check-ready until all weeks are fully projected
    let loading_template = MealPlanLoadingTemplate {
        user: Some(()),
        meal_plan_id: first_week_id.clone(),
        current_path: "/plan".to_string(),
    };

    loading_template.render().map(Html).map_err(|e| {
        tracing::error!("Failed to render meal plan loading template: {:?}", e);
        AppError::InternalError("Failed to render page".to_string())
    })
}

/// Helper: Collect all ingredients from recipes for shopping list generation
///
/// Parses ingredients JSON from each recipe and returns a flat vector of (name, quantity, unit) tuples
async fn collect_ingredients_from_recipes(
    recipe_ids: &[String],
    pool: &SqlitePool,
) -> Result<Vec<(String, f32, String)>, AppError> {
    let recipes = fetch_recipes_by_ids(recipe_ids, pool).await?;

    let mut all_ingredients = Vec::new();

    for recipe in recipes {
        // Parse ingredients JSON
        let ingredients: Vec<serde_json::Value> = serde_json::from_str(&recipe.ingredients)
            .map_err(|e| {
                tracing::warn!(
                    "Failed to parse ingredients for recipe {}: {}",
                    recipe.id,
                    e
                );
                AppError::InternalError(format!("Invalid ingredients JSON: {}", e))
            })?;

        for ingredient in ingredients {
            let name = ingredient["name"].as_str().unwrap_or("").to_string();
            let quantity = ingredient["quantity"].as_f64().unwrap_or(0.0) as f32;
            let unit = ingredient["unit"].as_str().unwrap_or("").to_string();

            if !name.is_empty() && quantity > 0.0 {
                all_ingredients.push((name, quantity, unit));
            }
        }
    }

    Ok(all_ingredients)
}

/// Helper: Fetch recipes by IDs
async fn fetch_recipes_by_ids(
    recipe_ids: &[String],
    pool: &SqlitePool,
) -> Result<Vec<RecipeReadModel>, sqlx::Error> {
    if recipe_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Build placeholders for IN clause
    let placeholders = recipe_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query_str = format!(
        "SELECT id, user_id, title, recipe_type, ingredients, instructions, prep_time_min, cook_time_min, advance_prep_hours, serving_size, is_favorite, is_shared, complexity, cuisine, dietary_tags, accepts_accompaniment, preferred_accompaniments, accompaniment_category, created_at, updated_at FROM recipes WHERE id IN ({}) AND deleted_at IS NULL",
        placeholders
    );

    let mut query = sqlx::query_as::<_, RecipeReadModel>(&query_str);
    for id in recipe_ids {
        query = query.bind(id);
    }

    query.fetch_all(pool).await
}

/// Helper: Group meal assignments by date into DayData
fn build_day_data(
    assignments: &[MealAssignmentReadModel],
    recipes: &[RecipeReadModel],
    accompaniment_recipes: &[RecipeReadModel], // Story 9.2: Accompaniment recipe lookups
    meal_plan_id: &str,                        // Story 3.5: Pass meal_plan_id for calendar context
) -> Vec<DayData> {
    use std::collections::HashMap;

    // AC-6, AC-7: Get today's date for highlighting logic
    let today = chrono::Local::now().date_naive();

    // Create recipe lookup maps
    let recipe_map: HashMap<String, &RecipeReadModel> =
        recipes.iter().map(|r| (r.id.clone(), r)).collect();

    // Story 9.2: Create accompaniment recipe lookup map
    let accompaniment_map: HashMap<String, &RecipeReadModel> = accompaniment_recipes
        .iter()
        .map(|r| (r.id.clone(), r))
        .collect();

    // Group assignments by date
    let mut days_map: HashMap<String, DayData> = HashMap::new();

    for assignment in assignments {
        let date = assignment.date.clone();

        // Parse date to get day name and compute is_today/is_past flags
        let (day_name, is_today, is_past) =
            if let Ok(parsed_date) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
                let day_name = parsed_date.weekday().to_string();
                let is_today = parsed_date == today;
                let is_past = parsed_date < today;
                (day_name, is_today, is_past)
            } else {
                (String::new(), false, false)
            };

        // Get or create DayData
        let day_data = days_map.entry(date.clone()).or_insert(DayData {
            date: date.clone(),
            day_name,
            is_today,
            is_past,
            meal_plan_id: meal_plan_id.to_string(), // Story 3.5: Include for calendar context
            appetizer: None,                        // AC-5: Course-based model
            main_course: None,                      // AC-5: Course-based model
            dessert: None,                          // AC-5: Course-based model
        });

        // Get recipe details
        let recipe = recipe_map.get(&assignment.recipe_id);

        if let Some(recipe) = recipe {
            // Story 9.2: Load accompaniment if present
            let accompaniment = assignment
                .accompaniment_recipe_id
                .as_ref()
                .and_then(|acc_id| accompaniment_map.get(acc_id))
                .map(|acc_recipe| AccompanimentView {
                    id: acc_recipe.id.clone(),
                    title: acc_recipe.title.clone(),
                });

            let slot_data = MealSlotData {
                assignment_id: assignment.id.clone(), // Story 3.4: Include for replacement
                date: assignment.date.clone(),
                course_type: assignment.course_type.clone(), // AC-5: Use course_type
                recipe_id: recipe.id.clone(),
                recipe_title: recipe.title.clone(),
                prep_time_min: recipe.prep_time_min,
                cook_time_min: recipe.cook_time_min,
                prep_required: assignment.prep_required,
                complexity: recipe.complexity.clone(),
                assignment_reasoning: assignment.assignment_reasoning.clone(), // Story 3.8: Reasoning tooltip
                accompaniment, // Story 9.2: Accompaniment display
            };

            // Assign to appropriate course slot (AC-5)
            match assignment.course_type.as_str() {
                "appetizer" => day_data.appetizer = Some(slot_data),
                "main_course" => day_data.main_course = Some(slot_data),
                "dessert" => day_data.dessert = Some(slot_data),
                // Backward compatibility for old data
                "breakfast" => day_data.appetizer = Some(slot_data),
                "lunch" => day_data.main_course = Some(slot_data),
                "dinner" => day_data.dessert = Some(slot_data),
                _ => {}
            }
        }
    }

    // Sort days by date and return as Vec
    let mut days: Vec<DayData> = days_map.into_values().collect();
    days.sort_by(|a, b| a.date.cmp(&b.date));
    days
}

/// Form data for regenerating meal plan (Story 3.7)
#[derive(Debug, Deserialize)]
pub struct RegenerateMealPlanForm {
    pub regeneration_reason: Option<String>,
}

/// Template for regeneration confirmation modal (Story 3.7)
#[derive(Template, Debug)]
#[template(path = "components/regenerate-confirmation-modal.html")]
pub struct RegenerateConfirmationModalTemplate {
    pub meal_plan_id: String,
}

/// GET /plan/regenerate/confirm - Get confirmation modal for regeneration (Story 3.7)
///
/// AC-2: Confirmation dialog: "This will replace your entire meal plan. Continue?"
///
/// Returns a modal with confirmation message and optional reason field.
pub async fn get_regenerate_confirm(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    // Query active meal plan for user
    let meal_plan = MealPlanQueries::get_active_meal_plan(&auth.user_id, &state.db_pool)
        .await?
        .ok_or_else(|| AppError::InternalError("No active meal plan found".to_string()))?;

    // Render confirmation modal
    let template = RegenerateConfirmationModalTemplate {
        meal_plan_id: meal_plan.id,
    };

    let modal_html = template
        .render()
        .map_err(|e| AppError::InternalError(format!("Template render error: {}", e)))?;

    Ok(Html(modal_html))
}

/// POST /plan/regenerate - Regenerate entire meal plan (Story 3.7)
///
/// AC-3: Clicking confirm triggers full meal plan regeneration
/// AC-4: Algorithm runs with same logic as initial generation
/// AC-5: Rotation state preserved (doesn't reset cycle)
/// AC-6: New plan fills all slots with different recipe assignments
/// AC-7: Calendar updates to show new plan
/// AC-8: Shopping list regenerated for new plan (cross-domain event)
/// AC-9: Old meal plan archived for audit trail (event sourcing)
/// AC-10: Generation respects same optimization factors
pub async fn post_regenerate_meal_plan(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
    axum::Form(_form): axum::Form<RegenerateMealPlanForm>,
) -> Result<Html<String>, AppError> {
    // Acquire generation lock to prevent concurrent regeneration
    let _lock_guard = {
        let mut locks = state.generation_locks.lock().await;

        if locks.contains_key(&auth.user_id) {
            tracing::warn!(
                "Concurrent regeneration attempt detected for user: {}",
                auth.user_id
            );
            return Err(AppError::ConcurrentGenerationInProgress);
        }

        locks.insert(auth.user_id.clone(), ());
        tracing::debug!("Acquired regeneration lock for user: {}", auth.user_id);

        GenerationLockGuard {
            user_id: auth.user_id.clone(),
            locks: state.generation_locks.clone(),
        }
    };

    // Query active meal plan
    let _meal_plan = MealPlanQueries::get_active_meal_plan(&auth.user_id, &state.db_pool)
        .await?
        .ok_or_else(|| AppError::InternalError("No active meal plan to regenerate".to_string()))?;

    // Query user's favorited recipes
    let favorites = query_recipes_by_user(&auth.user_id, true, &state.db_pool).await?;

    // Validate minimum 7 favorite recipes
    if favorites.len() < 7 {
        return Err(AppError::InsufficientRecipes {
            current: favorites.len(),
            required: 7,
        });
    }

    // Convert RecipeReadModel to RecipeForPlanning
    let recipes_for_planning: Vec<RecipeForPlanning> = favorites
        .into_iter()
        .map(|r| {
            let ingredients: Vec<serde_json::Value> = serde_json::from_str(&r.ingredients)
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        "Failed to parse ingredients JSON for recipe {}: {}",
                        r.id,
                        e
                    );
                    Vec::new()
                });
            let instructions: Vec<serde_json::Value> = serde_json::from_str(&r.instructions)
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        "Failed to parse instructions JSON for recipe {}: {}",
                        r.id,
                        e
                    );
                    Vec::new()
                });

            // Parse dietary_tags from JSON
            let dietary_tags: Vec<String> = r
                .dietary_tags
                .as_ref()
                .and_then(|tags_json| serde_json::from_str(tags_json).ok())
                .unwrap_or_default();

            // Parse cuisine from string to enum (Story 7.2)
            let cuisine = r
                .cuisine
                .as_ref()
                .and_then(|c| serde_json::from_str(&format!("\"{}\"", c)).ok())
                .unwrap_or(recipe::Cuisine::Italian); // Default fallback

            RecipeForPlanning {
                id: r.id,
                title: r.title,
                recipe_type: r.recipe_type, // AC-4: Add recipe_type for course matching
                ingredients_count: ingredients.len(),
                instructions_count: instructions.len(),
                prep_time_min: r.prep_time_min.map(|v| v as u32),
                cook_time_min: r.cook_time_min.map(|v| v as u32),
                advance_prep_hours: r.advance_prep_hours.map(|v| v as u32),
                complexity: r.complexity,
                dietary_tags,
                cuisine,
                accepts_accompaniment: false, // Story 7.3: Default for now, will be set from DB later
                preferred_accompaniments: vec![],
                accompaniment_category: None,
            }
        })
        .collect();

    // Load user profile preferences from database (for multi-week generation)
    let preferences = load_user_preferences(&auth.user_id, &state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to load user preferences: {:?}", e);
            AppError::InternalError("Failed to load user preferences".to_string())
        })?;

    tracing::info!(
        "Regenerating multi-week meal plan for user {} with {} favorite recipes",
        auth.user_id,
        recipes_for_planning.len()
    );

    // Generate new multi-week meal plan (this will archive old plans via projection)
    let multi_week_plan = match meal_planning::generate_multi_week_meal_plans(
        auth.user_id.clone(),
        recipes_for_planning.clone(),
        preferences,
    )
    .await
    {
        Ok(plan) => plan,
        Err(meal_planning::MealPlanningError::InsufficientRecipes { minimum, current }) => {
            return Err(AppError::InsufficientRecipes {
                current,
                required: minimum,
            });
        }
        Err(e) => {
            tracing::error!("Multi-week meal planning algorithm error: {:?}", e);
            return Err(AppError::MealPlanningError(e));
        }
    };

    tracing::info!(
        "Multi-week regeneration successful: {} weeks generated",
        multi_week_plan.generated_weeks.len()
    );

    // Emit MultiWeekMealPlanGenerated event via evento
    let generation_batch_id = multi_week_plan.generation_batch_id.clone();
    let weeks_data: Vec<meal_planning::WeekMealPlanData> = multi_week_plan
        .generated_weeks
        .iter()
        .map(|week| meal_planning::WeekMealPlanData {
            id: week.id.clone(),
            start_date: week.start_date.clone(),
            end_date: week.end_date.clone(),
            status: week.status,
            is_locked: week.is_locked,
            meal_assignments: week.meal_assignments.clone(),
            shopping_list_id: week.shopping_list_id.clone(),
        })
        .collect();

    let event = meal_planning::MultiWeekMealPlanGenerated {
        generation_batch_id: generation_batch_id.clone(),
        user_id: auth.user_id.clone(),
        weeks: weeks_data,
        rotation_state: multi_week_plan.rotation_state.clone(),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<meal_planning::MealPlanAggregate>()
        .data(&event)
        .map_err(|e| {
            tracing::error!("Failed to encode MultiWeekMealPlanGenerated event: {:?}", e);
            anyhow::anyhow!("Failed to encode event: {}", e)
        })?
        .metadata(&true)
        .map_err(|e| {
            tracing::error!("Failed to encode metadata: {:?}", e);
            anyhow::anyhow!("Failed to encode metadata: {}", e)
        })?
        .commit(&state.evento_executor)
        .await
        .map_err(|e| {
            tracing::error!("Failed to commit MultiWeekMealPlanGenerated event: {:?}", e);
            anyhow::anyhow!("Failed to commit event: {}", e)
        })?;

    tracing::info!(
        "MultiWeekMealPlanGenerated event emitted successfully for batch {}",
        generation_batch_id
    );

    // BUSINESS RULE: Auto-generate shopping lists for each week
    tracing::info!(
        "Auto-generating shopping lists for {} weeks",
        multi_week_plan.generated_weeks.len()
    );

    for (week_index, week) in multi_week_plan.generated_weeks.iter().enumerate() {
        tracing::debug!(
            "Generating shopping list for week {} ({}): meal_plan_id={}",
            week_index + 1,
            week.start_date,
            week.id
        );

        // Collect all recipe IDs from this week's meal assignments
        let recipe_ids: Vec<String> = week
            .meal_assignments
            .iter()
            .map(|assignment| assignment.recipe_id.clone())
            .collect();

        // Collect ingredients from all recipes
        let ingredients = collect_ingredients_from_recipes(&recipe_ids, &state.db_pool).await?;

        tracing::info!(
            "Generating shopping list for week {} with {} ingredients from {} recipes",
            week.start_date,
            ingredients.len(),
            recipe_ids.len()
        );

        // Generate shopping list
        let shopping_list_cmd = shopping::GenerateShoppingListCommand {
            user_id: auth.user_id.clone(),
            meal_plan_id: week.id.clone(),
            week_start_date: week.start_date.clone(),
            ingredients,
        };

        match shopping::generate_shopping_list(shopping_list_cmd, &state.evento_executor).await {
            Ok(shopping_list_id) => {
                tracing::info!(
                    "Shopping list generated successfully: id={}, week={}, meal_plan_id={}",
                    shopping_list_id,
                    week.start_date,
                    week.id
                );
            }
            Err(e) => {
                tracing::error!(
                    "Failed to generate shopping list for week {}: {:?}",
                    week.start_date,
                    e
                );
                // Continue processing other weeks even if one fails
            }
        }
    }

    // Get the first week ID for polling redirect
    let first_week_id = multi_week_plan
        .generated_weeks
        .first()
        .map(|w| w.id.clone())
        .ok_or_else(|| anyhow::anyhow!("No weeks generated in multi-week plan"))?;

    // Return loading state that polls for read model completion
    // TwinSpark will poll /plan/check-ready until all weeks are fully projected
    let loading_template = MealPlanLoadingTemplate {
        user: Some(()),
        meal_plan_id: first_week_id.clone(),
        current_path: "/plan".to_string(),
    };

    loading_template.render().map(Html).map_err(|e| {
        tracing::error!("Failed to render meal plan loading template: {:?}", e);
        AppError::InternalError("Failed to render page".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: build_day_data() correctly sets is_today and is_past flags (Story 3.4 Review Action Item #5)
    #[test]
    fn test_build_day_data_date_highlighting() {
        use recipe::read_model::RecipeReadModel;

        // Mock data
        let today = chrono::Local::now().date_naive();
        let yesterday = today - chrono::Duration::days(1);
        let tomorrow = today + chrono::Duration::days(1);

        let assignments = vec![
            MealAssignmentReadModel {
                id: "assignment_yesterday".to_string(),
                meal_plan_id: "plan1".to_string(),
                date: yesterday.format("%Y-%m-%d").to_string(),
                course_type: "appetizer".to_string(), // AC-5: Use course_type instead of meal_type
                recipe_id: "recipe1".to_string(),
                prep_required: false,
                assignment_reasoning: None,
                accompaniment_recipe_id: None, // Story 9.2
            },
            MealAssignmentReadModel {
                id: "assignment_today".to_string(),
                meal_plan_id: "plan1".to_string(),
                date: today.format("%Y-%m-%d").to_string(),
                course_type: "appetizer".to_string(), // AC-5: Use course_type instead of meal_type
                recipe_id: "recipe2".to_string(),
                prep_required: false,
                assignment_reasoning: None,
                accompaniment_recipe_id: None, // Story 9.2
            },
            MealAssignmentReadModel {
                id: "assignment_tomorrow".to_string(),
                meal_plan_id: "plan1".to_string(),
                date: tomorrow.format("%Y-%m-%d").to_string(),
                course_type: "appetizer".to_string(), // AC-5: Use course_type instead of meal_type
                recipe_id: "recipe3".to_string(),
                prep_required: false,
                assignment_reasoning: None,
                accompaniment_recipe_id: None, // Story 9.2
            },
        ];

        let recipes = vec![
            RecipeReadModel {
                id: "recipe1".to_string(),
                user_id: "user1".to_string(),
                title: "Recipe 1".to_string(),
                recipe_type: "main_course".to_string(),
                ingredients: "[]".to_string(),
                instructions: "[]".to_string(),
                prep_time_min: Some(10),
                cook_time_min: Some(20),
                advance_prep_hours: None,
                serving_size: Some(4),
                is_favorite: true,
                is_shared: false,
                complexity: Some("simple".to_string()),
                cuisine: None,
                dietary_tags: None,
                accepts_accompaniment: false,
                preferred_accompaniments: None,
                accompaniment_category: None,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
            RecipeReadModel {
                id: "recipe2".to_string(),
                user_id: "user1".to_string(),
                title: "Recipe 2".to_string(),
                recipe_type: "appetizer".to_string(),
                ingredients: "[]".to_string(),
                instructions: "[]".to_string(),
                prep_time_min: Some(15),
                cook_time_min: Some(25),
                advance_prep_hours: None,
                serving_size: Some(4),
                is_favorite: true,
                is_shared: false,
                complexity: Some("moderate".to_string()),
                cuisine: None,
                dietary_tags: None,
                accepts_accompaniment: false,
                preferred_accompaniments: None,
                accompaniment_category: None,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
            RecipeReadModel {
                id: "recipe3".to_string(),
                user_id: "user1".to_string(),
                title: "Recipe 3".to_string(),
                recipe_type: "dessert".to_string(),
                ingredients: "[]".to_string(),
                instructions: "[]".to_string(),
                prep_time_min: Some(20),
                cook_time_min: Some(30),
                advance_prep_hours: None,
                serving_size: Some(4),
                is_favorite: true,
                is_shared: false,
                complexity: Some("complex".to_string()),
                cuisine: None,
                dietary_tags: None,
                accepts_accompaniment: false,
                preferred_accompaniments: None,
                accompaniment_category: None,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
        ];

        // Execute
        let days = build_day_data(&assignments, &recipes, &[], "test_meal_plan_id");

        // Assert
        assert_eq!(days.len(), 3, "Should have 3 days");

        // Find days by date
        let yesterday_day = days
            .iter()
            .find(|d| d.date == yesterday.format("%Y-%m-%d").to_string())
            .expect("Yesterday not found");
        let today_day = days
            .iter()
            .find(|d| d.date == today.format("%Y-%m-%d").to_string())
            .expect("Today not found");
        let tomorrow_day = days
            .iter()
            .find(|d| d.date == tomorrow.format("%Y-%m-%d").to_string())
            .expect("Tomorrow not found");

        // Verify is_past flag (yesterday should be past, others not)
        assert!(yesterday_day.is_past, "Yesterday should be marked as past");
        assert!(!today_day.is_past, "Today should NOT be marked as past");
        assert!(
            !tomorrow_day.is_past,
            "Tomorrow should NOT be marked as past"
        );

        // Verify is_today flag (only today should be marked)
        assert!(
            !yesterday_day.is_today,
            "Yesterday should NOT be marked as today"
        );
        assert!(today_day.is_today, "Today should be marked as today");
        assert!(
            !tomorrow_day.is_today,
            "Tomorrow should NOT be marked as today"
        );
    }
}
