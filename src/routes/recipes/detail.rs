use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

use imkitchen_recipe::RecipeType;
use imkitchen_shared::Metadata;

use crate::{
    auth::AuthUser,
    query::{RecipeDetail, query_recipe_detail_by_id},
    routes::AppState,
    template::{
        FORBIDDEN, ForbiddenTemplate, NOT_FOUND, NotFoundTemplate, SERVER_ERROR_MESSAGE,
        ServerErrorTemplate, Template, filters,
    },
};

#[derive(askama::Template)]
#[template(path = "recipes-delete-modal.html")]
pub struct DeleteModalTemplate {
    pub id: String,
}

#[derive(askama::Template)]
#[template(path = "recipes-delete.html")]
pub struct DeleteTemplate {
    pub id: String,
    pub error_message: Option<String>,
}

#[derive(askama::Template)]
#[template(path = "recipes-delete-status.html")]
pub struct DeleteStatusTemplate {
    pub id: String,
}

#[derive(askama::Template)]
#[template(path = "recipes-detail.html")]
pub struct DetailTemplate {
    pub current_path: String,
    pub user: imkitchen_user::AuthUser,
    pub recipe: RecipeDetail,
}

impl Default for DetailTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: imkitchen_user::AuthUser::default(),
            recipe: RecipeDetail::default(),
        }
    }
}

pub async fn page(
    template: Template<DetailTemplate>,
    server_error: Template<ServerErrorTemplate>,
    not_found_error: Template<NotFoundTemplate>,
    forbidden_error: Template<ForbiddenTemplate>,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let recipe = match query_recipe_detail_by_id(&app.pool, &id).await {
        Ok(Some(r)) => r,
        Ok(_) => return not_found_error.render(NotFoundTemplate).into_response(),
        Err(err) => {
            tracing::error!(recipe = id, user = user.id, err = %err,"Failed to get recipe");

            return server_error.render(ServerErrorTemplate).into_response();
        }
    };

    if recipe.user_id != user.id {
        return forbidden_error.render(ForbiddenTemplate).into_response();
    }

    template
        .render(DetailTemplate {
            user,
            recipe,
            ..Default::default()
        })
        .into_response()
}

pub async fn delete_action(
    template: Template<DeleteTemplate>,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let recipe = match app.recipe_command.load_optional(&id).await {
        Ok(Some(r)) => r,
        Ok(_) => {
            return template.render(DeleteTemplate {
                id,
                error_message: Some(NOT_FOUND.to_owned()),
            });
        }
        Err(err) => {
            tracing::error!(recipe = id, user = user.id, err = %err,"Failed to delete recipe");

            return template.render(DeleteTemplate {
                id,
                error_message: Some(SERVER_ERROR_MESSAGE.to_owned()),
            });
        }
    };

    if recipe.item.deleted {
        return template.render(DeleteTemplate {
            id,
            error_message: Some(NOT_FOUND.to_owned()),
        });
    }

    if recipe.item.user_id != user.id {
        return template.render(DeleteTemplate {
            id,
            error_message: Some(FORBIDDEN.to_owned()),
        });
    }

    match app
        .recipe_command
        .delete_with(recipe, &Metadata::by(user.id.to_owned()))
        .await
    {
        Ok(_) => template.render(DeleteTemplate {
            id,
            error_message: None,
        }),
        Err(err) => {
            tracing::error!(recipe = id, user = user.id, err = %err, "Failed to delete recipe");

            template.render(DeleteTemplate {
                id,
                error_message: Some(SERVER_ERROR_MESSAGE.to_owned()),
            })
        }
    }
}

pub async fn delete_status(
    template: Template<DeleteStatusTemplate>,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    match query_recipe_detail_by_id(&app.pool, &id).await {
        Ok(Some(_)) => template.render(DeleteStatusTemplate { id }).into_response(),
        Ok(_) => Redirect::to("/recipes").into_response(),
        Err(err) => {
            tracing::error!(recipe = id, user = user.id, err = %err,"Failed to check recipe delete status");

            Redirect::to("/recipes").into_response()
        }
    }
}

pub async fn delete_modal(
    template: Template<DeleteModalTemplate>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    template.render(DeleteModalTemplate { id })
}
