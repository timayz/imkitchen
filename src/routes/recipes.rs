use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Extension,
};
use recipe::{
    create_recipe, query_recipe_by_id, CreateRecipeCommand, Ingredient, InstructionStep,
    RecipeError,
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
    pub user: Option<Auth>, // Some(Auth) for authenticated pages
}

#[derive(Template)]
#[template(path = "pages/recipe-detail.html")]
pub struct RecipeDetailTemplate {
    pub recipe: RecipeDetailView,
    pub is_owner: bool,
    pub user: Option<Auth>,
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
            };
            return (StatusCode::BAD_REQUEST, Html(template.render().unwrap())).into_response();
        }
    };
    // Build ingredients from parallel arrays
    let ingredients: Vec<Ingredient> = form
        .ingredient_name
        .iter()
        .zip(&form.ingredient_quantity)
        .zip(&form.ingredient_unit)
        .map(|((name, quantity_str), unit)| Ingredient {
            name: name.clone(),
            quantity: quantity_str.parse::<f32>().unwrap_or(0.0),
            unit: unit.clone(),
        })
        .collect();

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
            // Redirect to recipe detail page (PRG pattern)
            Redirect::to(&format!("/recipes/{}", recipe_id)).into_response()
        }
        Err(RecipeError::RecipeLimitReached) => {
            tracing::warn!("User {} reached recipe limit", auth.user_id);
            let template = RecipeFormTemplate {
                error: "You've reached your recipe limit (10 recipes for free tier). Please upgrade to premium for unlimited recipes.".to_string(),
                user: Some(auth),
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

            let template = RecipeDetailTemplate {
                recipe: recipe_view,
                is_owner,
                user: Some(auth),
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
