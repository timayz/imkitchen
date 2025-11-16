use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use imkitchen_recipe::{CuisineType, RecipeType};
use imkitchen_shared::Metadata;
use strum::VariantArray;

use crate::{
    auth::AuthUser,
    query::{UserGlobalStats, query_recipe_detail_by_id},
    routes::AppState,
    template::{Template, filters},
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
    pub stat: UserGlobalStats,
}

impl Default for IndexTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: imkitchen_user::AuthUser::default(),
            stat: UserGlobalStats::default(),
        }
    }
}

pub async fn page(
    template: Template<IndexTemplate>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    template.render(IndexTemplate {
        user,
        ..Default::default()
    })
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
