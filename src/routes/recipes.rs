use askama::Template;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension, Form,
};
use recipe::{
    batch_import_recipes, copy_recipe, create_recipe, delete_recipe, favorite_recipe,
    list_shared_recipes, query_collections_by_user, query_collections_for_recipe,
    query_recipe_by_id, query_recipes_by_collection, query_recipes_by_user, share_recipe,
    update_recipe, update_recipe_tags, BatchImportRecipe, BatchImportRecipesCommand,
    CollectionReadModel, CopyRecipeCommand, CreateRecipeCommand, DeleteRecipeCommand,
    FavoriteRecipeCommand, Ingredient, InstructionStep, RecipeDiscoveryFilters, RecipeError,
    ShareRecipeCommand, UpdateRecipeCommand, UpdateRecipeTagsCommand,
};
use serde::Deserialize;
use sqlx::Row;

use crate::middleware::auth::Auth;
use crate::routes::auth::AppState;

#[derive(Template)]
#[template(path = "components/ingredient-row.html")]
struct IngredientRowTemplate;

#[derive(Template)]
#[template(path = "components/share-button.html")]
struct ShareButtonTemplate {
    recipe_id: String,
    is_shared: bool,
}

#[derive(Template)]
#[template(path = "pages/recipe-waiting.html")]
struct RecipeWaitingTemplate {
    recipe_id: String,
    user: Option<Auth>,
    current_path: String,
}

#[derive(Template)]
#[template(path = "components/instruction-row.html")]
struct InstructionRowTemplate {
    step_number: usize,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecipeForm {
    pub title: String,
    pub recipe_type: String, // AC-2: "appetizer", "main_course", or "dessert"
    pub ingredient_name: Vec<String>,
    pub ingredient_quantity: Vec<String>,
    pub ingredient_unit: Vec<String>,
    pub instruction_text: Vec<String>,
    pub instruction_timer: Vec<String>,
    pub prep_time_min: Option<String>,
    pub cook_time_min: Option<String>,
    pub advance_prep_hours: Option<String>,
    pub serving_size: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/recipe-form.html")]
pub struct RecipeFormTemplate {
    pub error: String,
    pub user: Option<Auth>,               // Some(Auth) for authenticated pages
    pub mode: String,                     // "create" or "edit"
    pub recipe: Option<RecipeDetailView>, // Pre-populated data for edit mode
    pub current_path: String,
}

/// Query parameters for calendar context (Story 3.5) and notification deep linking (Story 4.6)
#[derive(Debug, Deserialize)]
pub struct CalendarContext {
    pub from: Option<String>,
    pub meal_plan_id: Option<String>,
    pub assignment_id: Option<String>,
    pub kitchen_mode: Option<bool>,
    pub notification_id: Option<String>, // Story 4.6: Deep link from notification
}

#[derive(Template)]
#[template(path = "pages/recipe-detail.html")]
pub struct RecipeDetailTemplate {
    pub recipe: RecipeDetailView,
    pub is_owner: bool,
    pub user: Option<Auth>,
    pub all_collections: Vec<CollectionReadModel>,
    pub recipe_collections: Vec<CollectionReadModel>,
    pub is_from_calendar: bool,          // Story 3.5 AC-1, AC-5
    pub kitchen_mode: bool,              // Story 3.5 AC-6
    pub meal_plan_id: Option<String>,    // Story 3.5 AC-4 (Replace button context)
    pub assignment_id: Option<String>,   // Story 3.5 AC-4 (Replace button context)
    pub notification_id: Option<String>, // Story 4.6 AC-7 (Deep link from notification)
    pub highlight_prep: bool,            // Story 4.6 AC-7 (Highlight prep instructions)
    pub prep_status: Option<notifications::read_model::UserNotification>, // Story 4.9 AC-8
    pub current_path: String,
}

/// Recipe detail view model for template
#[derive(Debug, Clone)]
pub struct RecipeDetailView {
    pub id: String,
    pub title: String,
    pub recipe_type: String, // AC-2: Recipe type (appetizer, main_course, dessert)
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<InstructionStep>,
    pub prep_time_min: Option<u32>,
    pub cook_time_min: Option<u32>,
    pub advance_prep_hours: Option<u32>,
    pub serving_size: Option<u32>,
    pub is_favorite: bool,
    pub is_shared: bool,
    pub complexity: Option<String>,
    pub cuisine: Option<String>,
    pub dietary_tags: Vec<String>,
    pub created_at: String,
    pub creator_email: Option<String>, // AC-4: Recipe attribution (creator username)
    pub avg_rating: Option<f32>,       // Story 2.9 AC-4, AC-9: Average rating
    pub review_count: Option<i32>,     // Story 2.9 AC-4: Number of reviews
}

/// GET /recipes/new - Display recipe creation form
#[tracing::instrument(skip(auth))]
pub async fn get_recipe_form(Extension(auth): Extension<Auth>) -> impl IntoResponse {
    let template = RecipeFormTemplate {
        error: String::new(),
        user: Some(auth),
        mode: "create".to_string(),
        recipe: None,
        current_path: "/recipes/new".to_string(),
    };
    Html(template.render().unwrap())
}

/// POST /recipes - Handle recipe creation form submission
#[tracing::instrument(skip(state, auth), fields(user_id = %auth.user_id))]
pub async fn post_create_recipe(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    body: String,
) -> Response {
    // Parse URL-encoded form manually to handle array fields
    let form = match parse_recipe_form(&body) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("Failed to parse form: {:?}", e);
            let template = RecipeFormTemplate {
                error: format!("Failed to parse form data: {}", e),
                user: Some(auth),
                mode: "create".to_string(),
                recipe: None,
                current_path: "/recipes/new".to_string(),
            };
            return (StatusCode::BAD_REQUEST, Html(template.render().unwrap())).into_response();
        }
    };
    // Build ingredients from parallel arrays - validate quantities first
    let mut ingredients: Vec<Ingredient> = Vec::new();
    for ((name, quantity_str), unit) in form
        .ingredient_name
        .iter()
        .zip(&form.ingredient_quantity)
        .zip(&form.ingredient_unit)
    {
        let quantity = match quantity_str.parse::<f32>() {
            Ok(q) => q,
            Err(_) => {
                tracing::warn!(
                    ingredient_name = %name,
                    invalid_quantity = %quantity_str,
                    "Invalid ingredient quantity in form submission"
                );
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!(
                        "Invalid ingredient quantity '{}' for '{}'. Must be a valid number.",
                        quantity_str, name
                    ),
                )
                    .into_response();
            }
        };
        ingredients.push(Ingredient {
            name: name.clone(),
            quantity,
            unit: unit.clone(),
        });
    }

    // Build instructions from parallel arrays
    let instructions: Vec<InstructionStep> = form
        .instruction_text
        .iter()
        .zip(&form.instruction_timer)
        .enumerate()
        .map(|(index, (text, timer_str))| InstructionStep {
            step_number: (index + 1) as u32,
            instruction_text: text.clone(),
            timer_minutes: if timer_str.is_empty() {
                None
            } else {
                timer_str.parse::<u32>().ok()
            },
        })
        .collect();

    // Parse optional numeric fields (handle empty strings as None)
    let prep_time_min =
        form.prep_time_min
            .and_then(|s| if s.is_empty() { None } else { s.parse().ok() });
    let cook_time_min =
        form.cook_time_min
            .and_then(|s| if s.is_empty() { None } else { s.parse().ok() });
    let advance_prep_hours =
        form.advance_prep_hours
            .and_then(|s| if s.is_empty() { None } else { s.parse().ok() });
    let serving_size = form
        .serving_size
        .and_then(|s| if s.is_empty() { None } else { s.parse().ok() });

    // Create command
    let command = CreateRecipeCommand {
        title: form.title.clone(),
        recipe_type: form.recipe_type.clone(), // AC-2: Recipe type classification
        ingredients,
        instructions,
        prep_time_min,
        cook_time_min,
        advance_prep_hours,
        serving_size,
    };

    // Execute recipe creation (evento event sourcing)
    match create_recipe(
        command,
        &auth.user_id,
        &state.evento_executor,
        &state.db_pool,
    )
    .await
    {
        Ok(recipe_id) => {
            tracing::info!("Recipe created successfully: {}", recipe_id);
            // Redirect to waiting page to poll for read model sync
            // Returns 200 OK with ts-location header for client-side navigation
            (
                StatusCode::OK,
                [(
                    "ts-location",
                    format!("/recipes/{}/waiting", recipe_id).as_str(),
                )],
                (),
            )
                .into_response()
        }
        Err(RecipeError::RecipeLimitReached) => {
            tracing::warn!("User {} reached recipe limit", auth.user_id);
            let template = RecipeFormTemplate {
                error: "You've reached your recipe limit (10 recipes for free tier). Please upgrade to premium for unlimited recipes.".to_string(),
                user: Some(auth),
                mode: "create".to_string(),
                recipe: None,
                current_path: "/recipes/new".to_string(),
            };
            (StatusCode::OK, Html(template.render().unwrap())).into_response()
        }
        Err(RecipeError::ValidationError(msg)) => {
            tracing::warn!("Recipe validation failed: {}", msg);
            let template = RecipeFormTemplate {
                error: format!("Validation error: {}", msg),
                user: Some(auth),
                mode: "create".to_string(),
                recipe: None,
                current_path: "/recipes/new".to_string(),
            };
            (StatusCode::OK, Html(template.render().unwrap())).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create recipe: {:?}", e);
            let template = RecipeFormTemplate {
                error: "An error occurred while creating the recipe. Please try again.".to_string(),
                user: Some(auth),
                mode: "create".to_string(),
                recipe: None,
                current_path: "/recipes/new".to_string(),
            };
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(template.render().unwrap()),
            )
                .into_response()
        }
    }
}

/// GET /recipes/:id/waiting - Display waiting page with polling
/// Shown after recipe creation/copy to wait for read model synchronization
#[tracing::instrument(skip(auth), fields(recipe_id = %recipe_id))]
pub async fn get_recipe_waiting(
    Extension(auth): Extension<Auth>,
    Path(recipe_id): Path<String>,
) -> impl IntoResponse {
    let template = RecipeWaitingTemplate {
        recipe_id: recipe_id.clone(),
        user: Some(auth),
        current_path: format!("/recipes/{}/waiting", recipe_id),
    };
    Html(template.render().unwrap())
}

/// GET /recipes/:id/check - Check if recipe exists in read model
/// Used by waiting page to poll for recipe availability after creation/copy
/// Follows same pattern as register flow - returns HTML that continues polling
#[tracing::instrument(skip(state, auth), fields(recipe_id = %recipe_id))]
pub async fn check_recipe_exists(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path(recipe_id): Path<String>,
) -> Response {
    // Query recipe from read model
    match query_recipe_by_id(&recipe_id, &state.db_pool).await {
        Ok(Some(recipe_data)) => {
            // Check if user has permission to view this recipe
            if !recipe_data.is_shared && recipe_data.user_id != auth.user_id {
                // Recipe exists but user doesn't have access - return 404
                (StatusCode::NOT_FOUND, "Recipe not found").into_response()
            } else {
                // Recipe exists and user has access - redirect to recipe detail
                (
                    StatusCode::OK,
                    [("ts-location", format!("/recipes/{}", recipe_id).as_str())],
                    (),
                )
                    .into_response()
            }
        }
        Ok(None) => {
            // Recipe not yet in read model - return HTML that continues polling
            // TwinSpark requires at least one element to swap, so we return a div that continues polling
            let polling_html = format!(
                r#"<div ts-req="/recipes/{}/check" ts-trigger="load delay:500ms"></div>"#,
                recipe_id
            );
            (StatusCode::OK, Html(polling_html)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to query recipe: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to check recipe").into_response()
        }
    }
}

/// GET /recipes/:id - Display recipe detail page
/// Story 3.5: Added calendar context query parameters for navigation and kitchen mode
/// AC-10: Returns 404 for private recipes when accessed by non-owners
#[tracing::instrument(skip(state, auth), fields(recipe_id = %recipe_id))]
pub async fn get_recipe_detail(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path(recipe_id): Path<String>,
    Query(context): Query<CalendarContext>, // Story 3.5: Calendar context query params
) -> Response {
    // Query recipe from read model
    match query_recipe_by_id(&recipe_id, &state.db_pool).await {
        Ok(Some(recipe_data)) => {
            // AC-10: Privacy check - return 404 if recipe is private and user is not owner
            if !recipe_data.is_shared && recipe_data.user_id != auth.user_id {
                tracing::warn!(
                    recipe_id = %recipe_id,
                    requesting_user = %auth.user_id,
                    owner_user = %recipe_data.user_id,
                    "Non-owner attempted to access private recipe"
                );
                return (StatusCode::NOT_FOUND, "Recipe not found").into_response();
            }

            // Parse ingredients and instructions from JSON
            let ingredients: Vec<Ingredient> = match serde_json::from_str(&recipe_data.ingredients)
            {
                Ok(ing) => ing,
                Err(e) => {
                    tracing::error!("Failed to parse ingredients JSON: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to load recipe data",
                    )
                        .into_response();
                }
            };

            let instructions: Vec<InstructionStep> =
                match serde_json::from_str(&recipe_data.instructions) {
                    Ok(inst) => inst,
                    Err(e) => {
                        tracing::error!("Failed to parse instructions JSON: {:?}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to load recipe data",
                        )
                            .into_response();
                    }
                };

            // Parse dietary tags JSON array
            let dietary_tags = recipe_data
                .dietary_tags
                .as_ref()
                .and_then(|tags_json| serde_json::from_str::<Vec<String>>(tags_json).ok())
                .unwrap_or_default();

            let recipe_view = RecipeDetailView {
                id: recipe_data.id.clone(),
                title: recipe_data.title,
                recipe_type: recipe_data.recipe_type, // AC-2: Include recipe_type
                ingredients,
                instructions,
                prep_time_min: recipe_data.prep_time_min.map(|v| v as u32),
                cook_time_min: recipe_data.cook_time_min.map(|v| v as u32),
                advance_prep_hours: recipe_data.advance_prep_hours.map(|v| v as u32),
                serving_size: recipe_data.serving_size.map(|v| v as u32),
                is_favorite: recipe_data.is_favorite,
                is_shared: recipe_data.is_shared,
                complexity: recipe_data.complexity,
                cuisine: recipe_data.cuisine,
                dietary_tags,
                created_at: recipe_data.created_at,
                creator_email: None, // Not needed for owner's own recipe views
                avg_rating: None,    // Not needed for owner's recipe view
                review_count: None,  // Not needed for owner's recipe view
            };

            // Check if user is the owner
            let is_owner = recipe_data.user_id == auth.user_id;

            // Query collections for this recipe (only if owner)
            let (all_collections, recipe_collections) = if is_owner {
                let all = query_collections_by_user(&auth.user_id, &state.db_pool)
                    .await
                    .unwrap_or_default();
                let recipe_cols =
                    query_collections_for_recipe(&recipe_id, &auth.user_id, &state.db_pool)
                        .await
                        .unwrap_or_default();
                (all, recipe_cols)
            } else {
                (Vec::new(), Vec::new())
            };

            // Story 3.5: Parse calendar context from query params
            let is_from_calendar = context.from.as_deref() == Some("calendar");
            let kitchen_mode = context.kitchen_mode.unwrap_or(false);

            // Story 4.6: Parse notification context for prep highlighting
            let highlight_prep = context.notification_id.is_some();

            // Story 4.9 AC-8: Query prep status for this recipe
            let prep_status = notifications::read_model::get_prep_status_for_recipe(
                &state.db_pool,
                &auth.user_id,
                &recipe_id,
            )
            .await
            .unwrap_or(None);

            let template = RecipeDetailTemplate {
                recipe: recipe_view,
                is_owner,
                user: Some(auth),
                all_collections,
                recipe_collections,
                is_from_calendar,                     // Story 3.5 AC-1, AC-5
                kitchen_mode,                         // Story 3.5 AC-6
                meal_plan_id: context.meal_plan_id,   // Story 3.5 AC-4
                assignment_id: context.assignment_id, // Story 3.5 AC-4
                notification_id: context.notification_id.clone(), // Story 4.6 AC-7
                highlight_prep,                       // Story 4.6 AC-7
                prep_status,                          // Story 4.9 AC-8
                current_path: format!("/recipes/{}", recipe_id),
            };

            // Action Item 1: Proper error handling for template rendering
            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(e) => {
                    tracing::error!("Template render error: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render page").into_response()
                }
            }
        }
        Ok(None) => {
            tracing::warn!("Recipe not found: {}", recipe_id);
            (StatusCode::NOT_FOUND, "Recipe not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to query recipe: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load recipe").into_response()
        }
    }
}

/// GET /recipes/ingredient-row - Return HTML fragment for new ingredient row
#[tracing::instrument(skip(_auth))]
pub async fn get_ingredient_row(Extension(_auth): Extension<Auth>) -> impl IntoResponse {
    let template = IngredientRowTemplate;
    Html(template.render().unwrap())
}

/// GET /recipes/instruction-row - Return HTML fragment for new instruction row
/// Note: Step number will be updated client-side after insertion
#[tracing::instrument(skip(_auth))]
pub async fn get_instruction_row(Extension(_auth): Extension<Auth>) -> impl IntoResponse {
    let template = InstructionRowTemplate { step_number: 1 };
    Html(template.render().unwrap())
}

/// GET /recipes/:id/edit - Display recipe edit form
#[tracing::instrument(skip(state, auth), fields(recipe_id = %recipe_id, user_id = %auth.user_id))]
pub async fn get_recipe_edit_form(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path(recipe_id): Path<String>,
) -> Response {
    // Query recipe from read model
    match query_recipe_by_id(&recipe_id, &state.db_pool).await {
        Ok(Some(recipe_data)) => {
            // Check ownership: only the owner can edit
            if recipe_data.user_id != auth.user_id {
                tracing::warn!(
                    user_id = %auth.user_id,
                    recipe_id = %recipe_id,
                    owner_id = %recipe_data.user_id,
                    event = "ownership_violation",
                    action = "edit_recipe_form",
                    "User attempted to access edit form for recipe owned by another user"
                );
                return (
                    StatusCode::FORBIDDEN,
                    "You do not have permission to edit this recipe",
                )
                    .into_response();
            }

            // Parse ingredients and instructions from JSON
            let ingredients: Vec<Ingredient> = match serde_json::from_str(&recipe_data.ingredients)
            {
                Ok(ing) => ing,
                Err(e) => {
                    tracing::error!("Failed to parse ingredients JSON: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to load recipe data",
                    )
                        .into_response();
                }
            };

            let instructions: Vec<InstructionStep> =
                match serde_json::from_str(&recipe_data.instructions) {
                    Ok(inst) => inst,
                    Err(e) => {
                        tracing::error!("Failed to parse instructions JSON: {:?}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to load recipe data",
                        )
                            .into_response();
                    }
                };

            // Parse dietary tags JSON array
            let dietary_tags = recipe_data
                .dietary_tags
                .as_ref()
                .and_then(|tags_json| serde_json::from_str::<Vec<String>>(tags_json).ok())
                .unwrap_or_default();

            let recipe_view = RecipeDetailView {
                id: recipe_data.id.clone(),
                title: recipe_data.title,
                recipe_type: recipe_data.recipe_type, // AC-2: Include recipe_type
                ingredients,
                instructions,
                prep_time_min: recipe_data.prep_time_min.map(|v| v as u32),
                cook_time_min: recipe_data.cook_time_min.map(|v| v as u32),
                advance_prep_hours: recipe_data.advance_prep_hours.map(|v| v as u32),
                serving_size: recipe_data.serving_size.map(|v| v as u32),
                is_favorite: recipe_data.is_favorite,
                is_shared: recipe_data.is_shared,
                complexity: recipe_data.complexity,
                cuisine: recipe_data.cuisine,
                dietary_tags,
                created_at: recipe_data.created_at,
                creator_email: None, // Not needed for owner's own recipe views
                avg_rating: None,    // Not needed for owner's recipe view
                review_count: None,  // Not needed for owner's recipe view
            };

            let template = RecipeFormTemplate {
                error: String::new(),
                user: Some(auth),
                mode: "edit".to_string(),
                recipe: Some(recipe_view),
                current_path: format!("/recipes/{}/edit", recipe_id),
            };

            Html(template.render().unwrap()).into_response()
        }
        Ok(None) => {
            tracing::warn!("Recipe not found: {}", recipe_id);
            (StatusCode::NOT_FOUND, "Recipe not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to query recipe: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load recipe").into_response()
        }
    }
}

/// POST /recipes/:id - Handle recipe update form submission (TwinSpark pattern)
#[tracing::instrument(skip(state, auth), fields(recipe_id = %recipe_id, user_id = %auth.user_id))]
pub async fn post_update_recipe(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path(recipe_id): Path<String>,
    body: String,
) -> Response {
    // Parse URL-encoded form manually to handle array fields
    let form = match parse_recipe_form(&body) {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("Failed to parse form: {:?}", e);
            return (
                StatusCode::BAD_REQUEST,
                format!("Failed to parse form data: {}", e),
            )
                .into_response();
        }
    };

    // Build ingredients from parallel arrays - validate quantities first
    let mut ingredients: Vec<Ingredient> = Vec::new();
    for ((name, quantity_str), unit) in form
        .ingredient_name
        .iter()
        .zip(&form.ingredient_quantity)
        .zip(&form.ingredient_unit)
    {
        let quantity = match quantity_str.parse::<f32>() {
            Ok(q) => q,
            Err(_) => {
                tracing::warn!(
                    ingredient_name = %name,
                    invalid_quantity = %quantity_str,
                    "Invalid ingredient quantity in form submission"
                );
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!(
                        "Invalid ingredient quantity '{}' for '{}'. Must be a valid number.",
                        quantity_str, name
                    ),
                )
                    .into_response();
            }
        };
        ingredients.push(Ingredient {
            name: name.clone(),
            quantity,
            unit: unit.clone(),
        });
    }

    // Build instructions from parallel arrays
    let instructions: Vec<InstructionStep> = form
        .instruction_text
        .iter()
        .zip(&form.instruction_timer)
        .enumerate()
        .map(|(index, (text, timer_str))| InstructionStep {
            step_number: (index + 1) as u32,
            instruction_text: text.clone(),
            timer_minutes: if timer_str.is_empty() {
                None
            } else {
                timer_str.parse::<u32>().ok()
            },
        })
        .collect();

    // Parse optional numeric fields (handle empty strings as None)
    let prep_time_min = form
        .prep_time_min
        .map(|s| if s.is_empty() { None } else { s.parse().ok() });
    let cook_time_min = form
        .cook_time_min
        .map(|s| if s.is_empty() { None } else { s.parse().ok() });
    let advance_prep_hours =
        form.advance_prep_hours
            .map(|s| if s.is_empty() { None } else { s.parse().ok() });
    let serving_size = form
        .serving_size
        .map(|s| if s.is_empty() { None } else { s.parse().ok() });

    // Create update command
    let command = UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: auth.user_id.clone(),
        title: Some(form.title.clone()),
        recipe_type: Some(form.recipe_type.clone()), // AC-3: Allow updating recipe type
        ingredients: Some(ingredients),
        instructions: Some(instructions),
        prep_time_min,
        cook_time_min,
        advance_prep_hours,
        serving_size,
    };

    // Execute recipe update (evento event sourcing)
    match update_recipe(command, &state.evento_executor, &state.db_pool).await {
        Ok(()) => {
            tracing::info!("Recipe updated successfully: {}", recipe_id);

            // Handle share status toggle (AC-1, AC-2)
            // Check if is_shared checkbox was included in the form
            let is_shared = body.contains("is_shared=on");

            // Call share_recipe to emit RecipeShared event
            let share_command = ShareRecipeCommand {
                recipe_id: recipe_id.clone(),
                user_id: auth.user_id.clone(),
                shared: is_shared,
            };

            if let Err(e) =
                share_recipe(share_command, &state.evento_executor, &state.db_pool).await
            {
                tracing::error!(
                    recipe_id = %recipe_id,
                    error = ?e,
                    "Failed to update share status during recipe update"
                );
                // Continue - don't fail the whole update if share toggle fails
            }

            // Redirect to recipe detail page using TwinSpark (progressive enhancement)
            // Returns 200 OK for proper form swap, ts-location triggers client-side navigation
            (
                StatusCode::OK,
                [("ts-location", format!("/recipes/{}", recipe_id).as_str())],
                (),
            )
                .into_response()
        }
        Err(RecipeError::PermissionDenied) => {
            tracing::warn!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                event = "ownership_violation",
                action = "update_recipe",
                "User denied permission to update recipe - ownership check failed"
            );
            (
                StatusCode::FORBIDDEN,
                "You do not have permission to edit this recipe",
            )
                .into_response()
        }
        Err(RecipeError::NotFound) => {
            tracing::warn!("Recipe not found: {}", recipe_id);
            (StatusCode::NOT_FOUND, "Recipe not found").into_response()
        }
        Err(RecipeError::ValidationError(msg)) => {
            tracing::warn!("Recipe validation failed: {}", msg);
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Validation error: {}", msg),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update recipe: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while updating the recipe. Please try again.",
            )
                .into_response()
        }
    }
}

/// Parse URL-encoded form with array fields (ingredient_name[], etc.)
fn parse_recipe_form(body: &str) -> Result<CreateRecipeForm, String> {
    use std::collections::HashMap;

    let mut fields: HashMap<String, Vec<String>> = HashMap::new();

    // Parse URL-encoded body
    for pair in body.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            // First replace '+' with space, then decode percent-encoded characters
            let key_with_spaces = key.replace('+', " ");
            let value_with_spaces = value.replace('+', " ");

            let key = urlencoding::decode(&key_with_spaces).map_err(|e| e.to_string())?;
            let value = urlencoding::decode(&value_with_spaces).map_err(|e| e.to_string())?;

            fields
                .entry(key.to_string())
                .or_default()
                .push(value.to_string());
        }
    }

    // Extract fields
    let title = fields
        .get("title")
        .and_then(|v| v.first())
        .ok_or("Missing title")?
        .clone();

    let recipe_type = fields
        .get("recipe_type")
        .and_then(|v| v.first())
        .ok_or("Missing recipe_type")? // AC-2: recipe_type is required
        .clone();

    let ingredient_name = fields.get("ingredient_name[]").cloned().unwrap_or_default();
    let ingredient_quantity = fields
        .get("ingredient_quantity[]")
        .cloned()
        .unwrap_or_default();
    let ingredient_unit = fields.get("ingredient_unit[]").cloned().unwrap_or_default();
    let instruction_text = fields
        .get("instruction_text[]")
        .cloned()
        .unwrap_or_default();
    let instruction_timer = fields
        .get("instruction_timer[]")
        .cloned()
        .unwrap_or_default();

    let prep_time_min = fields.get("prep_time_min").and_then(|v| v.first()).cloned();
    let cook_time_min = fields.get("cook_time_min").and_then(|v| v.first()).cloned();
    let advance_prep_hours = fields
        .get("advance_prep_hours")
        .and_then(|v| v.first())
        .cloned();
    let serving_size = fields.get("serving_size").and_then(|v| v.first()).cloned();

    Ok(CreateRecipeForm {
        title,
        recipe_type, // AC-2: Include recipe_type in form
        ingredient_name,
        ingredient_quantity,
        ingredient_unit,
        instruction_text,
        instruction_timer,
        prep_time_min,
        cook_time_min,
        advance_prep_hours,
        serving_size,
    })
}

/// POST /recipes/:id/delete - Handle recipe deletion (TwinSpark pattern)
#[tracing::instrument(skip(state, auth), fields(recipe_id = %recipe_id, user_id = %auth.user_id))]
pub async fn post_delete_recipe(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path(recipe_id): Path<String>,
) -> Response {
    // Create delete command
    let command = DeleteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: auth.user_id.clone(),
    };

    // Execute recipe deletion (evento event sourcing)
    match delete_recipe(command, &state.evento_executor, &state.db_pool).await {
        Ok(()) => {
            tracing::info!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                event = "recipe_deleted",
                action = "delete_recipe",
                "Recipe successfully deleted"
            );
            // Redirect to recipes list using TwinSpark (progressive enhancement)
            // Returns 200 OK with ts-location header for client-side navigation
            (StatusCode::OK, [("ts-location", "/recipes")], ()).into_response()
        }
        Err(RecipeError::PermissionDenied) => {
            tracing::warn!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                event = "ownership_violation",
                action = "delete_recipe",
                "User denied permission to delete recipe - ownership check failed"
            );
            (
                StatusCode::FORBIDDEN,
                "You do not have permission to delete this recipe",
            )
                .into_response()
        }
        Err(RecipeError::NotFound) => {
            tracing::warn!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                event = "recipe_not_found",
                action = "delete_recipe",
                "Recipe not found for deletion"
            );
            (StatusCode::NOT_FOUND, "Recipe not found").into_response()
        }
        Err(e) => {
            tracing::error!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                error = ?e,
                "Failed to delete recipe"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while deleting the recipe. Please try again.",
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RecipeListQuery {
    pub collection: Option<String>,
    pub complexity: Option<String>,  // "simple", "moderate", "complex"
    pub cuisine: Option<String>,     // e.g., "Italian", "Asian"
    pub dietary: Option<String>,     // e.g., "vegetarian", "vegan", "gluten-free"
    pub favorite_only: Option<bool>, // Filter for favorited recipes only
    pub recipe_type: Option<String>, // "appetizer", "main_course", "dessert"
}

/// View model for recipe list with parsed ingredient/instruction counts
#[derive(Debug, Clone)]
pub struct RecipeListView {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub recipe_type: String,
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub advance_prep_hours: Option<i32>,
    pub serving_size: Option<i32>,
    pub is_favorite: bool,
    pub ingredient_count: usize,
    pub instruction_count: usize,
    pub complexity: Option<String>,
    pub cuisine: Option<String>,
    pub dietary_tags: Vec<String>,
    pub created_at: String,
}

#[derive(Template)]
#[template(path = "pages/recipe-list.html")]
pub struct RecipeListTemplate {
    pub recipes: Vec<RecipeListView>,
    pub collections: Vec<CollectionReadModel>,
    pub active_collection: Option<String>,
    pub complexity_filter: Option<String>,
    pub recipe_type_filter: Option<String>,
    pub favorite_only: bool,
    pub favorite_count: i64,
    pub user: Option<Auth>,
    pub current_path: String,
}

/// GET /recipes - Display recipe library with optional collection filter
#[tracing::instrument(skip(state, auth), fields(user_id = %auth.user_id))]
pub async fn get_recipe_list(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Query(query): Query<RecipeListQuery>,
) -> impl IntoResponse {
    // Query user's collections for sidebar
    let collections = query_collections_by_user(&auth.user_id, &state.db_pool)
        .await
        .unwrap_or_default();

    // Query recipes based on collection filter
    let recipes = if let Some(ref collection_id) = query.collection {
        // Filter by specific collection
        query_recipes_by_collection(collection_id, &state.db_pool)
            .await
            .unwrap_or_default()
    } else {
        // Show all user's recipes (with optional favorite filter)
        let favorite_only = query.favorite_only.unwrap_or(false);
        query_recipes_by_user(&auth.user_id, favorite_only, &state.db_pool)
            .await
            .unwrap_or_default()
    };

    // Convert RecipeReadModel to RecipeListView (parse JSON counts)
    let mut recipe_views: Vec<RecipeListView> = recipes
        .into_iter()
        .map(|r| {
            // Parse ingredient and instruction JSON arrays to get counts
            let ingredient_count = serde_json::from_str::<Vec<Ingredient>>(&r.ingredients)
                .map(|v| v.len())
                .unwrap_or(0);
            let instruction_count = serde_json::from_str::<Vec<InstructionStep>>(&r.instructions)
                .map(|v| v.len())
                .unwrap_or(0);

            // Parse dietary tags JSON array
            let dietary_tags = r
                .dietary_tags
                .as_ref()
                .and_then(|tags_json| serde_json::from_str::<Vec<String>>(tags_json).ok())
                .unwrap_or_default();

            RecipeListView {
                id: r.id,
                user_id: r.user_id,
                title: r.title,
                recipe_type: r.recipe_type,
                prep_time_min: r.prep_time_min,
                cook_time_min: r.cook_time_min,
                advance_prep_hours: r.advance_prep_hours,
                serving_size: r.serving_size,
                is_favorite: r.is_favorite,
                ingredient_count,
                instruction_count,
                complexity: r.complexity,
                cuisine: r.cuisine,
                dietary_tags,
                created_at: r.created_at,
            }
        })
        .collect();

    // Apply tag filters
    if let Some(ref complexity_filter) = query.complexity {
        recipe_views.retain(|r| {
            r.complexity
                .as_ref()
                .map(|c| c.eq_ignore_ascii_case(complexity_filter))
                .unwrap_or(false)
        });
    }

    if let Some(ref recipe_type_filter) = query.recipe_type {
        recipe_views.retain(|r| r.recipe_type.eq_ignore_ascii_case(recipe_type_filter));
    }

    if let Some(ref cuisine_filter) = query.cuisine {
        recipe_views.retain(|r| {
            r.cuisine
                .as_ref()
                .map(|c| c.eq_ignore_ascii_case(cuisine_filter))
                .unwrap_or(false)
        });
    }

    if let Some(ref dietary_filter) = query.dietary {
        recipe_views.retain(|r| {
            r.dietary_tags
                .iter()
                .any(|tag| tag.eq_ignore_ascii_case(dietary_filter))
        });
    }

    // Query favorite count from users table (O(1) query via subscription)
    let favorite_count =
        sqlx::query_scalar::<_, i32>("SELECT favorite_count FROM users WHERE id = ?1")
            .bind(&auth.user_id)
            .fetch_one(&state.db_pool)
            .await
            .unwrap_or(0);

    let favorite_only = query.favorite_only.unwrap_or(false);

    tracing::info!(
        user_id = %auth.user_id,
        recipe_count = recipe_views.len(),
        collection_filter = ?query.collection,
        favorite_only = favorite_only,
        "Fetched recipe list"
    );

    let template = RecipeListTemplate {
        recipes: recipe_views,
        collections,
        active_collection: query.collection,
        complexity_filter: query.complexity,
        recipe_type_filter: query.recipe_type,
        favorite_only,
        favorite_count: favorite_count as i64,
        user: Some(auth),
        current_path: "/recipes".to_string(),
    };

    Html(template.render().unwrap())
}

/// POST /recipes/:id/tags - Update recipe tags manually
#[tracing::instrument(skip(state, auth), fields(user_id = %auth.user_id, recipe_id = %recipe_id))]
pub async fn post_update_recipe_tags(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path(recipe_id): Path<String>,
    body: String,
) -> Response {
    // Parse form data manually (simple key=value pairs)
    use std::collections::HashMap;
    let mut form_data: HashMap<String, String> = HashMap::new();
    for pair in body.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_key = urlencoding::decode(key).unwrap_or_default().to_string();
            let decoded_value = urlencoding::decode(value).unwrap_or_default().to_string();
            form_data.insert(decoded_key, decoded_value);
        }
    }

    // Parse complexity (optional) - filter out empty strings
    let complexity = form_data
        .get("complexity")
        .cloned()
        .filter(|s| !s.is_empty());

    // Parse cuisine (optional) - filter out empty strings
    let cuisine = form_data.get("cuisine").cloned().filter(|s| !s.is_empty());

    // Parse dietary tags (checkbox array)
    let dietary_tags: Vec<String> = form_data
        .iter()
        .filter_map(|(key, value)| {
            if key.starts_with("dietary_") && value == "on" {
                Some(key.strip_prefix("dietary_").unwrap().to_string())
            } else {
                None
            }
        })
        .collect();

    let command = UpdateRecipeTagsCommand {
        recipe_id: recipe_id.clone(),
        user_id: auth.user_id.clone(),
        complexity,
        cuisine,
        dietary_tags,
    };

    match update_recipe_tags(command, &state.evento_executor, &state.db_pool).await {
        Ok(_) => {
            tracing::info!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                "Recipe tags updated manually"
            );

            // Return success message for TwinSpark to display
            (
                StatusCode::OK,
                Html(r#"<div class="bg-green-50 border border-green-200 text-green-800 px-4 py-3 rounded mb-4" role="alert">
                    ✓ Tags updated successfully!
                </div>"#),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(
                error = ?e,
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                "Failed to update recipe tags"
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to update tags: {:?}", e),
            )
                .into_response()
        }
    }
}

#[derive(Template)]
#[template(path = "components/favorite-icon.html")]
pub struct FavoriteIconTemplate {
    pub recipe_id: String,
    pub is_favorite: bool,
}

/// POST /recipes/:id/favorite - Toggle favorite status (TwinSpark pattern)
#[tracing::instrument(skip(state, auth), fields(recipe_id = %recipe_id, user_id = %auth.user_id))]
pub async fn post_favorite_recipe(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path(recipe_id): Path<String>,
) -> Response {
    let command = FavoriteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: auth.user_id.clone(),
    };

    match favorite_recipe(command, &state.evento_executor, &state.db_pool).await {
        Ok(new_favorited_status) => {
            tracing::info!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                favorited = new_favorited_status,
                "Recipe favorite status toggled"
            );

            let template = FavoriteIconTemplate {
                recipe_id,
                is_favorite: new_favorited_status,
            };

            Html(template.render().unwrap()).into_response()
        }
        Err(RecipeError::PermissionDenied) => {
            tracing::warn!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                "User denied permission to favorite recipe"
            );
            (
                StatusCode::FORBIDDEN,
                "You do not have permission to favorite this recipe",
            )
                .into_response()
        }
        Err(RecipeError::NotFound) => {
            tracing::warn!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                "Recipe not found for favorite toggle"
            );
            (StatusCode::NOT_FOUND, "Recipe not found").into_response()
        }
        Err(e) => {
            tracing::error!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                error = ?e,
                "Failed to toggle favorite status"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while updating favorite status. Please try again.",
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ShareRecipeForm {
    pub shared: String, // "true" or "false" from checkbox
}

/// POST /recipes/:id/share - Toggle share status
///
/// AC-2: Toggle changes privacy from "private" to "shared" (RecipeShared event)
/// AC-6: Owner can revert to private at any time (removes from community discovery)
#[tracing::instrument(skip(state, auth), fields(recipe_id = %recipe_id, user_id = %auth.user_id))]
pub async fn post_share_recipe(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path(recipe_id): Path<String>,
    axum::Form(form): axum::Form<ShareRecipeForm>,
) -> Response {
    // Parse shared boolean from form string
    let shared = form.shared == "true" || form.shared == "on";

    let command = ShareRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: auth.user_id.clone(),
        shared,
    };

    match share_recipe(command, &state.evento_executor, &state.db_pool).await {
        Ok(()) => {
            tracing::info!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                shared = shared,
                "Recipe share status toggled"
            );

            // Story GH-139: Return updated button HTML for TwinSpark swap using Askama template
            let share_button_template = ShareButtonTemplate {
                recipe_id: recipe_id.clone(),
                is_shared: shared,
            };

            let button_html = share_button_template.render().unwrap();

            // Add success message
            let success_message = if shared {
                r#"<div class="bg-green-50 border border-green-200 text-green-800 px-4 py-3 rounded mb-4 mt-2" role="alert">
                    ✓ Recipe shared with community!
                </div>"#
            } else {
                r#"<div class="bg-green-50 border border-green-200 text-green-800 px-4 py-3 rounded mb-4 mt-2" role="alert">
                    ✓ Recipe is now private
                </div>"#
            };

            let response_html = format!("{}{}", button_html, success_message);

            (StatusCode::OK, Html(response_html)).into_response()
        }
        Err(RecipeError::PermissionDenied) => {
            tracing::warn!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                "User denied permission to share recipe"
            );
            (
                StatusCode::FORBIDDEN,
                "You do not have permission to share this recipe",
            )
                .into_response()
        }
        Err(RecipeError::NotFound) => {
            tracing::warn!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                "Recipe not found"
            );
            (StatusCode::NOT_FOUND, "Recipe not found").into_response()
        }
        Err(e) => {
            tracing::error!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                error = ?e,
                "Failed to toggle share status"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while updating share status. Please try again.",
            )
                .into_response()
        }
    }
}

/// Query parameters for /discover route (AC-4, AC-5, AC-6, AC-7)
#[derive(Debug, Deserialize)]
pub struct DiscoveryQueryParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub cuisine: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub min_rating: Option<u8>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub max_prep_time: Option<u32>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub dietary: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub search: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub sort: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub page: Option<u32>,
}

/// Helper function to deserialize empty strings as None
fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    use serde::Deserialize;
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        T::from_str(&s).map(Some).map_err(serde::de::Error::custom)
    }
}

#[derive(Template)]
#[template(path = "pages/discover.html")]
pub struct DiscoverTemplate {
    pub recipes: Vec<RecipeDetailView>,
    pub user: Option<Auth>,
    pub filters: DiscoveryQueryParams,
    pub current_page: u32,
    pub has_next_page: bool,
    pub current_path: String,
}

/// GET /discover - Community discovery feed (AC-1 to AC-7, AC-12)
///
/// Public route showing all shared recipes (no authentication required for read-only)
/// AC-3: Filters by is_shared = TRUE AND deleted_at IS NULL
/// AC-4: Supports filtering by cuisine, rating, prep time, dietary
/// AC-5: Search by title or ingredients
/// AC-6: Sorting by rating, date, alphabetical
/// AC-7: Pagination (20 recipes per page)
#[tracing::instrument(skip(state))]
pub async fn get_discover(
    State(state): State<AppState>,
    user: Option<Extension<Auth>>,
    Query(params): Query<DiscoveryQueryParams>,
) -> impl IntoResponse {
    // If user is logged in and hasn't specified dietary filter, apply their profile preferences
    let dietary_filter = if let Some(ref auth) = user {
        if params.dietary.is_none() {
            // Load user's dietary restrictions from profile
            let user_dietary: Option<String> =
                sqlx::query_scalar("SELECT dietary_restrictions FROM users WHERE id = ?1")
                    .bind(&auth.user_id)
                    .fetch_optional(&state.db_pool)
                    .await
                    .ok()
                    .flatten();

            // Parse JSON array and join with commas for filter
            user_dietary.and_then(|json| {
                serde_json::from_str::<Vec<String>>(&json)
                    .ok()
                    .filter(|tags| !tags.is_empty())
                    .map(|tags| tags.join(","))
            })
        } else {
            params.dietary.clone()
        }
    } else {
        params.dietary.clone()
    };

    // Convert query params to RecipeDiscoveryFilters
    let filters = RecipeDiscoveryFilters {
        cuisine: params.cuisine.clone(),
        min_rating: params.min_rating,
        max_prep_time: params.max_prep_time,
        dietary: dietary_filter,
        search: params.search.clone(),
        sort: params.sort.clone(),
        page: params.page,
    };

    // Query shared recipes using read model function
    match list_shared_recipes(&state.db_pool, filters).await {
        Ok(recipes) => {
            // Convert RecipeReadModel to RecipeDetailView
            let mut recipe_views = Vec::new();
            for recipe in &recipes {
                // Parse ingredients and instructions JSON
                let ingredients: Vec<Ingredient> = match serde_json::from_str(&recipe.ingredients) {
                    Ok(ing) => ing,
                    Err(e) => {
                        tracing::error!("Failed to parse ingredients JSON: {:?}", e);
                        continue;
                    }
                };

                let instructions: Vec<InstructionStep> =
                    match serde_json::from_str(&recipe.instructions) {
                        Ok(inst) => inst,
                        Err(e) => {
                            tracing::error!("Failed to parse instructions JSON: {:?}", e);
                            continue;
                        }
                    };

                // Parse dietary tags JSON array
                let dietary_tags = recipe
                    .dietary_tags
                    .as_ref()
                    .and_then(|tags_json| serde_json::from_str::<Vec<String>>(tags_json).ok())
                    .unwrap_or_default();

                // Query creator email from users table
                let creator_email =
                    sqlx::query_scalar::<_, String>("SELECT email FROM users WHERE id = ?")
                        .bind(&recipe.user_id)
                        .fetch_optional(&state.db_pool)
                        .await
                        .ok()
                        .flatten();

                // Query rating statistics (Story 2.9 AC-4, AC-9)
                use recipe::query_rating_stats;
                let rating_stats = query_rating_stats(&recipe.id, &state.db_pool).await.ok();

                let (avg_rating, review_count) = if let Some(stats) = rating_stats {
                    if stats.review_count > 0 {
                        (Some(stats.avg_rating), Some(stats.review_count))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

                recipe_views.push(RecipeDetailView {
                    id: recipe.id.clone(),
                    title: recipe.title.clone(),
                    recipe_type: recipe.recipe_type.clone(), // AC-2: Include recipe_type
                    ingredients,
                    instructions,
                    prep_time_min: recipe.prep_time_min.map(|v| v as u32),
                    cook_time_min: recipe.cook_time_min.map(|v| v as u32),
                    advance_prep_hours: recipe.advance_prep_hours.map(|v| v as u32),
                    serving_size: recipe.serving_size.map(|v| v as u32),
                    is_favorite: recipe.is_favorite,
                    is_shared: recipe.is_shared,
                    complexity: recipe.complexity.clone(),
                    cuisine: recipe.cuisine.clone(),
                    dietary_tags,
                    created_at: recipe.created_at.clone(),
                    creator_email,
                    avg_rating,
                    review_count,
                });
            }

            tracing::info!(
                recipe_count = recipe_views.len(),
                page = params.page.unwrap_or(1),
                filters = ?params,
                "Fetched community discovery feed"
            );

            // Check if there are more pages
            let current_page = params.page.unwrap_or(1);
            let has_next_page = recipes.len() == 20; // If we got exactly 20, there might be more

            let template = DiscoverTemplate {
                recipes: recipe_views,
                user: user.map(|u| u.0),
                filters: params,
                current_page,
                has_next_page,
                current_path: "/discover".to_string(),
            };

            Html(template.render().unwrap()).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to query shared recipes: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load community recipes",
            )
                .into_response()
        }
    }
}

#[derive(Template)]
#[template(path = "pages/discover-detail.html")]
pub struct DiscoverDetailTemplate {
    pub recipe: RecipeDetailView,
    pub user: Option<Auth>,
    pub avg_rating: f32,
    pub review_count: i32,
    pub ratings: Vec<RatingDisplay>,
    pub user_rating: Option<RatingDisplay>,
    pub already_copied: bool, // Story 2.10 AC-10: User already has this recipe
    pub at_recipe_limit: bool, // Story 2.10 AC-11: Free user at 10 recipe limit
    pub current_path: String,
}

#[derive(Debug, Clone)]
pub struct RatingDisplay {
    pub user_id: String,
    pub username: String, // email or username
    pub stars: i32,
    pub review_text: Option<String>,
    pub created_at: String,
    pub is_own: bool, // true if this rating belongs to the logged-in user
}

/// GET /discover/:id - Community recipe detail view (AC-8, AC-9, AC-10, AC-14, AC-15)
///
/// Public route showing full recipe details with SEO optimization
/// AC-9: Full recipe with creator attribution
/// AC-10: Read-only for non-owners (no edit/delete buttons)
/// AC-14: SEO meta tags (Open Graph)
/// AC-15: Schema.org Recipe JSON-LD markup
#[tracing::instrument(skip(state))]
pub async fn get_discover_detail(
    State(state): State<AppState>,
    Path(recipe_id): Path<String>,
    user: Option<Extension<Auth>>,
) -> impl IntoResponse {
    // Query recipe from read model (AC-3: must be shared and not deleted)
    let recipe_query = sqlx::query(
        r#"
        SELECT r.id, r.user_id, r.title, r.recipe_type, r.ingredients, r.instructions,
               r.prep_time_min, r.cook_time_min, r.advance_prep_hours, r.serving_size,
               r.is_favorite, r.is_shared, r.complexity, r.cuisine, r.dietary_tags,
               r.created_at, r.updated_at, u.email as creator_email
        FROM recipes r
        LEFT JOIN users u ON r.user_id = u.id
        WHERE r.id = ? AND r.is_shared = 1 AND r.deleted_at IS NULL
        "#,
    )
    .bind(&recipe_id)
    .fetch_optional(&state.db_pool)
    .await;

    match recipe_query {
        Ok(Some(row)) => {
            // Parse ingredients and instructions JSON
            let ingredients_json: String = row.get("ingredients");
            let instructions_json: String = row.get("instructions");

            let ingredients: Vec<Ingredient> = match serde_json::from_str(&ingredients_json) {
                Ok(ing) => ing,
                Err(e) => {
                    tracing::error!("Failed to parse ingredients JSON: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to parse recipe data",
                    )
                        .into_response();
                }
            };

            let instructions: Vec<InstructionStep> = match serde_json::from_str(&instructions_json)
            {
                Ok(inst) => inst,
                Err(e) => {
                    tracing::error!("Failed to parse instructions JSON: {:?}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to parse recipe data",
                    )
                        .into_response();
                }
            };

            // Parse dietary tags JSON array
            let dietary_tags_json: Option<String> = row.get("dietary_tags");
            let dietary_tags = dietary_tags_json
                .as_ref()
                .and_then(|tags_json| serde_json::from_str::<Vec<String>>(tags_json).ok())
                .unwrap_or_default();

            let recipe = RecipeDetailView {
                id: row.get("id"),
                title: row.get("title"),
                recipe_type: row.get("recipe_type"), // AC-2: Include recipe_type
                ingredients,
                instructions,
                prep_time_min: row.get::<Option<i32>, _>("prep_time_min").map(|v| v as u32),
                cook_time_min: row.get::<Option<i32>, _>("cook_time_min").map(|v| v as u32),
                advance_prep_hours: row
                    .get::<Option<i32>, _>("advance_prep_hours")
                    .map(|v| v as u32),
                serving_size: row.get::<Option<i32>, _>("serving_size").map(|v| v as u32),
                is_favorite: row.get("is_favorite"),
                is_shared: row.get("is_shared"),
                complexity: row.get("complexity"),
                cuisine: row.get("cuisine"),
                dietary_tags,
                created_at: row.get("created_at"),
                creator_email: row.get("creator_email"), // AC-9: Recipe attribution
                avg_rating: None,                        // Populated later
                review_count: None,                      // Populated later
            };

            // Query rating statistics (Story 2.9 AC-4)
            use recipe::{query_rating_stats, query_recipe_ratings, query_user_rating};

            let rating_stats = query_rating_stats(&recipe_id, &state.db_pool)
                .await
                .unwrap_or(recipe::RatingStats {
                    avg_rating: 0.0,
                    review_count: 0,
                });

            // Query all ratings for display (Story 2.9 AC-5)
            let ratings_data = query_recipe_ratings(&recipe_id, &state.db_pool)
                .await
                .unwrap_or_default();

            // Get current user's rating if logged in
            let current_user_id = user.as_ref().map(|u| u.0.user_id.clone());
            let user_rating_data = if let Some(ref uid) = current_user_id {
                query_user_rating(&recipe_id, uid, &state.db_pool)
                    .await
                    .unwrap_or(None)
            } else {
                None
            };

            // Build rating displays with usernames (Story 2.9 AC-5)
            let mut ratings = Vec::new();
            for rating in ratings_data {
                // Query username/email for each rating
                let username_result = sqlx::query("SELECT email FROM users WHERE id = ?")
                    .bind(&rating.user_id)
                    .fetch_optional(&state.db_pool)
                    .await;

                let username = username_result
                    .ok()
                    .and_then(|row_opt| row_opt.map(|r| r.get::<String, _>("email")))
                    .unwrap_or_else(|| "Anonymous".to_string());

                let is_own = current_user_id
                    .as_ref()
                    .map(|uid| uid == &rating.user_id)
                    .unwrap_or(false);

                ratings.push(RatingDisplay {
                    user_id: rating.user_id.clone(),
                    username,
                    stars: rating.stars,
                    review_text: rating.review_text.clone(),
                    created_at: rating.created_at.clone(),
                    is_own,
                });
            }

            // Build user's own rating display if exists
            let user_rating = if let Some(rating) = user_rating_data {
                Some(RatingDisplay {
                    user_id: rating.user_id.clone(),
                    username: "You".to_string(),
                    stars: rating.stars,
                    review_text: rating.review_text.clone(),
                    created_at: rating.created_at.clone(),
                    is_own: true,
                })
            } else {
                None
            };

            // Story 2.10 AC-10, AC-11: Check if user already copied and if at recipe limit
            let (already_copied, at_recipe_limit) = if let Some(ref auth) = user {
                // AC-10: Check if user already copied this recipe
                let copy_check: Option<i64> = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM recipes WHERE user_id = ?1 AND original_recipe_id = ?2 AND deleted_at IS NULL"
                )
                .bind(&auth.user_id)
                .bind(&recipe_id)
                .fetch_optional(&state.db_pool)
                .await
                .unwrap_or(Some(0));

                let copied = copy_check.unwrap_or(0) > 0;

                // AC-11: Check if user is at recipe limit (free tier only)
                // First, get user tier
                let user_tier: Option<String> =
                    sqlx::query_scalar("SELECT tier FROM users WHERE id = ?1")
                        .bind(&auth.user_id)
                        .fetch_optional(&state.db_pool)
                        .await
                        .unwrap_or(None);

                let limit_reached = if user_tier.as_deref() != Some("premium") {
                    let private_recipe_count: i64 = sqlx::query_scalar(
                        "SELECT COUNT(*) FROM recipes WHERE user_id = ?1 AND is_shared = 0 AND deleted_at IS NULL"
                    )
                    .bind(&auth.user_id)
                    .fetch_one(&state.db_pool)
                    .await
                    .unwrap_or(0);

                    private_recipe_count >= 10
                } else {
                    false
                };

                (copied, limit_reached)
            } else {
                (false, false)
            };

            let template = DiscoverDetailTemplate {
                recipe,
                user: user.map(|u| u.0),
                avg_rating: rating_stats.avg_rating,
                review_count: rating_stats.review_count,
                ratings,
                user_rating,
                already_copied,
                at_recipe_limit,
                current_path: format!("/discover/{}", recipe_id),
            };

            Html(template.render().unwrap()).into_response()
        }
        Ok(None) => {
            // Recipe not found, not shared, or deleted (AC-8)
            (StatusCode::NOT_FOUND, "Recipe not found").into_response()
        }
        Err(e) => {
            tracing::error!("Failed to query recipe: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load recipe").into_response()
        }
    }
}

/// POST /discover/:id/add - Add community recipe to user's library (Story 2.10)
///
/// Authenticated route that copies a shared recipe to the user's personal library
/// AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-9, AC-10, AC-11, AC-12
#[tracing::instrument(skip(state, auth), fields(user_id = %auth.user_id, recipe_id = %recipe_id))]
pub async fn post_add_to_library(
    State(state): State<AppState>,
    Path(recipe_id): Path<String>,
    Extension(auth): Extension<Auth>,
) -> impl IntoResponse {
    // AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-10, AC-11: Use copy_recipe command
    let command = CopyRecipeCommand {
        original_recipe_id: recipe_id.clone(),
    };

    match copy_recipe(
        command,
        &auth.user_id,
        &state.evento_executor,
        &state.db_pool,
    )
    .await
    {
        Ok(new_recipe_id) => {
            tracing::info!(
                user_id = %auth.user_id,
                original_recipe_id = %recipe_id,
                new_recipe_id = %new_recipe_id,
                "Recipe copied to user's library"
            );

            // AC-12: Redirect to waiting page to poll for read model sync
            (
                StatusCode::SEE_OTHER,
                [("Location", format!("/recipes/{}/waiting", new_recipe_id))],
            )
                .into_response()
        }
        Err(RecipeError::RecipeLimitReached) => {
            // AC-11: Freemium limit enforcement
            tracing::warn!(
                user_id = %auth.user_id,
                original_recipe_id = %recipe_id,
                "Recipe limit reached when copying community recipe"
            );
            (
                StatusCode::OK,
                Html(format!(
                    r#"<div style="padding: 2rem; text-align: center;">
                        <h2 style="color: #dc2626; margin-bottom: 1rem;">Recipe Limit Reached</h2>
                        <p>You've reached your recipe limit (10 recipes for free tier).</p>
                        <p style="margin: 1rem 0;"><a href="/subscription" style="color: #2563eb;">Upgrade to premium</a> for unlimited recipes.</p>
                        <p><a href="/discover/{}" style="color: #2563eb;">Back to recipe</a></p>
                    </div>"#,
                    recipe_id
                )),
            )
                .into_response()
        }
        Err(RecipeError::AlreadyCopied) => {
            // AC-10: Prevent duplicate copies
            tracing::warn!(
                user_id = %auth.user_id,
                original_recipe_id = %recipe_id,
                "User already copied this recipe"
            );
            (
                StatusCode::OK,
                Html(format!(
                    r#"<div style="padding: 2rem; text-align: center;">
                        <h2 style="color: #f59e0b; margin-bottom: 1rem;">Already in Your Library</h2>
                        <p>You have already added this recipe to your library.</p>
                        <p style="margin-top: 1rem;"><a href="/discover/{}" style="color: #2563eb;">Back to recipe</a></p>
                    </div>"#,
                    recipe_id
                )),
            )
                .into_response()
        }
        Err(RecipeError::ValidationError(msg)) => {
            tracing::warn!(
                user_id = %auth.user_id,
                original_recipe_id = %recipe_id,
                error = %msg,
                "Validation error when copying recipe"
            );
            (
                StatusCode::OK,
                Html(format!(
                    r#"<div style="padding: 2rem; text-align: center;">
                        <h2 style="color: #dc2626; margin-bottom: 1rem;">Error</h2>
                        <p>{}</p>
                        <p style="margin-top: 1rem;"><a href="/discover/{}" style="color: #2563eb;">Back to recipe</a></p>
                    </div>"#,
                    msg, recipe_id
                )),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(
                user_id = %auth.user_id,
                original_recipe_id = %recipe_id,
                error = ?e,
                "Failed to copy recipe"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to add recipe to library",
            )
                .into_response()
        }
    }
}

/// Form data for rating submission
#[derive(Debug, Deserialize)]
pub struct RateRecipeForm {
    pub stars: i32,
    pub review_text: Option<String>,
}

/// POST /discover/:id/rate - Rate a community recipe (Story 2.9)
///
/// AC-1: Rating widget (1-5 stars) visible on shared recipe detail pages
/// AC-2: User can rate recipe only once per recipe_id (can update existing rating)
/// AC-3: Optional text review field with 500 character maximum
/// AC-10: Rating submission requires authentication
/// AC-11: Validation: rating must be integer between 1-5, review text <= 500 chars
/// AC-12: Duplicate rating prevention: UPDATE existing rating rather than INSERT new one
#[tracing::instrument(skip(state, auth), fields(user_id = %auth.user_id, recipe_id = %recipe_id))]
pub async fn post_rate_recipe(
    State(state): State<AppState>,
    Path(recipe_id): Path<String>,
    Extension(auth): Extension<Auth>,
    Form(form): Form<RateRecipeForm>,
) -> impl IntoResponse {
    use recipe::{rate_recipe, RateRecipeCommand};

    let command = RateRecipeCommand {
        recipe_id: recipe_id.clone(),
        stars: form.stars,
        review_text: form.review_text.clone(),
    };

    match rate_recipe(
        command,
        &auth.user_id,
        &state.evento_executor,
        &state.db_pool,
    )
    .await
    {
        Ok(_) => {
            tracing::info!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                stars = form.stars,
                "Recipe rated successfully"
            );

            (
                StatusCode::SEE_OTHER,
                [("Location", format!("/discover/{}", recipe_id))],
            )
                .into_response()
        }
        Err(RecipeError::ValidationError(msg)) => {
            tracing::warn!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                error = %msg,
                "Rating validation failed"
            );
            (StatusCode::UNPROCESSABLE_ENTITY, msg).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to rate recipe: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to submit rating").into_response()
        }
    }
}

/// Form data for rating update
#[derive(Debug, Deserialize)]
pub struct UpdateReviewForm {
    pub stars: i32,
    pub review_text: Option<String>,
}

/// POST /discover/:id/review/delete - Delete own review (Story 2.9)
///
/// AC-7: User can delete their own review via POST (creates a RatingDeleted event)
#[tracing::instrument(skip(state, auth), fields(user_id = %auth.user_id, recipe_id = %recipe_id))]
pub async fn post_delete_review(
    State(state): State<AppState>,
    Path(recipe_id): Path<String>,
    Extension(auth): Extension<Auth>,
) -> impl IntoResponse {
    use recipe::{delete_rating, DeleteRatingCommand};

    let command = DeleteRatingCommand {
        recipe_id: recipe_id.clone(),
    };

    match delete_rating(
        command,
        &auth.user_id,
        &state.evento_executor,
        &state.db_pool,
    )
    .await
    {
        Ok(_) => {
            tracing::info!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                "Review deleted successfully"
            );

            (
                StatusCode::SEE_OTHER,
                [("Location", format!("/discover/{}", recipe_id))],
            )
                .into_response()
        }
        Err(RecipeError::ValidationError(msg)) => {
            // AC-7: Returns 403 Forbidden if user attempts to delete another user's review
            tracing::warn!(
                user_id = %auth.user_id,
                recipe_id = %recipe_id,
                error = %msg,
                "Review deletion forbidden"
            );
            (StatusCode::FORBIDDEN, msg).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to delete review: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete review").into_response()
        }
    }
}

// ============================================================================
// Batch Import Routes (Story 2.12)
// ============================================================================

/// Template for batch import modal (AC-2, AC-3, AC-4)
#[derive(Template)]
#[template(path = "components/batch-import-modal.html")]
pub struct BatchImportModalTemplate {
    pub user: Option<Auth>,
}

/// Template for batch import results (AC-10)
#[derive(Template)]
#[template(path = "components/batch-import-results.html")]
pub struct BatchImportResultsTemplate {
    pub successful_count: usize,
    pub failed_count: usize,
    pub total_attempted: usize,
    pub failures: Vec<(usize, String)>,
    pub user: Option<Auth>,
}

/// GET /recipes/import-modal - Render batch import modal (AC-2)
#[tracing::instrument(skip(auth))]
pub async fn get_import_modal(Extension(auth): Extension<Auth>) -> impl IntoResponse {
    let template = BatchImportModalTemplate { user: Some(auth) };
    Html(template.render().unwrap())
}

/// POST /recipes/import - Handle batch recipe import (AC-4, AC-5, AC-6, AC-7, AC-8, AC-9, AC-10)
#[tracing::instrument(skip(state, auth, multipart), fields(user_id = %auth.user_id))]
pub async fn post_import_recipes(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    mut multipart: Multipart,
) -> Response {
    // AC-4: Extract file contents from multipart form data
    let mut file_content: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("recipes_file") {
            // Read file bytes
            match field.bytes().await {
                Ok(bytes) => {
                    // Convert bytes to string
                    match String::from_utf8(bytes.to_vec()) {
                        Ok(content) => {
                            file_content = Some(content);
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse file as UTF-8: {:?}", e);
                            return (
                                StatusCode::UNPROCESSABLE_ENTITY,
                                "File must be valid UTF-8 text",
                            )
                                .into_response();
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to read file bytes: {:?}", e);
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "Failed to read file contents",
                    )
                        .into_response();
                }
            }
        }
    }

    let file_content = match file_content {
        Some(content) => content,
        None => {
            return (StatusCode::BAD_REQUEST, "No file uploaded").into_response();
        }
    };

    // AC-5: Parse JSON to Vec<BatchImportRecipe>
    let recipes: Vec<BatchImportRecipe> = match serde_json::from_str(&file_content) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to parse JSON: {:?}", e);
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid JSON format. Please check your file syntax.",
            )
                .into_response();
        }
    };

    // AC-5: Validate root is array (not single object) - already handled by Vec deserialization
    // AC-5: Validate array is non-empty
    if recipes.is_empty() {
        return (StatusCode::UNPROCESSABLE_ENTITY, "No recipes found in file").into_response();
    }

    // Create batch import command
    let command = BatchImportRecipesCommand { recipes };

    // AC-6, AC-7, AC-8, AC-9, AC-12: Execute batch import
    match batch_import_recipes(
        command,
        &auth.user_id,
        &state.evento_executor,
        &state.db_pool,
    )
    .await
    {
        Ok(result) => {
            // AC-10: Render results modal with success/failure counts
            let successful_count = result.successful_recipe_ids.len();
            let failed_count = result.failed_imports.len();

            tracing::info!(
                user_id = %auth.user_id,
                successful = successful_count,
                failed = failed_count,
                "Batch import completed"
            );

            let template = BatchImportResultsTemplate {
                successful_count,
                failed_count,
                total_attempted: result.total_attempted,
                failures: result.failed_imports,
                user: Some(auth.clone()),
            };

            (StatusCode::OK, Html(template.render().unwrap())).into_response()
        }
        Err(RecipeError::RecipeLimitReached) => {
            // AC-8: Free tier limit exceeded
            tracing::warn!(
                user_id = %auth.user_id,
                "Batch import rejected: free tier limit exceeded"
            );
            (
                StatusCode::FORBIDDEN,
                "Import would exceed free tier limit (10 recipes maximum)",
            )
                .into_response()
        }
        Err(RecipeError::ValidationError(msg)) => {
            // AC-5: Validation error (empty array, etc.)
            tracing::warn!(
                user_id = %auth.user_id,
                error = %msg,
                "Batch import validation failed"
            );
            (StatusCode::UNPROCESSABLE_ENTITY, msg).into_response()
        }
        Err(e) => {
            // Database error or other unexpected error
            tracing::error!("Batch import failed: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server error during import. Please try again.",
            )
                .into_response()
        }
    }
}
