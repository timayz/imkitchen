/// Story 8.1 & 8.2: Multi-Week Meal Plan Generation and Week Navigation API Routes
///
/// RESTful JSON API endpoints for generating multi-week meal plans and navigating week details.
/// Integrates with Epic 7 algorithm and evento event sourcing.
use axum::{
    extract::{Path, State},
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

/// JSON response for week detail navigation (Story 8.2 AC-6)
///
/// Returns full week data with meal assignments and shopping list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekDetailResponse {
    pub week: WeekDetailData,
    pub navigation: WeekNavigationData,
}

/// Week detail data with complete meal assignments and shopping list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekDetailData {
    pub id: String,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
    pub is_locked: bool,
    pub meal_assignments: Vec<MealAssignmentData>,
    pub shopping_list: Option<ShoppingListData>,
}

/// Shopping list data with categorized items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingListData {
    pub id: String,
    pub categories: Vec<ShoppingCategoryData>,
}

/// Shopping list category with items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingCategoryData {
    pub name: String,
    pub items: Vec<ShoppingItemData>,
}

/// Shopping list item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingItemData {
    pub ingredient_name: String,
    pub quantity: f32,
    pub unit: String,
    pub from_recipe_ids: Vec<String>,
}

/// Week navigation data with previous/next links
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekNavigationData {
    pub previous_week_id: Option<String>,
    pub next_week_id: Option<String>,
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
pub async fn load_user_preferences(
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

/// API-specific error types with JSON responses (Story 8.1 AC-9, AC-10, Story 8.2 AC-7, AC-8, Story 8.3 AC-7, AC-8, Story 8.4 AC-2)
#[derive(Debug)]
pub enum ApiError {
    InsufficientRecipes {
        appetizers: usize,
        main_courses: usize,
        desserts: usize,
    },
    AlgorithmTimeout,
    WeekNotFound,         // Story 8.2 AC-7
    Forbidden,            // Story 8.2 AC-8
    WeekLocked,           // Story 8.3 AC-7: Cannot regenerate locked/current week
    WeekAlreadyStarted,   // Story 8.3 AC-8: Cannot regenerate past week
    ConfirmationRequired, // Story 8.4 AC-2: Regenerate all requires confirmation
    BadRequest(String),   // Story 8.2: Invalid UUID format
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
            ApiError::WeekNotFound => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: "WeekNotFound".to_string(),
                    message: "Week not found or does not belong to you.".to_string(),
                    details: None,
                    action: None,
                },
            ),
            ApiError::Forbidden => (
                StatusCode::FORBIDDEN,
                ErrorResponse {
                    error: "Forbidden".to_string(),
                    message: "This week belongs to a different user.".to_string(),
                    details: None,
                    action: None,
                },
            ),
            ApiError::WeekLocked => (
                StatusCode::FORBIDDEN,
                ErrorResponse {
                    error: "WeekLocked".to_string(),
                    message: "Cannot regenerate current week. It is locked to prevent disrupting in-progress meals.".to_string(),
                    details: None,
                    action: None,
                },
            ),
            ApiError::WeekAlreadyStarted => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: "WeekAlreadyStarted".to_string(),
                    message: "Cannot regenerate a week that has already started.".to_string(),
                    details: None,
                    action: None,
                },
            ),
            ApiError::ConfirmationRequired => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: "ConfirmationRequired".to_string(),
                    message: "This action requires confirmation. Include { \"confirmation\": true } in request body.".to_string(),
                    details: None,
                    action: None,
                },
            ),
            ApiError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: "BadRequest".to_string(),
                    message: msg,
                    details: None,
                    action: None,
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

/// GET /plan/week/:week_id route handler (Story 8.2 AC-1)
///
/// Displays specific week meal plan details with shopping list for authenticated users.
///
/// # Authentication
/// Protected by JWT cookie middleware (AC-2)
///
/// # Authorization
/// Verifies week belongs to authenticated user (AC-3)
///
/// # Returns
/// - 200 OK: JSON with week data, meal assignments, shopping list, and navigation links (AC-6)
/// - 400 Bad Request: Invalid UUID format
/// - 403 Forbidden: Week belongs to different user (AC-8)
/// - 404 Not Found: Week not found (AC-7)
#[tracing::instrument(skip(state), fields(user_id = %auth.user_id, week_id = %week_id))]
pub async fn get_week_detail(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>, // AC-2: Extract user_id from JWT
    Path(week_id): Path<String>,      // AC-1: Extract week_id from path
) -> Result<Json<WeekDetailResponse>, ApiError> {
    let user_id = &auth.user_id;

    tracing::debug!(user_id = %user_id, week_id = %week_id, "Loading week detail");

    // Validate week_id is valid UUID format
    uuid::Uuid::parse_str(&week_id).map_err(|e| {
        tracing::warn!(
            user_id = %user_id,
            week_id = %week_id,
            error = %e,
            "Invalid UUID format for week_id"
        );
        ApiError::BadRequest("Invalid week_id format: must be a valid UUID".to_string())
    })?;

    // AC-4: Load week data from meal_plans table
    let week_row = sqlx::query(
        r#"
        SELECT
            id,
            user_id,
            start_date,
            end_date,
            status,
            is_locked
        FROM meal_plans
        WHERE id = ?1
        "#,
    )
    .bind(&week_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let week_data = week_row.ok_or_else(|| {
        tracing::warn!(
            user_id = %user_id,
            week_id = %week_id,
            "Week not found in database"
        );
        ApiError::WeekNotFound
    })?;

    // AC-3: Verify week belongs to authenticated user (authorization check)
    let week_user_id: String = week_data.try_get("user_id")?;
    if week_user_id != *user_id {
        tracing::warn!(
            user_id = %user_id,
            week_id = %week_id,
            week_user_id = %week_user_id,
            "Authorization failed: week belongs to different user"
        );
        return Err(ApiError::Forbidden);
    }

    // Parse week metadata
    let id: String = week_data.try_get("id")?;
    let start_date: String = week_data.try_get("start_date")?;
    let end_date: String = week_data.try_get("end_date")?;
    let status: String = week_data.try_get("status")?;
    let is_locked: bool = week_data.try_get("is_locked")?;

    tracing::debug!(
        user_id = %user_id,
        week_id = %week_id,
        status = %status,
        is_locked = is_locked,
        "Week metadata loaded successfully"
    );

    // AC-4: Load meal assignments for week with recipe details (JOIN query to avoid N+1)
    let meal_assignments = sqlx::query(
        r#"
        SELECT
            ma.id as assignment_id,
            ma.date,
            ma.course_type,
            ma.prep_required,
            ma.assignment_reasoning,
            ma.accompaniment_recipe_id,
            r.id as recipe_id,
            r.title as recipe_title,
            r.prep_time_min,
            r.cook_time_min,
            r.complexity,
            acc.id as acc_id,
            acc.title as acc_title,
            acc.accompaniment_category as acc_category
        FROM meal_assignments ma
        JOIN recipes r ON ma.recipe_id = r.id
        LEFT JOIN recipes acc ON ma.accompaniment_recipe_id = acc.id
        WHERE ma.meal_plan_id = ?1
        ORDER BY ma.date, ma.course_type
        "#,
    )
    .bind(&week_id)
    .fetch_all(&state.db_pool)
    .await?;

    let meal_assignment_data: Vec<MealAssignmentData> = meal_assignments
        .iter()
        .map(|row| {
            let assignment_id: String = row.try_get("assignment_id")?;
            let date: String = row.try_get("date")?;
            let course_type: String = row.try_get("course_type")?;
            let prep_required: bool = row.try_get("prep_required")?;
            let assignment_reasoning: Option<String> = row.try_get("assignment_reasoning")?;

            let recipe_id: String = row.try_get("recipe_id")?;
            let recipe_title: String = row.try_get("recipe_title")?;
            let prep_time_min: Option<i32> = row.try_get("prep_time_min")?;
            let cook_time_min: Option<i32> = row.try_get("cook_time_min")?;
            let complexity: Option<String> = row.try_get("complexity")?;

            // Parse accompaniment if present
            let accompaniment = if let Ok(Some(acc_id)) = row.try_get::<Option<String>, _>("acc_id")
            {
                let acc_title: String = row.try_get("acc_title")?;
                let acc_category: Option<String> = row.try_get("acc_category")?;

                Some(AccompanimentData {
                    id: acc_id,
                    title: acc_title,
                    category: acc_category.unwrap_or_else(|| "other".to_string()),
                })
            } else {
                None
            };

            Ok(MealAssignmentData {
                id: assignment_id,
                date,
                course_type,
                recipe: RecipeData {
                    id: recipe_id,
                    title: recipe_title,
                    prep_time_min: prep_time_min.map(|t| t as u32),
                    cook_time_min: cook_time_min.map(|t| t as u32),
                    complexity,
                },
                accompaniment,
                prep_required,
                algorithm_reasoning: assignment_reasoning,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    tracing::debug!(
        user_id = %user_id,
        week_id = %week_id,
        meal_count = meal_assignment_data.len(),
        "Meal assignments loaded successfully"
    );

    // AC-5: Load shopping list for week (use LEFT JOIN to handle weeks without shopping lists)
    let shopping_list_data = load_shopping_list_for_week(&week_id, &state.db_pool).await?;

    tracing::debug!(
        user_id = %user_id,
        week_id = %week_id,
        has_shopping_list = shopping_list_data.is_some(),
        "Shopping list loaded"
    );

    // Calculate previous_week_id and next_week_id from database
    let navigation = calculate_week_navigation(user_id, &start_date, &state.db_pool).await?;

    tracing::info!(
        user_id = %user_id,
        week_id = %week_id,
        status = %status,
        meal_count = meal_assignment_data.len(),
        "Week detail loaded successfully"
    );

    // AC-6: Build JSON response
    Ok(Json(WeekDetailResponse {
        week: WeekDetailData {
            id,
            start_date,
            end_date,
            status,
            is_locked,
            meal_assignments: meal_assignment_data,
            shopping_list: shopping_list_data,
        },
        navigation,
    }))
}

/// Load shopping list for week from database (Story 8.2 AC-5)
async fn load_shopping_list_for_week(
    week_id: &str,
    db_pool: &SqlitePool,
) -> Result<Option<ShoppingListData>, ApiError> {
    // Query shopping list by meal_plan_id
    let shopping_list_row = sqlx::query(
        r#"
        SELECT id
        FROM shopping_lists
        WHERE meal_plan_id = ?1
        "#,
    )
    .bind(week_id)
    .fetch_optional(db_pool)
    .await?;

    let shopping_list_id: String = match shopping_list_row {
        Some(row) => row.try_get("id")?,
        None => return Ok(None), // No shopping list for this week
    };

    // Query shopping list items with category grouping
    let items_rows = sqlx::query(
        r#"
        SELECT
            ingredient_name,
            quantity,
            unit,
            category
        FROM shopping_list_items
        WHERE shopping_list_id = ?1
        ORDER BY category, ingredient_name
        "#,
    )
    .bind(&shopping_list_id)
    .fetch_all(db_pool)
    .await?;

    // Group items by category
    let mut categories: std::collections::HashMap<String, Vec<ShoppingItemData>> =
        std::collections::HashMap::new();

    for row in items_rows {
        let ingredient_name: String = row.try_get("ingredient_name")?;
        let quantity: f64 = row.try_get("quantity")?;
        let unit: String = row.try_get("unit")?;
        let category: String = row.try_get("category")?;

        categories
            .entry(category.clone())
            .or_default()
            .push(ShoppingItemData {
                ingredient_name,
                quantity: quantity as f32,
                unit,
                from_recipe_ids: vec![], // TODO: Track recipe IDs in shopping list items (future enhancement)
            });
    }

    // Convert HashMap to Vec with standardized category order
    let category_order = [
        "Produce", "Dairy", "Meat", "Pantry", "Frozen", "Bakery", "Other",
    ];
    let mut category_data: Vec<ShoppingCategoryData> = category_order
        .iter()
        .filter_map(|&cat_name| {
            categories
                .remove(cat_name)
                .map(|items| ShoppingCategoryData {
                    name: cat_name.to_string(),
                    items,
                })
        })
        .collect();

    // Add any remaining categories not in standard order
    for (cat_name, items) in categories {
        category_data.push(ShoppingCategoryData {
            name: cat_name,
            items,
        });
    }

    Ok(Some(ShoppingListData {
        id: shopping_list_id,
        categories: category_data,
    }))
}

/// Calculate previous/next week navigation links (Story 8.2 AC-6)
async fn calculate_week_navigation(
    user_id: &str,
    current_start_date: &str,
    db_pool: &SqlitePool,
) -> Result<WeekNavigationData, ApiError> {
    // Query previous week (week with start_date < current_start_date, ORDER BY start_date DESC LIMIT 1)
    let previous_week_row = sqlx::query(
        r#"
        SELECT id
        FROM meal_plans
        WHERE user_id = ?1 AND start_date < ?2
        ORDER BY start_date DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(current_start_date)
    .fetch_optional(db_pool)
    .await?;

    let previous_week_id: Option<String> = previous_week_row
        .as_ref()
        .and_then(|row| row.try_get("id").ok());

    // Query next week (week with start_date > current_start_date, ORDER BY start_date ASC LIMIT 1)
    let next_week_row = sqlx::query(
        r#"
        SELECT id
        FROM meal_plans
        WHERE user_id = ?1 AND start_date > ?2
        ORDER BY start_date ASC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(current_start_date)
    .fetch_optional(db_pool)
    .await?;

    let next_week_id: Option<String> = next_week_row
        .as_ref()
        .and_then(|row| row.try_get("id").ok());

    Ok(WeekNavigationData {
        previous_week_id,
        next_week_id,
    })
}

/// JSON response for week regeneration (Story 8.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekRegenerationResponse {
    pub week: WeekData,
    pub message: String,
}

/// POST /plan/week/:week_id/regenerate route handler (Story 8.3 AC-1)
///
/// Regenerates a single future week's meal plan by calling the Epic 7 algorithm
/// with the current rotation state and emitting a SingleWeekRegenerated event.
///
/// # Authentication
/// Protected by JWT cookie middleware (AC-1)
///
/// # Authorization & Validation
/// - Verifies week belongs to authenticated user (AC-2)
/// - Checks week is not locked (is_locked == false) (AC-7)
/// - Checks week status is not "past" (AC-8)
/// - Checks week status is not "current" (AC-7)
///
/// # Returns
/// - 200 OK: JSON with regenerated week data (AC-1)
/// - 403 Forbidden: Week is locked or belongs to different user (AC-7)
/// - 400 Bad Request: Week already started (AC-8)
/// - 404 Not Found: Week not found
#[tracing::instrument(skip(state), fields(user_id = %auth.user_id, week_id = %week_id))]
pub async fn regenerate_week(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>, // AC-1: Extract user_id from JWT
    Path(week_id): Path<String>,      // AC-1: Extract week_id from path
) -> Result<Json<WeekRegenerationResponse>, ApiError> {
    let user_id = &auth.user_id;

    tracing::info!(user_id = %user_id, week_id = %week_id, "Week regeneration requested");

    // Validate week_id is valid UUID format
    uuid::Uuid::parse_str(&week_id).map_err(|e| {
        tracing::warn!(
            user_id = %user_id,
            week_id = %week_id,
            error = %e,
            "Invalid UUID format for week_id"
        );
        ApiError::BadRequest("Invalid week_id format: must be a valid UUID".to_string())
    })?;

    // AC-2: Load week from read model and verify ownership
    let week_row = sqlx::query(
        r#"
        SELECT
            id,
            user_id,
            start_date,
            end_date,
            status,
            is_locked,
            generation_batch_id
        FROM meal_plans
        WHERE id = ?1
        "#,
    )
    .bind(&week_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let week_data = week_row.ok_or_else(|| {
        tracing::warn!(
            user_id = %user_id,
            week_id = %week_id,
            "Week not found in database"
        );
        ApiError::WeekNotFound
    })?;

    // AC-2: Verify week belongs to authenticated user (authorization check)
    let week_user_id: String = week_data.try_get("user_id")?;
    if week_user_id != *user_id {
        tracing::warn!(
            user_id = %user_id,
            week_id = %week_id,
            week_user_id = %week_user_id,
            "Authorization failed: week belongs to different user"
        );
        return Err(ApiError::Forbidden);
    }

    // Parse week metadata
    let start_date: String = week_data.try_get("start_date")?;
    let end_date: String = week_data.try_get("end_date")?;
    let status: String = week_data.try_get("status")?;
    let is_locked: bool = week_data.try_get("is_locked")?;
    let generation_batch_id: String = week_data.try_get("generation_batch_id")?;

    tracing::debug!(
        user_id = %user_id,
        week_id = %week_id,
        status = %status,
        is_locked = is_locked,
        "Week metadata loaded for regeneration"
    );

    // AC-2, AC-7: Check if week is locked (current week cannot be regenerated)
    // Note: Database uses 'active'/'archived' status values (not 'current'/'future'/'past')
    // - 'active' = current OR future weeks (is_locked differentiates them)
    // - 'archived' = past weeks that have ended
    // - is_locked=true = current week (actively in use, cannot regenerate)
    // - is_locked=false + status='active' = future week (can regenerate)
    if is_locked {
        tracing::warn!(
            user_id = %user_id,
            week_id = %week_id,
            status = %status,
            is_locked = is_locked,
            "Cannot regenerate locked week (current week in progress)"
        );
        return Err(ApiError::WeekLocked);
    }

    // AC-2, AC-8: Check if week already started (archived/past week cannot be regenerated)
    // Archived weeks are past weeks that have already ended (end_date < today)
    if status == "archived" {
        tracing::warn!(
            user_id = %user_id,
            week_id = %week_id,
            status = %status,
            "Cannot regenerate archived week (week has already ended)"
        );
        return Err(ApiError::WeekAlreadyStarted);
    }

    // AC-3: Load current rotation state for meal plan batch
    let rotation_state_row = sqlx::query(
        r#"
        SELECT used_main_course_ids, used_appetizer_ids, used_dessert_ids,
               cuisine_usage_count, last_complex_meal_date
        FROM meal_plan_rotation_state
        WHERE generation_batch_id = ?1 AND user_id = ?2
        "#,
    )
    .bind(&generation_batch_id)
    .bind(user_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let mut rotation_state = match rotation_state_row {
        Some(row) => {
            // Parse individual columns to reconstruct RotationState
            // Note: Database schema stores rotation state in separate columns (migration 06_v0.8.sql)
            // rather than a single JSON blob for query performance and field-level updates
            let used_main_ids: String = row.try_get("used_main_course_ids")?;
            let used_app_ids: String = row.try_get("used_appetizer_ids")?;
            let used_dess_ids: String = row.try_get("used_dessert_ids")?;
            let cuisine_usage: String = row.try_get("cuisine_usage_count")?;
            let last_complex: Option<String> = row.try_get("last_complex_meal_date")?;

            let used_main_course_ids: Vec<String> =
                serde_json::from_str(&used_main_ids).unwrap_or_default();
            let used_appetizer_ids: Vec<String> =
                serde_json::from_str(&used_app_ids).unwrap_or_default();
            let used_dessert_ids: Vec<String> =
                serde_json::from_str(&used_dess_ids).unwrap_or_default();
            let cuisine_usage_count: std::collections::HashMap<recipe::Cuisine, u32> =
                serde_json::from_str(&cuisine_usage).unwrap_or_default();

            // Reconstruct RotationState from database columns
            // Note: cycle_number, cycle_started_at, used_recipe_ids, and total_favorite_count
            // are re-initialized rather than persisted because:
            // 1. cycle_number is recalculated based on main_course exhaustion during generation
            // 2. cycle_started_at is reset when a new cycle begins (tracked implicitly)
            // 3. used_recipe_ids (global set) is derived from the three typed lists
            // 4. total_favorite_count is a snapshot at generation time, not a persistent stat
            // The critical persisted fields are the typed used lists (main/app/dessert) which
            // drive the rotation algorithm's variety logic across weeks within a batch.
            meal_planning::rotation::RotationState {
                cycle_number: 1,
                cycle_started_at: chrono::Utc::now().to_rfc3339(),
                used_recipe_ids: std::collections::HashSet::new(),
                total_favorite_count: 0,
                used_main_course_ids,
                used_appetizer_ids,
                used_dessert_ids,
                cuisine_usage_count,
                last_complex_meal_date: last_complex,
            }
        }
        None => {
            tracing::warn!(
                user_id = %user_id,
                generation_batch_id = %generation_batch_id,
                "Rotation state not found, initializing new state"
            );
            meal_planning::rotation::RotationState::new()
        }
    };

    tracing::debug!(
        user_id = %user_id,
        week_id = %week_id,
        "Rotation state loaded successfully"
    );

    // Load user's favorite recipes
    let favorite_recipes = load_favorite_recipes(user_id, &state.db_pool).await?;

    tracing::debug!(
        user_id = %user_id,
        recipe_count = favorite_recipes.len(),
        "Loaded favorite recipes for regeneration"
    );

    // Validate minimum recipe count (need at least 7 favorites for 1 week)
    if favorite_recipes.len() < 7 {
        tracing::warn!(
            user_id = %user_id,
            count = favorite_recipes.len(),
            "Insufficient recipes for week regeneration"
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

    // Load user's meal planning preferences
    let preferences = load_user_preferences(user_id, &state.db_pool).await?;

    tracing::debug!(
        user_id = %user_id,
        "Loaded meal planning preferences for regeneration"
    );

    // AC-4: Call Epic 7 algorithm to generate single week
    tracing::info!(user_id = %user_id, week_id = %week_id, "Calling single week regeneration algorithm");

    // Parse week_start_date as NaiveDate
    let week_start_date = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| ApiError::InternalServerError(format!("Invalid date format: {}", e)))?;

    let regenerated_week = meal_planning::algorithm::generate_single_week(
        favorite_recipes.clone(),
        &preferences,
        &mut rotation_state,
        week_start_date,
    )
    .map_err(|e| {
        tracing::error!(user_id = %user_id, week_id = %week_id, error = %e, "Week regeneration algorithm failed");
        match e {
            meal_planning::MealPlanningError::InsufficientRecipes { minimum: _, current: _ } => {
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
        week_id = %week_id,
        meal_count = regenerated_week.meal_assignments.len(),
        "Week regeneration algorithm completed successfully"
    );

    // AC-5: Emit SingleWeekRegenerated evento event
    let event = meal_planning::events::SingleWeekRegenerated {
        week_id: week_id.clone(),
        week_start_date: start_date.clone(),
        meal_assignments: regenerated_week.meal_assignments.clone(),
        updated_rotation_state: rotation_state.clone(),
        regenerated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<meal_planning::MealPlanAggregate>()
        .data(&event)
        .map_err(|e| {
            tracing::error!(
                user_id = %user_id,
                week_id = %week_id,
                error = %e,
                "Failed to encode SingleWeekRegenerated event"
            );
            ApiError::InternalServerError(format!("Failed to encode event: {}", e))
        })?
        .metadata(&true)
        .map_err(|e| {
            tracing::error!(
                user_id = %user_id,
                week_id = %week_id,
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
                week_id = %week_id,
                error = %e,
                "Failed to commit SingleWeekRegenerated event to evento"
            );
            ApiError::InternalServerError(format!("Failed to commit event: {}", e))
        })?;

    tracing::info!(
        user_id = %user_id,
        week_id = %week_id,
        "SingleWeekRegenerated event emitted successfully"
    );

    // Build meal assignments for response
    let meal_assignments: Vec<MealAssignmentData> = regenerated_week
        .meal_assignments
        .iter()
        .map(|assignment| {
            // Find recipe details
            let recipe = favorite_recipes
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
                    favorite_recipes
                        .iter()
                        .find(|r| r.id == *acc_id)
                        .map(|acc_recipe| {
                            let category = acc_recipe
                                .accompaniment_category
                                .as_ref()
                                .map(|cat| match cat {
                                    recipe::AccompanimentCategory::Pasta => "pasta",
                                    recipe::AccompanimentCategory::Rice => "rice",
                                    recipe::AccompanimentCategory::Fries => "fries",
                                    recipe::AccompanimentCategory::Salad => "salad",
                                    recipe::AccompanimentCategory::Bread => "bread",
                                    recipe::AccompanimentCategory::Vegetable => "vegetable",
                                    recipe::AccompanimentCategory::Other => "other",
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

    // Build JSON response
    let response = WeekRegenerationResponse {
        week: WeekData {
            id: week_id.clone(),
            start_date,
            end_date,
            status,
            is_locked,
            meal_assignments,
            shopping_list_id: Some(regenerated_week.shopping_list_id),
        },
        message: "Week regenerated successfully. Shopping list updated.".to_string(),
    };

    tracing::info!(
        user_id = %user_id,
        week_id = %week_id,
        "Week regeneration completed successfully"
    );

    Ok(Json(response))
}

/// JSON request payload for regenerate all future weeks (Story 8.4 AC-2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegenerateAllPayload {
    pub confirmation: bool,
}

/// JSON response for regenerate all future weeks (Story 8.4 AC-8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegenerateAllResponse {
    pub regenerated_weeks: usize,
    pub preserved_current_week_id: Option<String>,
    pub first_future_week: Option<WeekData>,
    pub message: String,
}

/// POST /plan/regenerate-all-future route handler (Story 8.4 AC-1)
///
/// Regenerates all future weeks while preserving the current week by calling
/// the Epic 7 algorithm for each future week and emitting an AllFutureWeeksRegenerated event.
///
/// # Authentication
/// Protected by JWT cookie middleware (AC-1)
///
/// # Confirmation
/// Requires explicit confirmation parameter (AC-2) to prevent accidental regeneration
///
/// # Preservation Logic
/// - Identifies current week (is_locked == true OR status == "active") (AC-3)
/// - Current week remains unchanged (AC-3)
/// - Only future weeks (status == "active" AND is_locked == false) regenerated (AC-4)
/// - Rotation state reset but seeded with current week's recipes (AC-5)
///
/// # Returns
/// - 200 OK: JSON with count of regenerated weeks + first future week data (AC-8)
/// - 400 Bad Request: ConfirmationRequired error if confirmation != true (AC-2, AC-10)
/// - 400 Bad Request: InsufficientRecipes error if < 7 favorite recipes
/// - 500 Internal Server Error: AlgorithmTimeout or internal error
#[tracing::instrument(skip(state), fields(user_id = %auth.user_id))]
pub async fn regenerate_all_future_weeks(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>, // AC-1: Extract user_id from JWT
    Json(payload): Json<RegenerateAllPayload>, // AC-2: Confirmation parameter
) -> Result<Json<RegenerateAllResponse>, ApiError> {
    let user_id = &auth.user_id;

    tracing::info!(user_id = %user_id, "Regenerate all future weeks requested");

    // AC-2, AC-10: Validate confirmation parameter
    if !payload.confirmation {
        tracing::warn!(
            user_id = %user_id,
            "Confirmation required but not provided"
        );
        return Err(ApiError::ConfirmationRequired);
    }

    tracing::debug!(user_id = %user_id, "Confirmation validated");

    // AC-3, AC-4: Identify current week and future weeks
    let all_weeks_rows = sqlx::query(
        r#"
        SELECT
            id,
            start_date,
            end_date,
            status,
            is_locked,
            generation_batch_id
        FROM meal_plans
        WHERE user_id = ?1 AND status IN ('active', 'archived')
        ORDER BY start_date ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(&state.db_pool)
    .await?;

    tracing::debug!(
        user_id = %user_id,
        total_weeks = all_weeks_rows.len(),
        "Loaded all meal plan weeks"
    );

    // AC-3: Filter to identify current week (is_locked == true)
    // Note: Database uses 'active' status for both current and future weeks.
    // Current week is differentiated by is_locked=true flag.
    // Future weeks have status='active' AND is_locked=false.
    let current_week_data = all_weeks_rows.iter().find(|row| {
        let is_locked: bool = row.try_get("is_locked").unwrap_or(false);
        // Current week is the locked week (actively in use, cannot regenerate)
        is_locked
    });

    let current_week_id: Option<String> = current_week_data
        .as_ref()
        .and_then(|row| row.try_get("id").ok());

    // AC-4: Filter future weeks (status == "active" AND is_locked == false)
    let future_weeks_data: Vec<_> = all_weeks_rows
        .iter()
        .filter(|row| {
            let is_locked: bool = row.try_get("is_locked").unwrap_or(false);
            let status: String = match row.try_get("status") {
                Ok(s) => s,
                Err(_) => {
                    tracing::warn!("Week missing status field, skipping");
                    return false;
                }
            };
            // Future week: active status (not archived) AND not locked
            status == "active" && !is_locked
        })
        .collect();

    tracing::debug!(
        user_id = %user_id,
        current_week_id = ?current_week_id,
        future_weeks_count = future_weeks_data.len(),
        "Identified weeks for regeneration"
    );

    // Edge case: No future weeks to regenerate (AC-4)
    if future_weeks_data.is_empty() {
        tracing::info!(
            user_id = %user_id,
            "No future weeks to regenerate"
        );
        return Ok(Json(RegenerateAllResponse {
            regenerated_weeks: 0,
            preserved_current_week_id: current_week_id,
            first_future_week: None,
            message: "No future weeks to regenerate. Current week preserved.".to_string(),
        }));
    }

    // Load user's favorite recipes
    let favorite_recipes = load_favorite_recipes(user_id, &state.db_pool).await?;

    tracing::debug!(
        user_id = %user_id,
        recipe_count = favorite_recipes.len(),
        "Loaded favorite recipes for bulk regeneration"
    );

    // Validate minimum recipe count (at least 7 total)
    if favorite_recipes.len() < 7 {
        tracing::warn!(
            user_id = %user_id,
            count = favorite_recipes.len(),
            "Insufficient recipes for regeneration"
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

    // Load user's meal planning preferences
    let preferences = load_user_preferences(user_id, &state.db_pool).await?;

    tracing::debug!(
        user_id = %user_id,
        "Loaded meal planning preferences for regeneration"
    );

    // AC-5: Initialize rotation state for regeneration (reset with current week preservation)
    let mut rotation_state = meal_planning::rotation::RotationState::new();

    // AC-5: Seed rotation_state with current week's main course recipe IDs to prevent immediate repetition
    if let Some(current_week_row) = current_week_data {
        let current_week_id_str: String = current_week_row.try_get("id")?;

        // Load current week's main course recipes
        let current_main_courses = sqlx::query(
            r#"
            SELECT recipe_id
            FROM meal_assignments
            WHERE meal_plan_id = ?1 AND course_type = 'main_course'
            "#,
        )
        .bind(&current_week_id_str)
        .fetch_all(&state.db_pool)
        .await?;

        let current_recipe_ids: Vec<String> = current_main_courses
            .iter()
            .filter_map(|row| row.try_get("recipe_id").ok())
            .collect();

        tracing::debug!(
            user_id = %user_id,
            current_week_id = %current_week_id_str,
            current_recipes = current_recipe_ids.len(),
            "Seeding rotation state with current week's recipes"
        );

        // Seed rotation state with current recipes
        rotation_state.used_main_course_ids = current_recipe_ids;
    }

    // AC-6: Call Epic 7 algorithm to regenerate all future weeks
    tracing::info!(
        user_id = %user_id,
        future_weeks_count = future_weeks_data.len(),
        "Starting bulk regeneration for future weeks"
    );

    let mut regenerated_weeks: Vec<WeekMealPlanData> = Vec::new();

    for (week_idx, week_row) in future_weeks_data.iter().enumerate() {
        let week_id: String = week_row.try_get("id")?;
        let start_date: String = week_row.try_get("start_date")?;
        let end_date: String = week_row.try_get("end_date")?;

        tracing::debug!(
            user_id = %user_id,
            week_index = week_idx,
            week_id = %week_id,
            "Regenerating week"
        );

        // Parse week_start_date as NaiveDate
        let week_start_date = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
            .map_err(|e| ApiError::InternalServerError(format!("Invalid date format: {}", e)))?;

        // Call generate_single_week for this week
        let regenerated_week = meal_planning::algorithm::generate_single_week(
            favorite_recipes.clone(),
            &preferences,
            &mut rotation_state,
            week_start_date,
        )
        .map_err(|e| {
            tracing::error!(
                user_id = %user_id,
                week_id = %week_id,
                error = %e,
                "Week regeneration algorithm failed"
            );
            match e {
                meal_planning::MealPlanningError::InsufficientRecipes {
                    minimum: _,
                    current: _,
                } => {
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

        tracing::debug!(
            user_id = %user_id,
            week_index = week_idx,
            week_id = %week_id,
            meal_count = regenerated_week.meal_assignments.len(),
            "Week regenerated successfully"
        );

        // Collect regenerated week data
        regenerated_weeks.push(WeekMealPlanData {
            id: week_id.clone(),
            start_date: start_date.clone(),
            end_date: end_date.clone(),
            status: meal_planning::events::WeekStatus::Future, // Future weeks remain future
            is_locked: false,                                  // Future weeks unlocked
            meal_assignments: regenerated_week.meal_assignments.clone(),
            shopping_list_id: regenerated_week.shopping_list_id.clone(),
        });
    }

    tracing::info!(
        user_id = %user_id,
        regenerated_count = regenerated_weeks.len(),
        "All future weeks regenerated successfully"
    );

    // AC-6: Emit AllFutureWeeksRegenerated evento event
    let generation_batch_id = uuid::Uuid::new_v4().to_string();

    let event = meal_planning::events::AllFutureWeeksRegenerated {
        generation_batch_id: generation_batch_id.clone(),
        user_id: user_id.to_string(),
        weeks: regenerated_weeks.clone(),
        preserved_current_week_id: current_week_id.clone(),
        regenerated_at: Utc::now().to_rfc3339(),
    };

    evento::create::<meal_planning::MealPlanAggregate>()
        .data(&event)
        .map_err(|e| {
            tracing::error!(
                user_id = %user_id,
                error = %e,
                "Failed to encode AllFutureWeeksRegenerated event"
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
                "Failed to commit AllFutureWeeksRegenerated event to evento"
            );
            ApiError::InternalServerError(format!("Failed to commit event: {}", e))
        })?;

    tracing::info!(
        user_id = %user_id,
        generation_batch_id = %generation_batch_id,
        "AllFutureWeeksRegenerated event emitted successfully"
    );

    // AC-8: Build JSON response with count + first future week data
    let first_future_week: Option<WeekData> = if let Some(week_data) = regenerated_weeks.first() {
        // Build meal assignments for first future week
        let meal_assignments: Vec<MealAssignmentData> = week_data
            .meal_assignments
            .iter()
            .map(|assignment| {
                // Find recipe details
                let recipe = favorite_recipes
                    .iter()
                    .find(|r| r.id == assignment.recipe_id)
                    .ok_or_else(|| {
                        ApiError::InternalServerError(format!(
                            "Recipe {} not found",
                            assignment.recipe_id
                        ))
                    })?;

                // Build accompaniment data if present
                let accompaniment =
                    assignment
                        .accompaniment_recipe_id
                        .as_ref()
                        .and_then(|acc_id| {
                            favorite_recipes
                                .iter()
                                .find(|r| r.id == *acc_id)
                                .map(|acc_recipe| {
                                    let category = acc_recipe
                                        .accompaniment_category
                                        .as_ref()
                                        .map(|cat| match cat {
                                            recipe::AccompanimentCategory::Pasta => "pasta",
                                            recipe::AccompanimentCategory::Rice => "rice",
                                            recipe::AccompanimentCategory::Fries => "fries",
                                            recipe::AccompanimentCategory::Salad => "salad",
                                            recipe::AccompanimentCategory::Bread => "bread",
                                            recipe::AccompanimentCategory::Vegetable => "vegetable",
                                            recipe::AccompanimentCategory::Other => "other",
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

        Some(WeekData {
            id: week_data.id.clone(),
            start_date: week_data.start_date.clone(),
            end_date: week_data.end_date.clone(),
            status: "future".to_string(), // Future weeks always have "future" status
            is_locked: false,
            meal_assignments,
            shopping_list_id: Some(week_data.shopping_list_id.clone()),
        })
    } else {
        None
    };

    let response = RegenerateAllResponse {
        regenerated_weeks: regenerated_weeks.len(),
        preserved_current_week_id: current_week_id,
        first_future_week,
        message: format!(
            "All {} future weeks regenerated successfully. Current week preserved.",
            regenerated_weeks.len()
        ),
    };

    tracing::info!(
        user_id = %user_id,
        regenerated_count = regenerated_weeks.len(),
        "Regenerate all future weeks completed successfully"
    );

    Ok(Json(response))
}
