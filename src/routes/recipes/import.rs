use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use imkitchen_shared::recipe::{CuisineType, Ingredient, Instruction, RecipeType};
use serde::Deserialize;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{SERVER_ERROR_MESSAGE, Template, UpgradeModalTemplate, filters},
};

#[derive(Deserialize, Default, Clone)]
pub struct ErrorRecipe {
    pub name: String,
    pub error: String,
}

#[derive(Deserialize, Default, Clone)]
pub struct ImportJson {
    pub recipe_type: RecipeType,
    pub name: String,
    pub description: String,
    pub household_size: u16,
    pub prep_time: u16,
    pub cook_time: u16,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
    pub cuisine_type: CuisineType,
    pub advance_prep: Option<String>,
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-importing-status.html")]
pub struct ImportingStatusTemplate {
    pub id: Option<String>,
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-importing.html")]
pub struct ImportingTemplate {
    pub id: Option<String>,
    pub error_recipes: Vec<ErrorRecipe>,
}

#[derive(askama::Template)]
#[template(path = "recipes-import.html")]
pub struct ImportTemplate {
    pub current_path: String,
    pub user: AuthUser,
}

impl Default for ImportTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: Default::default(),
        }
    }
}

pub async fn page(template: Template, user: AuthUser) -> impl IntoResponse {
    template.render(ImportTemplate {
        user,
        ..Default::default()
    })
}

pub async fn action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Json(recipes): Json<Vec<ImportJson>>,
) -> impl IntoResponse {
    if !user.is_premium() {
        return ([("ts-swap", "skip")], template.render(UpgradeModalTemplate)).into_response();
    }

    let mut id = None;
    let mut error_recipes = vec![];

    for recipe in recipes {
        match imkitchen_recipe::Recipe::import(
            &app.executor,
            imkitchen_recipe::ImportInput {
                recipe_type: recipe.recipe_type,
                name: recipe.name.to_owned(),
                description: recipe.description,
                household_size: recipe.household_size,
                prep_time: recipe.prep_time,
                cook_time: recipe.cook_time,
                ingredients: recipe.ingredients,
                instructions: recipe.instructions,
                cuisine_type: recipe.cuisine_type,
                advance_prep: recipe.advance_prep.unwrap_or_default(),
            },
            &user.id,
            user.username.to_owned(),
        )
        .await
        {
            Ok(recipe_id) => {
                id = Some(recipe_id);
            }
            Err(imkitchen_shared::Error::Server(err)) => {
                tracing::error!(user = user.id, err = %err,"failed to import recipes");

                error_recipes.push(ErrorRecipe {
                    name: recipe.name,
                    error: SERVER_ERROR_MESSAGE.to_string(),
                });
            }
            Err(error) => {
                error_recipes.push(ErrorRecipe {
                    name: recipe.name,
                    error: error.to_string(),
                });
            }
        };
    }
    template.render(ImportingTemplate { id, error_recipes })
}

pub async fn status(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    match imkitchen_recipe::user::find(&app.read_db, &id).await {
        Ok(Some(_)) => template
            .render(ImportingStatusTemplate { id: None })
            .into_response(),
        Ok(_) => template
            .render(ImportingStatusTemplate { id: Some(id) })
            .into_response(),
        Err(err) => {
            tracing::error!(user = user.id,err = %err, "Failed to check recipe importation");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
