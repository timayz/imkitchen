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
    pub assignment_id: String, // Story 3.4: Needed for "Replace This Meal" button
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
    pub day_name: String,     // "Monday", "Tuesday", etc.
    pub is_today: bool,       // AC-6: Today's date highlighted
    pub is_past: bool,        // AC-7: Past dates dimmed
    pub meal_plan_id: String, // Story 3.5: Needed for calendar context links
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
    pub rotation_used: usize,  // AC (Story 3.3): Rotation progress display
    pub rotation_total: usize, // AC (Story 3.3): Total favorites
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

            // AC (Story 3.3): Query rotation progress for display
            let (rotation_used, rotation_total) =
                MealPlanQueries::query_rotation_progress(&auth.user_id, &state.db_pool).await?;

            // Group assignments by date into DayData with today/past flags
            let days = build_day_data(&plan_data.assignments, &recipes, &plan_data.meal_plan.id);

            let template = MealCalendarTemplate {
                user: Some(()),
                days,
                start_date: plan_data.meal_plan.start_date,
                has_meal_plan: true,
                rotation_used,
                rotation_total,
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
                rotation_used: 0,
                rotation_total: 0,
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
    meal_plan_id: &str, // Story 3.5: Pass meal_plan_id for calendar context
) -> Vec<DayData> {
    use std::collections::HashMap;

    // AC-6, AC-7: Get today's date for highlighting logic
    let today = chrono::Local::now().date_naive();

    // Create recipe lookup map
    let recipe_map: HashMap<String, &RecipeReadModel> =
        recipes.iter().map(|r| (r.id.clone(), r)).collect();

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
            breakfast: None,
            lunch: None,
            dinner: None,
        });

        // Get recipe details
        let recipe = recipe_map.get(&assignment.recipe_id);

        if let Some(recipe) = recipe {
            let slot_data = MealSlotData {
                assignment_id: assignment.id.clone(), // Story 3.4: Include for replacement
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

/// POST /plan/meal/:assignment_id/replace - Replace a single meal assignment
///
/// Story 3.4: "Replace This Meal" button functionality
/// Respects rotation by only selecting from unused recipes in current cycle
pub async fn post_replace_meal(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
    axum::extract::Path(assignment_id): axum::extract::Path<String>,
) -> Result<Html<String>, AppError> {
    // Get the meal assignment to replace
    let assignment = sqlx::query_as::<_, MealAssignmentReadModel>(
        r#"
        SELECT ma.id, ma.meal_plan_id, ma.date, ma.meal_type, ma.recipe_id, ma.prep_required
        FROM meal_assignments ma
        JOIN meal_plans mp ON ma.meal_plan_id = mp.id
        WHERE ma.id = ?1 AND mp.user_id = ?2 AND mp.status = 'active'
        "#,
    )
    .bind(&assignment_id)
    .bind(&auth.user_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::InternalError("Meal assignment not found".to_string()))?;

    // Query rotation-aware replacement candidates (unused recipes in current cycle)
    let available_recipes = MealPlanQueries::query_replacement_candidates(
        &auth.user_id,
        &assignment.meal_type,
        &state.db_pool,
    )
    .await?;

    if available_recipes.is_empty() {
        return Err(AppError::InternalError(
            "No available recipes for replacement. All favorites used in current rotation cycle."
                .to_string(),
        ));
    }

    // Select a random replacement recipe from available pool
    // Uses assignment_id as seed for deterministic randomization (same assignment = same sequence)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    assignment_id.hash(&mut hasher);
    let hash = hasher.finish();

    let index = (hash as usize) % available_recipes.len();
    let replacement_recipe_id = &available_recipes[index];

    // Update the meal assignment with the new recipe
    sqlx::query(
        r#"
        UPDATE meal_assignments
        SET recipe_id = ?1
        WHERE id = ?2
        "#,
    )
    .bind(replacement_recipe_id)
    .bind(&assignment_id)
    .execute(&state.db_pool)
    .await?;

    // Fetch the replacement recipe details
    let replacement_recipe = sqlx::query_as::<_, RecipeReadModel>(
        r#"
        SELECT id, user_id, title, ingredients, instructions, prep_time_min, cook_time_min,
               advance_prep_hours, serving_size, is_favorite, is_shared, complexity, cuisine,
               dietary_tags, created_at, updated_at
        FROM recipes
        WHERE id = ?1 AND deleted_at IS NULL
        "#,
    )
    .bind(replacement_recipe_id)
    .fetch_one(&state.db_pool)
    .await?;

    // Check if advance prep required
    let prep_required = replacement_recipe
        .advance_prep_hours
        .map(|hours| hours > 0)
        .unwrap_or(false);

    // Render the updated meal slot HTML to return via TwinSpark
    let border_color = match assignment.meal_type.as_str() {
        "breakfast" => "yellow",
        "lunch" => "green",
        "dinner" => "blue",
        _ => "gray",
    };

    let complexity_badge = replacement_recipe
        .complexity
        .as_ref()
        .map(|c| {
            let badge_class = match c.as_str() {
                "simple" => "bg-green-100 text-green-800",
                "moderate" => "bg-yellow-100 text-yellow-800",
                _ => "bg-red-100 text-red-800",
            };
            format!(
                r#"<span class="text-xs px-2 py-1 rounded {}">{}</span>"#,
                badge_class, c
            )
        })
        .unwrap_or_default();

    let prep_time_html = replacement_recipe
        .prep_time_min
        .map(|p| format!(r#"<span>🔪 {}m</span>"#, p))
        .unwrap_or_default();

    let cook_time_html = replacement_recipe
        .cook_time_min
        .map(|c| format!(r#"<span>🔥 {}m</span>"#, c))
        .unwrap_or_default();

    let prep_indicator = if prep_required {
        r#"<span class="text-orange-600 font-semibold" title="Advance prep required">⏰</span>"#
    } else {
        ""
    };

    // HTML escape user-controlled data (recipe title) for XSS prevention
    let escaped_title = replacement_recipe
        .title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;");

    let meal_slot_html = format!(
        "<div id=\"meal-slot-{}\" class=\"border-l-4 border-{}-400 pl-3 py-2\">\
    <div class=\"flex items-center justify-between mb-1\">\
        <span class=\"text-xs font-semibold text-gray-500 uppercase\">{}</span>\
        {}\
    </div>\
    <a href=\"/recipes/{}\" class=\"text-gray-900 hover:text-primary-500 font-medium\">\
        {}\
    </a>\
    <div class=\"flex items-center gap-2 mt-1 text-xs text-gray-500\">\
        {}\
        {}\
        {}\
    </div>\
    <button \
        ts-req=\"/plan/meal/{}/replace\" \
        ts-req-method=\"POST\" \
        ts-target=\"#meal-slot-{}\" \
        ts-swap=\"outerHTML\" \
        class=\"mt-2 text-xs text-primary-600 hover:text-primary-800 underline\">\
        Replace This Meal\
    </button>\
</div>",
        assignment_id,
        border_color,
        assignment.meal_type,
        complexity_badge,
        replacement_recipe.id,
        escaped_title,
        prep_time_html,
        cook_time_html,
        prep_indicator,
        assignment_id,
        assignment_id,
    );

    Ok(Html(meal_slot_html))
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
                meal_type: "breakfast".to_string(),
                recipe_id: "recipe1".to_string(),
                prep_required: false,
            },
            MealAssignmentReadModel {
                id: "assignment_today".to_string(),
                meal_plan_id: "plan1".to_string(),
                date: today.format("%Y-%m-%d").to_string(),
                meal_type: "breakfast".to_string(),
                recipe_id: "recipe2".to_string(),
                prep_required: false,
            },
            MealAssignmentReadModel {
                id: "assignment_tomorrow".to_string(),
                meal_plan_id: "plan1".to_string(),
                date: tomorrow.format("%Y-%m-%d").to_string(),
                meal_type: "breakfast".to_string(),
                recipe_id: "recipe3".to_string(),
                prep_required: false,
            },
        ];

        let recipes = vec![
            RecipeReadModel {
                id: "recipe1".to_string(),
                user_id: "user1".to_string(),
                title: "Recipe 1".to_string(),
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
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
            RecipeReadModel {
                id: "recipe2".to_string(),
                user_id: "user1".to_string(),
                title: "Recipe 2".to_string(),
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
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
            RecipeReadModel {
                id: "recipe3".to_string(),
                user_id: "user1".to_string(),
                title: "Recipe 3".to_string(),
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
                created_at: "2025-01-01T00:00:00Z".to_string(),
                updated_at: "2025-01-01T00:00:00Z".to_string(),
            },
        ];

        // Execute
        let days = build_day_data(&assignments, &recipes, "test_meal_plan_id");

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
