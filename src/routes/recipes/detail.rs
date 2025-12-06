use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

use imkitchen_recipe::{RecipeRow, RecipeType};
use imkitchen_shared::Metadata;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{FORBIDDEN, ForbiddenTemplate, NOT_FOUND, SERVER_ERROR_MESSAGE, Template, filters},
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
    pub recipe: RecipeRow,
}

impl Default for DetailTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: imkitchen_user::AuthUser::default(),
            recipe: RecipeRow::default(),
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let recipe = crate::try_anyhow_opt_response!(app.recipe_query.find(&id), template);

    if recipe.user_id != user.id {
        return template.render(ForbiddenTemplate).into_response();
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
    template: Template,
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
    template: Template,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    match app.recipe_query.find(&id).await {
        Ok(Some(_)) => template.render(DeleteStatusTemplate { id }).into_response(),
        Ok(_) => Redirect::to("/recipes").into_response(),
        Err(err) => {
            tracing::error!(recipe = id, user = user.id, err = %err,"Failed to check recipe delete status");

            Redirect::to("/recipes").into_response()
        }
    }
}

pub async fn delete_modal(template: Template, Path((id,)): Path<(String,)>) -> impl IntoResponse {
    template.render(DeleteModalTemplate { id })
}
