use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use imkitchen_recipe::{CuisineType, RecipeType};

use crate::{
    auth::AuthUser,
    query::{RecipeDetail, query_recipe_detail_by_id},
    routes::AppState,
    template::{ForbiddenTemplate, NotFoundTemplate, ServerErrorTemplate, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "recipes-edit.html")]
pub struct EditTemplate {
    pub current_path: String,
    pub user: imkitchen_user::AuthUser,
    pub recipe: RecipeDetail,
}

impl Default for EditTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: imkitchen_user::AuthUser::default(),
            recipe: RecipeDetail::default(),
        }
    }
}

pub async fn page(
    template: Template<EditTemplate>,
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
        .render(EditTemplate {
            user,
            recipe,
            ..Default::default()
        })
        .into_response()
}
