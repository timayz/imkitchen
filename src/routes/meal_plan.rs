use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use chrono::{Datelike, Duration, NaiveDate, Utc, Weekday};
use meal_planning::{
    algorithm::{MealPlanningAlgorithm, RecipeForPlanning, UserConstraints},
    events::MealPlanGenerated,
    read_model::{MealAssignmentReadModel, MealPlanQueries},
    rotation::RotationState,
};
use recipe::read_model::{query_recipes_by_user, RecipeReadModel};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

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

/// Recipe with meal assignment for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealSlotData {
    pub date: String,
    pub meal_type: String,
    pub recipe_id: String,
    pub recipe_title: String,
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub prep_required: bool,
    pub complexity: Option<String>,
}

/// Day with 3 meal slots for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayData {
    pub date: String,
    pub day_name: String, // "Monday", "Tuesday", etc.
    pub breakfast: Option<MealSlotData>,
    pub lunch: Option<MealSlotData>,
    pub dinner: Option<MealSlotData>,
}

#[derive(Template)]
#[template(path = "pages/meal-calendar.html")]
pub struct MealCalendarTemplate {
    pub user: Option<()>,
    pub days: Vec<DayData>,
    pub start_date: String,
    pub has_meal_plan: bool,
}

/// GET /plan - Display meal calendar view
///
/// AC-5: Week-view calendar displays generated plan with breakfast/lunch/dinner slots filled
/// AC-9: User redirected to calendar view after successful generation
pub async fn get_meal_plan(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    // Query active meal plan for user
    let meal_plan_with_assignments =
        MealPlanQueries::get_active_meal_plan_with_assignments(&auth.user_id, &state.db_pool)
            .await?;

    match meal_plan_with_assignments {
        Some(plan_data) => {
            // Fetch recipe details for all assignments
            let recipe_ids: Vec<String> = plan_data
                .assignments
                .iter()
                .map(|a| a.recipe_id.clone())
                .collect();

            let recipes = fetch_recipes_by_ids(&recipe_ids, &state.db_pool).await?;

            // Group assignments by date into DayData
            let days = build_day_data(&plan_data.assignments, &recipes);

            let template = MealCalendarTemplate {
                user: Some(()),
                days,
                start_date: plan_data.meal_plan.start_date,
                has_meal_plan: true,
            };
            template.render().map(Html).map_err(|e| {
                tracing::error!("Failed to render meal calendar template: {:?}", e);
                AppError::InternalError("Failed to render page".to_string())
            })
        }
        None => {
            // No meal plan exists, show empty calendar or prompt
            let template = MealCalendarTemplate {
                user: Some(()),
                days: Vec::new(),
                start_date: String::new(),
                has_meal_plan: false,
            };
            template.render().map(Html).map_err(|e| {
                tracing::error!("Failed to render empty meal calendar template: {:?}", e);
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
) -> Result<impl IntoResponse, AppError> {
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

            RecipeForPlanning {
                id: r.id,
                title: r.title,
                ingredients_count: ingredients.len(),
                instructions_count: instructions.len(),
                prep_time_min: r.prep_time_min.map(|v| v as u32),
                cook_time_min: r.cook_time_min.map(|v| v as u32),
                advance_prep_hours: r.advance_prep_hours.map(|v| v as u32),
                complexity: r.complexity,
            }
        })
        .collect();

    // Load user profile constraints (future: query from user profile table)
    let constraints = UserConstraints::default();

    // Load rotation state from most recent meal plan
    let previous_meal_plan =
        MealPlanQueries::get_active_meal_plan(&auth.user_id, &state.db_pool).await?;
    let mut rotation_state = match previous_meal_plan {
        Some(plan) => RotationState::from_json(&plan.rotation_state).unwrap_or_else(|e| {
            tracing::warn!(
                "Failed to parse rotation state for user {}: {}. Using default.",
                auth.user_id,
                e
            );
            RotationState::default()
        }),
        None => {
            RotationState::with_favorite_count(recipes_for_planning.len()).unwrap_or_else(|e| {
                tracing::warn!(
                    "Failed to create rotation state with count {}: {}. Using default.",
                    recipes_for_planning.len(),
                    e
                );
                RotationState::default()
            })
        }
    };

    // Ensure total_favorite_count is set correctly
    rotation_state.total_favorite_count = recipes_for_planning.len();

    // Store values for later use
    let old_cycle_number = rotation_state.cycle_number;
    let favorite_count = recipes_for_planning.len();

    // Calculate start date (next Monday)
    let start_date = get_next_monday();

    // AC-2, AC-3, AC-4, AC-6: Generate meal plan using algorithm
    // AC-9: Pass None for seed to get random variety (timestamp-based)
    let (meal_assignments, updated_rotation_state) = MealPlanningAlgorithm::generate(
        &start_date,
        recipes_for_planning,
        constraints,
        rotation_state,
        None, // None = use timestamp for variety
    )?;

    // Detect if cycle was reset during generation
    let cycle_reset_occurred = updated_rotation_state.cycle_number > old_cycle_number;

    // Create MealPlanGenerated event via evento
    let now = Utc::now().to_rfc3339();

    let event_data = MealPlanGenerated {
        user_id: auth.user_id.clone(),
        start_date: start_date.clone(),
        meal_assignments: meal_assignments.clone(),
        rotation_state_json: updated_rotation_state.to_json()?,
        generated_at: now.clone(),
    };

    // AC-8: Emit MealPlanGenerated event via evento (this will trigger read model projection)
    // evento::create() generates a ULID for the aggregator_id (meal_plan_id)
    tracing::info!(
        "Creating MealPlanGenerated event with {} assignments",
        event_data.meal_assignments.len()
    );
    tracing::debug!(
        "Event data: user_id={}, start_date={}",
        event_data.user_id,
        event_data.start_date
    );

    let meal_plan_id = evento::create::<meal_planning::MealPlanAggregate>()
        .data(&event_data)
        .map_err(|e| {
            tracing::error!("Failed to encode event data: {:?}", e);
            anyhow::anyhow!("Failed to encode event data: {}", e)
        })?
        .metadata(&true)
        .map_err(|e| {
            tracing::error!("Failed to encode metadata: {:?}", e);
            anyhow::anyhow!("Failed to encode metadata: {}", e)
        })?
        .commit(&state.evento_executor)
        .await
        .map_err(|e| {
            tracing::error!("Failed to commit event: {:?}", e);
            anyhow::anyhow!("Failed to commit event: {}", e)
        })?;

    // AC-1: Emit RecipeUsedInRotation events for each unique recipe used in this generation
    // This enables rotation tracking in the recipe_rotation_state table
    use std::collections::HashSet;
    let unique_recipe_ids: HashSet<String> = meal_assignments
        .iter()
        .map(|a| a.recipe_id.clone())
        .collect();

    for recipe_id in &unique_recipe_ids {
        use meal_planning::events::RecipeUsedInRotation;

        let rotation_event = RecipeUsedInRotation {
            recipe_id: recipe_id.clone(),
            cycle_number: updated_rotation_state.cycle_number,
            used_at: now.clone(),
        };

        tracing::debug!(
            "Emitting RecipeUsedInRotation: recipe_id={}, cycle={}",
            recipe_id,
            updated_rotation_state.cycle_number
        );

        evento::save::<meal_planning::MealPlanAggregate>(&meal_plan_id)
            .data(&rotation_event)
            .map_err(|e| {
                tracing::error!("Failed to encode RecipeUsedInRotation event: {:?}", e);
                anyhow::anyhow!("Failed to encode rotation event: {}", e)
            })?
            .metadata(&true)
            .map_err(|e| {
                tracing::error!("Failed to encode metadata: {:?}", e);
                anyhow::anyhow!("Failed to encode metadata: {}", e)
            })?
            .commit(&state.evento_executor)
            .await
            .map_err(|e| {
                tracing::error!("Failed to commit rotation event: {:?}", e);
                anyhow::anyhow!("Failed to commit rotation event: {}", e)
            })?;
    }

    tracing::info!(
        "Emitted {} RecipeUsedInRotation events for cycle {}",
        unique_recipe_ids.len(),
        updated_rotation_state.cycle_number
    );

    // **Critical Fix 1.3:** Emit RotationCycleReset event if cycle was reset
    if cycle_reset_occurred {
        use meal_planning::events::RotationCycleReset;

        let reset_event = RotationCycleReset {
            user_id: auth.user_id.clone(),
            old_cycle_number,
            new_cycle_number: updated_rotation_state.cycle_number,
            favorite_count,
            reset_at: now.clone(),
        };

        tracing::info!(
            "Rotation cycle reset: {} -> {} for user {}",
            old_cycle_number,
            updated_rotation_state.cycle_number,
            auth.user_id
        );

        evento::save::<meal_planning::MealPlanAggregate>(&meal_plan_id)
            .data(&reset_event)
            .map_err(|e| {
                tracing::error!("Failed to encode RotationCycleReset event: {:?}", e);
                anyhow::anyhow!("Failed to encode reset event: {}", e)
            })?
            .metadata(&true)
            .map_err(|e| {
                tracing::error!("Failed to encode metadata: {:?}", e);
                anyhow::anyhow!("Failed to encode metadata: {}", e)
            })?
            .commit(&state.evento_executor)
            .await
            .map_err(|e| {
                tracing::error!("Failed to commit reset event: {:?}", e);
                anyhow::anyhow!("Failed to commit reset event: {}", e)
            })?;
    }

    // AC-9: Redirect to calendar view
    Ok(Redirect::to("/plan"))
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
        "SELECT id, user_id, title, ingredients, instructions, prep_time_min, cook_time_min, advance_prep_hours, serving_size, is_favorite, is_shared, complexity, cuisine, dietary_tags, created_at, updated_at FROM recipes WHERE id IN ({}) AND deleted_at IS NULL",
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
) -> Vec<DayData> {
    use std::collections::HashMap;

    // Create recipe lookup map
    let recipe_map: HashMap<String, &RecipeReadModel> =
        recipes.iter().map(|r| (r.id.clone(), r)).collect();

    // Group assignments by date
    let mut days_map: HashMap<String, DayData> = HashMap::new();

    for assignment in assignments {
        let date = assignment.date.clone();

        // Parse date to get day name
        let day_name = if let Ok(parsed_date) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
            parsed_date.weekday().to_string()
        } else {
            String::new()
        };

        // Get or create DayData
        let day_data = days_map.entry(date.clone()).or_insert(DayData {
            date: date.clone(),
            day_name,
            breakfast: None,
            lunch: None,
            dinner: None,
        });

        // Get recipe details
        let recipe = recipe_map.get(&assignment.recipe_id);

        if let Some(recipe) = recipe {
            let slot_data = MealSlotData {
                date: assignment.date.clone(),
                meal_type: assignment.meal_type.clone(),
                recipe_id: recipe.id.clone(),
                recipe_title: recipe.title.clone(),
                prep_time_min: recipe.prep_time_min,
                cook_time_min: recipe.cook_time_min,
                prep_required: assignment.prep_required,
                complexity: recipe.complexity.clone(),
            };

            // Assign to appropriate meal slot
            match assignment.meal_type.as_str() {
                "breakfast" => day_data.breakfast = Some(slot_data),
                "lunch" => day_data.lunch = Some(slot_data),
                "dinner" => day_data.dinner = Some(slot_data),
                _ => {}
            }
        }
    }

    // Sort days by date and return as Vec
    let mut days: Vec<DayData> = days_map.into_values().collect();
    days.sort_by(|a, b| a.date.cmp(&b.date));
    days
}

/// Helper: Get next Monday's date as ISO 8601 string
fn get_next_monday() -> String {
    let today = Utc::now().naive_utc().date();
    let days_until_monday = match today.weekday() {
        Weekday::Mon => 7, // If today is Monday, next Monday is 7 days away
        Weekday::Tue => 6,
        Weekday::Wed => 5,
        Weekday::Thu => 4,
        Weekday::Fri => 3,
        Weekday::Sat => 2,
        Weekday::Sun => 1,
    };

    let next_monday = today + Duration::days(days_until_monday);
    next_monday.format("%Y-%m-%d").to_string()
}
