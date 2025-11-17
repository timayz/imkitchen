use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use evento::cursor::{Args, ReadResult, Value};
use imkitchen_recipe::{CuisineType, RecipeType};
use imkitchen_shared::Metadata;
use serde::Deserialize;
use strum::VariantArray;

use crate::{
    auth::AuthUser,
    query::{
        Recipe, RecipeSortBy, UserRecipeGlobalStats, query_recipe_detail_by_id, query_recipes,
        query_user_recipe_global_stats,
    },
    routes::AppState,
    template::{ServerErrorTemplate, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "recipes-create.html")]
pub struct CreateTemplate {
    pub id: String,
}

#[derive(askama::Template)]
#[template(path = "recipes-create-status.html")]
pub struct CreateStatusTemplate {
    pub id: String,
}

#[derive(askama::Template)]
#[template(path = "recipes-index.html")]
pub struct IndexTemplate {
    pub current_path: String,
    pub user: imkitchen_user::AuthUser,
    pub stat: UserRecipeGlobalStats,
    pub recipes: ReadResult<Recipe>,
}

impl Default for IndexTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: imkitchen_user::AuthUser::default(),
            stat: UserRecipeGlobalStats::default(),
            recipes: ReadResult::default(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct PageQuery {
    pub first: Option<u16>,
    pub after: Option<Value>,
    pub last: Option<u16>,
    pub before: Option<Value>,
    pub recipe_type: Option<String>,
    pub cuisine_type: Option<String>,
    pub sort_by: Option<RecipeSortBy>,
}

pub async fn page(
    template: Template<IndexTemplate>,
    server_error: Template<ServerErrorTemplate>,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    Query(input): Query<PageQuery>,
) -> impl IntoResponse {
    let stat = match query_user_recipe_global_stats(&app.pool, &user.id).await {
        Ok(s) => s,
        Err(err) => {
            tracing::error!(user = user.id, err = %err, "Failed to query user recipe global stats");

            return server_error.render(ServerErrorTemplate).into_response();
        }
    };
    let args = Args {
        first: input.first,
        after: input.after,
        last: input.last,
        before: input.before,
    };
    let recipe_type = input
        .recipe_type
        .and_then(|v| RecipeType::from_str(v.as_str()).ok());
    let cuisine_type = input
        .cuisine_type
        .and_then(|v| CuisineType::from_str(v.as_str()).ok());
    let recipes = match query_recipes(
        &app.pool,
        crate::query::RecipeInput {
            user_id: Some(user.id.to_owned()),
            recipe_type,
            cuisine_type,
            is_shared: None,
            sort_by: input.sort_by.unwrap_or_default(),
            args: args.limit(20),
        },
    )
    .await
    {
        Ok(recipes) => recipes,
        Err(err) => {
            tracing::error!(user = user.id, err = %err, "Failed to query user recipes");

            return server_error.render(ServerErrorTemplate).into_response();
        }
    };

    template
        .render(IndexTemplate {
            user,
            recipes,
            stat,
            ..Default::default()
        })
        .into_response()
}

pub async fn create(
    template: Template<CreateTemplate>,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    match app
        .recipe_command
        .create(Metadata::by(user.id.to_owned()))
        .await
    {
        Ok(id) => template.render(CreateTemplate { id }).into_response(),
        Err(err) => {
            tracing::error!(user_id = user.id, err = %err, "Failed to create recipe");

            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn create_status(
    template: Template<CreateStatusTemplate>,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    match query_recipe_detail_by_id(&app.pool, &id).await {
        Ok(Some(_)) => ([("ts-location", format!("/recipes/edit/{id}"))]).into_response(),
        Ok(_) => template.render(CreateStatusTemplate { id }).into_response(),
        Err(err) => {
            tracing::error!(user = user.id,err = %err, "Failed to check recipe creation");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
