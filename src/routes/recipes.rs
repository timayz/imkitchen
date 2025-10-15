use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use recipe::{
    create_recipe, delete_recipe, query_collections_by_user, query_collections_for_recipe,
    query_recipe_by_id, query_recipes_by_collection, query_recipes_by_user, update_recipe,
    CollectionReadModel, CreateRecipeCommand, DeleteRecipeCommand, Ingredient, InstructionStep,
    RecipeError, UpdateRecipeCommand,
};
use serde::Deserialize;

use crate::middleware::auth::Auth;
use crate::routes::auth::AppState;

#[derive(Template)]
#[template(path = "components/ingredient-row.html")]
struct IngredientRowTemplate;

#[derive(Template)]
#[template(path = "components/instruction-row.html")]
struct InstructionRowTemplate {
    step_number: usize,
}

#[derive(Debug, Deserialize)]
pub struct CreateRecipeForm {
    pub title: String,
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
}

#[derive(Template)]
#[template(path = "pages/recipe-detail.html")]
pub struct RecipeDetailTemplate {
    pub recipe: RecipeDetailView,
    pub is_owner: bool,
    pub user: Option<Auth>,
    pub all_collections: Vec<CollectionReadModel>,
    pub recipe_collections: Vec<CollectionReadModel>,
}

/// Recipe detail view model for template
#[derive(Debug, Clone)]
pub struct RecipeDetailView {
    pub id: String,
    pub title: String,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<InstructionStep>,
    pub prep_time_min: Option<u32>,
    pub cook_time_min: Option<u32>,
    pub advance_prep_hours: Option<u32>,
    pub serving_size: Option<u32>,
    pub is_favorite: bool,
    pub created_at: String,
}

/// GET /recipes/new - Display recipe creation form
#[tracing::instrument(skip(auth))]
pub async fn get_recipe_form(Extension(auth): Extension<Auth>) -> impl IntoResponse {
    let template = RecipeFormTemplate {
        error: String::new(),
        user: Some(auth),
        mode: "create".to_string(),
        recipe: None,
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
            // Redirect to recipe detail page using TwinSpark (progressive enhancement)
            // Returns 200 OK with ts-location header for client-side navigation
            (
                StatusCode::OK,
                [("ts-location", format!("/recipes/{}", recipe_id).as_str())],
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
            };
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Html(template.render().unwrap()),
            )
                .into_response()
        }
        Err(RecipeError::ValidationError(msg)) => {
            tracing::warn!("Recipe validation failed: {}", msg);
            let template = RecipeFormTemplate {
                error: format!("Validation error: {}", msg),
                user: Some(auth),
                mode: "create".to_string(),
                recipe: None,
            };
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Html(template.render().unwrap()),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create recipe: {:?}", e);
            let template = RecipeFormTemplate {
                error: "An error occurred while creating the recipe. Please try again.".to_string(),
                user: Some(auth),
                mode: "create".to_string(),
                recipe: None,
            };
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(template.render().unwrap()),
            )
                .into_response()
        }
    }
}

/// GET /recipes/:id - Display recipe detail page
#[tracing::instrument(skip(state, auth), fields(recipe_id = %recipe_id))]
pub async fn get_recipe_detail(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>,
    Path(recipe_id): Path<String>,
) -> Response {
    // Query recipe from read model
    match query_recipe_by_id(&recipe_id, &state.db_pool).await {
        Ok(Some(recipe_data)) => {
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

            let recipe_view = RecipeDetailView {
                id: recipe_data.id.clone(),
                title: recipe_data.title,
                ingredients,
                instructions,
                prep_time_min: recipe_data.prep_time_min.map(|v| v as u32),
                cook_time_min: recipe_data.cook_time_min.map(|v| v as u32),
                advance_prep_hours: recipe_data.advance_prep_hours.map(|v| v as u32),
                serving_size: recipe_data.serving_size.map(|v| v as u32),
                is_favorite: recipe_data.is_favorite,
                created_at: recipe_data.created_at,
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

            let template = RecipeDetailTemplate {
                recipe: recipe_view,
                is_owner,
                user: Some(auth),
                all_collections,
                recipe_collections,
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

            let recipe_view = RecipeDetailView {
                id: recipe_data.id.clone(),
                title: recipe_data.title,
                ingredients,
                instructions,
                prep_time_min: recipe_data.prep_time_min.map(|v| v as u32),
                cook_time_min: recipe_data.cook_time_min.map(|v| v as u32),
                advance_prep_hours: recipe_data.advance_prep_hours.map(|v| v as u32),
                serving_size: recipe_data.serving_size.map(|v| v as u32),
                is_favorite: recipe_data.is_favorite,
                created_at: recipe_data.created_at,
            };

            let template = RecipeFormTemplate {
                error: String::new(),
                user: Some(auth),
                mode: "edit".to_string(),
                recipe: Some(recipe_view),
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
            let key = urlencoding::decode(key).map_err(|e| e.to_string())?;
            let value = urlencoding::decode(value).map_err(|e| e.to_string())?;

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
}

/// View model for recipe list with parsed ingredient/instruction counts
#[derive(Debug, Clone)]
pub struct RecipeListView {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub advance_prep_hours: Option<i32>,
    pub serving_size: Option<i32>,
    pub is_favorite: bool,
    pub ingredient_count: usize,
    pub instruction_count: usize,
    pub created_at: String,
}

#[derive(Template)]
#[template(path = "pages/recipe-list.html")]
pub struct RecipeListTemplate {
    pub recipes: Vec<RecipeListView>,
    pub collections: Vec<CollectionReadModel>,
    pub active_collection: Option<String>,
    pub user: Option<Auth>,
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
        // Show all user's recipes
        query_recipes_by_user(&auth.user_id, &state.db_pool)
            .await
            .unwrap_or_default()
    };

    // Convert RecipeReadModel to RecipeListView (parse JSON counts)
    let recipe_views: Vec<RecipeListView> = recipes
        .into_iter()
        .map(|r| {
            // Parse ingredient and instruction JSON arrays to get counts
            let ingredient_count = serde_json::from_str::<Vec<Ingredient>>(&r.ingredients)
                .map(|v| v.len())
                .unwrap_or(0);
            let instruction_count = serde_json::from_str::<Vec<InstructionStep>>(&r.instructions)
                .map(|v| v.len())
                .unwrap_or(0);

            RecipeListView {
                id: r.id,
                user_id: r.user_id,
                title: r.title,
                prep_time_min: r.prep_time_min,
                cook_time_min: r.cook_time_min,
                advance_prep_hours: r.advance_prep_hours,
                serving_size: r.serving_size,
                is_favorite: r.is_favorite,
                ingredient_count,
                instruction_count,
                created_at: r.created_at,
            }
        })
        .collect();

    tracing::info!(
        user_id = %auth.user_id,
        recipe_count = recipe_views.len(),
        collection_filter = ?query.collection,
        "Fetched recipe list"
    );

    let template = RecipeListTemplate {
        recipes: recipe_views,
        collections,
        active_collection: query.collection,
        user: Some(auth),
    };

    Html(template.render().unwrap())
}
