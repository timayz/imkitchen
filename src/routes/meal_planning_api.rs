/// Story 8.1: Multi-Week Meal Plan Generation API Route
///
/// RESTful JSON API endpoint for generating multi-week meal plans.
/// Integrates with Epic 7 algorithm and evento event sourcing.
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    Extension,
};
use chrono::Utc;
use meal_planning::{
    algorithm::{generate_multi_week_meal_plans, RecipeForPlanning, SkillLevel, UserPreferences},
    events::{MultiWeekMealPlanGenerated, WeekMealPlanData},
};
use recipe::{
    read_model::{query_recipes_by_user, RecipeReadModel},
    AccompanimentCategory, Cuisine,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

use crate::error::AppError;
use crate::middleware::auth::Auth;
use crate::routes::AppState;

/// JSON response for multi-week meal plan generation (Story 8.1 AC-8)
///
/// Returns first week data with navigation links to all generated weeks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiWeekResponse {
    pub generation_batch_id: String,
    pub max_weeks_possible: usize,
    pub current_week_index: usize,
    pub first_week: WeekData,
    pub navigation: NavigationData,
}

/// Week data for JSON response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekData {
    pub id: String,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
    pub is_locked: bool,
    pub meal_assignments: Vec<MealAssignmentData>,
    pub shopping_list_id: Option<String>,
}

/// Meal assignment for JSON response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealAssignmentData {
    pub id: String,
    pub date: String,
    pub course_type: String,
    pub recipe: RecipeData,
    pub accompaniment: Option<AccompanimentData>,
    pub prep_required: bool,
    pub algorithm_reasoning: Option<String>,
}

/// Recipe data for JSON response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeData {
    pub id: String,
    pub title: String,
    pub prep_time_min: Option<u32>,
    pub cook_time_min: Option<u32>,
    pub complexity: Option<String>,
}

/// Accompaniment data for JSON response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccompanimentData {
    pub id: String,
    pub title: String,
    pub category: String,
}

/// Navigation data with week links
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationData {
    pub next_week_id: Option<String>,
    pub week_links: Vec<WeekLink>,
}

/// Week link for navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekLink {
    pub week_id: String,
    pub start_date: String,
    pub is_current: bool,
}

/// Error response with actionable guidance (Story 8.1 AC-9, AC-10)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<ActionLink>,
}

/// Action link for error recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionLink {
    pub label: String,
    pub url: String,
}

/// POST /plan/generate-multi-week route handler (Story 8.1 AC-1)
///
/// Generates a multi-week meal plan for the authenticated user by calling
/// the Epic 7 algorithm and emitting a MultiWeekMealPlanGenerated event.
///
/// # Authentication
/// Protected by JWT cookie middleware (AC-2)
///
/// # Returns
/// - 200 OK: JSON with first week data + navigation links (AC-8)
/// - 400 Bad Request: InsufficientRecipes error with category counts (AC-9)
/// - 500 Internal Server Error: AlgorithmTimeout or internal error (AC-10)
#[tracing::instrument(skip(state), fields(user_id = %auth.user_id))]
pub async fn generate_multi_week_meal_plan(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>, // AC-2, AC-3: Extract user_id from JWT
) -> Result<Json<MultiWeekResponse>, ApiError> {
    let user_id = &auth.user_id;

    tracing::info!(user_id = %user_id, "Multi-week meal plan generation requested");

    // AC-4: Load user's favorite recipes from database
    let favorite_recipes = load_favorite_recipes(user_id, &state.db_pool).await?;

    tracing::debug!(
        user_id = %user_id,
        recipe_count = favorite_recipes.len(),
        "Loaded favorite recipes"
    );

    // Validate minimum recipe count (minimum 7 per category required by algorithm)
    if favorite_recipes.len() < 7 {
        tracing::warn!(
            user_id = %user_id,
            count = favorite_recipes.len(),
            "Insufficient recipes for meal plan generation"
        );
        return Err(ApiError::InsufficientRecipes {
            appetizers: favorite_recipes
                .iter()
                .filter(|r| r.recipe_type == "appetizer")
                .count(),
            main_courses: favorite_recipes
                .iter()
                .filter(|r| r.recipe_type == "main_course")
                .count(),
            desserts: favorite_recipes
                .iter()
                .filter(|r| r.recipe_type == "dessert")
                .count(),
        });
    }

    // AC-5: Load user's meal planning preferences from users table
    let preferences = load_user_preferences(user_id, &state.db_pool).await?;

    tracing::debug!(
        user_id = %user_id,
        "Loaded meal planning preferences: max_prep_weeknight={}, max_prep_weekend={}",
        preferences.max_prep_time_weeknight,
        preferences.max_prep_time_weekend
    );

    // AC-6: Call generate_multi_week_meal_plans algorithm (Epic 7)
    tracing::info!(user_id = %user_id, "Calling multi-week meal plan algorithm");

    let multi_week_plan =
        generate_multi_week_meal_plans(user_id.to_string(), favorite_recipes.clone(), preferences)
            .await
            .map_err(|e| {
                tracing::error!(user_id = %user_id, error = %e, "Algorithm execution failed");
                match e {
                    meal_planning::MealPlanningError::InsufficientRecipes {
                        minimum: _,
                        current: _,
                    } => {
                        // Calculate counts per category for better error message
                        let appetizers = favorite_recipes
                            .iter()
                            .filter(|r| r.recipe_type == "appetizer")
                            .count();
                        let main_courses = favorite_recipes
                            .iter()
                            .filter(|r| r.recipe_type == "main_course")
                            .count();
                        let desserts = favorite_recipes
                            .iter()
                            .filter(|r| r.recipe_type == "dessert")
                            .count();

                        ApiError::InsufficientRecipes {
                            appetizers,
                            main_courses,
                            desserts,
                        }
                    }
                    _ => ApiError::AlgorithmTimeout,
                }
            })?;

    tracing::info!(
        user_id = %user_id,
        weeks_generated = multi_week_plan.generated_weeks.len(),
        "Algorithm execution completed successfully"
    );

    // AC-7: Emit MultiWeekMealPlanGenerated evento event
    let generation_batch_id = multi_week_plan.generation_batch_id.clone();
    let weeks_data: Vec<WeekMealPlanData> = multi_week_plan
        .generated_weeks
        .iter()
        .map(|week| WeekMealPlanData {
            id: week.id.clone(),
            start_date: week.start_date.clone(),
            end_date: week.end_date.clone(),
            status: week.status,
            is_locked: week.is_locked,
            meal_assignments: week.meal_assignments.clone(),
            shopping_list_id: week.shopping_list_id.clone(),
        })
        .collect();

    let event = MultiWeekMealPlanGenerated {
        generation_batch_id: generation_batch_id.clone(),
        user_id: user_id.to_string(),
        weeks: weeks_data,
        rotation_state: multi_week_plan.rotation_state.clone(),
        generated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<meal_planning::MealPlanAggregate>()
        .data(&event)
        .map_err(|e| {
            tracing::error!(
                user_id = %user_id,
                error = %e,
                "Failed to encode MultiWeekMealPlanGenerated event"
            );
            ApiError::InternalServerError(format!("Failed to encode event: {}", e))
        })?
        .metadata(&true)
        .map_err(|e| {
            tracing::error!(
                user_id = %user_id,
                error = %e,
                "Failed to encode event metadata"
            );
            ApiError::InternalServerError(format!("Failed to encode metadata: {}", e))
        })?
        .commit(&state.evento_executor)
        .await
        .map_err(|e| {
            tracing::error!(
                user_id = %user_id,
                error = %e,
                "Failed to commit MultiWeekMealPlanGenerated event to evento"
            );
            ApiError::InternalServerError(format!("Failed to commit event: {}", e))
        })?;

    tracing::info!(
        user_id = %user_id,
        generation_batch_id = %generation_batch_id,
        "MultiWeekMealPlanGenerated event emitted successfully"
    );

    // AC-8: Build JSON response with first week data + navigation links
    let response = build_multi_week_response(
        &multi_week_plan.generated_weeks,
        &generation_batch_id,
        &favorite_recipes,
    )
    .await?;

    Ok(Json(response))
}

/// Load user's favorite recipes from database (Story 8.1 AC-4)
///
/// Queries recipes table for all favorite recipes owned by the user.
async fn load_favorite_recipes(
    user_id: &str,
    db_pool: &SqlitePool,
) -> Result<Vec<RecipeForPlanning>, ApiError> {
    let recipes: Vec<RecipeReadModel> = query_recipes_by_user(user_id, false, db_pool)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to query recipes: {}", e)))?;

    let favorite_recipes: Vec<RecipeForPlanning> = recipes
        .into_iter()
        .filter(|r| r.is_favorite)
        .map(|r| {
            // Use recipe_type (it's already a String, not Option<String>)
            // Use recipe_type string from read model
            let recipe_type = r.recipe_type.clone();

            // Parse complexity with fallback based on total time
            let complexity = r.complexity.or_else(|| {
                let total_time = r.prep_time_min.unwrap_or(0) + r.cook_time_min.unwrap_or(0);
                Some(if total_time < 30 {
                    "simple".to_string()
                } else if total_time < 60 {
                    "moderate".to_string()
                } else {
                    "complex".to_string()
                })
            });

            // Parse ingredients count
            let ingredients_str: &String = &r.ingredients;
            let ingredients_count = serde_json::from_str::<Vec<serde_json::Value>>(ingredients_str)
                .ok()
                .map(|arr| arr.len())
                .unwrap_or(0);

            // Parse instructions count
            let instructions_str: &String = &r.instructions;
            let instructions_count =
                serde_json::from_str::<Vec<serde_json::Value>>(instructions_str)
                    .ok()
                    .map(|arr| arr.len())
                    .unwrap_or(0);

            // Parse dietary tags
            let dietary_tags: Vec<String> = r
                .dietary_tags
                .as_ref()
                .and_then(|json_str| serde_json::from_str(json_str).ok())
                .unwrap_or_default();

            // Parse cuisine from database field, fallback to Italian
            let cuisine = r
                .cuisine
                .as_ref()
                .and_then(|c| match c.to_lowercase().as_str() {
                    "italian" => Some(Cuisine::Italian),
                    "indian" => Some(Cuisine::Indian),
                    "mexican" => Some(Cuisine::Mexican),
                    "chinese" => Some(Cuisine::Chinese),
                    "japanese" => Some(Cuisine::Japanese),
                    "french" => Some(Cuisine::French),
                    "american" => Some(Cuisine::American),
                    "mediterranean" => Some(Cuisine::Mediterranean),
                    "thai" => Some(Cuisine::Thai),
                    "korean" => Some(Cuisine::Korean),
                    _ => None,
                })
                .unwrap_or(Cuisine::Italian);

            // Parse accompaniment fields
            let accepts_accompaniment = r.accepts_accompaniment;
            let preferred_accompaniments: Vec<AccompanimentCategory> = r
                .preferred_accompaniments
                .as_ref()
                .and_then(|json_str| serde_json::from_str(json_str).ok())
                .unwrap_or_default();
            let accompaniment_category: Option<AccompanimentCategory> = r
                .accompaniment_category
                .as_ref()
                .and_then(|cat_str| serde_json::from_str(cat_str).ok());

            RecipeForPlanning {
                id: r.id,
                title: r.title,
                recipe_type,
                ingredients_count,
                instructions_count,
                prep_time_min: r.prep_time_min.map(|t| t as u32),
                cook_time_min: r.cook_time_min.map(|t| t as u32),
                advance_prep_hours: r.advance_prep_hours.map(|h| h as u32),
                complexity,
                dietary_tags,
                cuisine,
                accepts_accompaniment,
                preferred_accompaniments,
                accompaniment_category,
            }
        })
        .collect();

    Ok(favorite_recipes)
}

/// Load user's meal planning preferences from users table (Story 8.1 AC-5)
///
/// Queries users table for dietary restrictions, skill level, and time constraints.
async fn load_user_preferences(
    user_id: &str,
    db_pool: &SqlitePool,
) -> Result<UserPreferences, ApiError> {
    let user_row = sqlx::query(
        r#"
        SELECT
            dietary_restrictions,
            skill_level,
            max_prep_time_weeknight,
            max_prep_time_weekend,
            avoid_consecutive_complex,
            cuisine_variety_weight
        FROM users
        WHERE id = ?1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db_pool)
    .await?;

    match user_row {
        Some(row) => {
            // Parse dietary restrictions from JSON array
            let dietary_str: Option<String> = row.get("dietary_restrictions");
            let dietary_restrictions: Vec<String> = dietary_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();

            // Parse skill level with fallback to Intermediate
            let skill_level_str: Option<String> = row.get("skill_level");
            let skill_level = skill_level_str
                .as_deref()
                .and_then(|s| match s.to_lowercase().as_str() {
                    "beginner" => Some(SkillLevel::Beginner),
                    "intermediate" => Some(SkillLevel::Intermediate),
                    "advanced" => Some(SkillLevel::Advanced),
                    _ => None,
                })
                .unwrap_or(SkillLevel::Intermediate);

            // Parse time constraints with defaults
            let max_prep_time_weeknight: Option<i32> = row.get("max_prep_time_weeknight");
            let max_prep_time_weekend: Option<i32> = row.get("max_prep_time_weekend");
            let avoid_consecutive_complex: Option<bool> = row.get("avoid_consecutive_complex");
            let cuisine_variety_weight: Option<f64> = row.get("cuisine_variety_weight");

            Ok(UserPreferences {
                dietary_restrictions,
                max_prep_time_weeknight: max_prep_time_weeknight.unwrap_or(30) as u32,
                max_prep_time_weekend: max_prep_time_weekend.unwrap_or(90) as u32,
                skill_level,
                avoid_consecutive_complex: avoid_consecutive_complex.unwrap_or(true),
                cuisine_variety_weight: cuisine_variety_weight.unwrap_or(0.7) as f32,
            })
        }
        None => {
            tracing::warn!(
                "User {} not found in read model, using default preferences",
                user_id
            );
            Ok(UserPreferences::default())
        }
    }
}

/// Build JSON response from generated weeks (Story 8.1 AC-8)
async fn build_multi_week_response(
    generated_weeks: &[meal_planning::events::WeekMealPlan],
    generation_batch_id: &str,
    recipes: &[RecipeForPlanning],
) -> Result<MultiWeekResponse, ApiError> {
    // Extract first week
    let first_week = generated_weeks
        .first()
        .ok_or_else(|| ApiError::InternalServerError("No weeks generated".to_string()))?;

    // Build meal assignments for first week
    let meal_assignments: Vec<MealAssignmentData> = first_week
        .meal_assignments
        .iter()
        .map(|assignment| {
            // Find recipe details
            let recipe = recipes
                .iter()
                .find(|r| r.id == assignment.recipe_id)
                .ok_or_else(|| {
                    ApiError::InternalServerError(format!(
                        "Recipe {} not found",
                        assignment.recipe_id
                    ))
                })?;

            // Build accompaniment data if present
            let accompaniment = assignment
                .accompaniment_recipe_id
                .as_ref()
                .and_then(|acc_id| {
                    recipes.iter().find(|r| r.id == *acc_id).map(|acc_recipe| {
                        let category = acc_recipe
                            .accompaniment_category
                            .as_ref()
                            .map(|cat| match cat {
                                AccompanimentCategory::Pasta => "pasta",
                                AccompanimentCategory::Rice => "rice",
                                AccompanimentCategory::Fries => "fries",
                                AccompanimentCategory::Salad => "salad",
                                AccompanimentCategory::Bread => "bread",
                                AccompanimentCategory::Vegetable => "vegetable",
                                AccompanimentCategory::Other => "other",
                            })
                            .unwrap_or("other")
                            .to_string();

                        AccompanimentData {
                            id: acc_recipe.id.clone(),
                            title: acc_recipe.title.clone(),
                            category,
                        }
                    })
                });

            Ok(MealAssignmentData {
                id: uuid::Uuid::new_v4().to_string(), // Generate assignment ID
                date: assignment.date.clone(),
                course_type: assignment.course_type.clone(),
                recipe: RecipeData {
                    id: recipe.id.clone(),
                    title: recipe.title.clone(),
                    prep_time_min: recipe.prep_time_min,
                    cook_time_min: recipe.cook_time_min,
                    complexity: recipe.complexity.clone(),
                },
                accompaniment,
                prep_required: assignment.prep_required,
                algorithm_reasoning: assignment.assignment_reasoning.clone(),
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    // Build navigation links
    let week_links: Vec<WeekLink> = generated_weeks
        .iter()
        .enumerate()
        .map(|(idx, week)| WeekLink {
            week_id: week.id.clone(),
            start_date: week.start_date.clone(),
            is_current: idx == 0,
        })
        .collect();

    let next_week_id = generated_weeks.get(1).map(|w| w.id.clone());

    Ok(MultiWeekResponse {
        generation_batch_id: generation_batch_id.to_string(),
        max_weeks_possible: generated_weeks.len(),
        current_week_index: 0,
        first_week: WeekData {
            id: first_week.id.clone(),
            start_date: first_week.start_date.clone(),
            end_date: first_week.end_date.clone(),
            status: match first_week.status {
                meal_planning::events::WeekStatus::Future => "future".to_string(),
                meal_planning::events::WeekStatus::Current => "current".to_string(),
                meal_planning::events::WeekStatus::Past => "past".to_string(),
                meal_planning::events::WeekStatus::Archived => "archived".to_string(),
            },
            is_locked: false, // First week is always future (not locked)
            meal_assignments,
            shopping_list_id: None, // Shopping list generated by projection
        },
        navigation: NavigationData {
            next_week_id,
            week_links,
        },
    })
}

/// API-specific error types with JSON responses (Story 8.1 AC-9, AC-10)
#[derive(Debug)]
pub enum ApiError {
    InsufficientRecipes {
        appetizers: usize,
        main_courses: usize,
        desserts: usize,
    },
    AlgorithmTimeout,
    InternalServerError(String),
    DatabaseError(sqlx::Error),
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::DatabaseError(e)
    }
}

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        ApiError::InternalServerError(e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            ApiError::InsufficientRecipes {
                appetizers,
                main_courses,
                desserts,
            } => {
                let total = appetizers + main_courses + desserts;
                let details = serde_json::json!({
                    "appetizers": appetizers,
                    "main_courses": main_courses,
                    "desserts": desserts,
                    "total": total,
                    "required": 7,
                });

                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse {
                        error: "InsufficientRecipes".to_string(),
                        message: format!(
                            "You need at least 7 favorite recipes to generate a meal plan. You have {}: {} appetizers, {} main courses, {} desserts.",
                            total, appetizers, main_courses, desserts
                        ),
                        details: Some(details),
                        action: Some(ActionLink {
                            label: "Add More Recipes".to_string(),
                            url: "/recipes/new".to_string(),
                        }),
                    },
                )
            }
            ApiError::AlgorithmTimeout => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: "AlgorithmTimeout".to_string(),
                    message: "Meal plan generation took too long. Please try again.".to_string(),
                    details: None,
                    action: Some(ActionLink {
                        label: "Retry Generation".to_string(),
                        url: "/plan/generate-multi-week".to_string(),
                    }),
                },
            ),
            ApiError::DatabaseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: "DatabaseError".to_string(),
                    message: "Database error occurred. Please try again later.".to_string(),
                    details: Some(serde_json::json!({ "error": e.to_string() })),
                    action: None,
                },
            ),
            ApiError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse {
                    error: "InternalServerError".to_string(),
                    message: msg,
                    details: None,
                    action: None,
                },
            ),
        };

        (status, Json(error_response)).into_response()
    }
}
