use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

use imkitchen_recipe::{RecipeRow, RecipeType};
use imkitchen_shared::Metadata;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{FORBIDDEN, ForbiddenTemplate, NOT_FOUND, Template, ToastErrorTemplate, filters},
};

#[derive(askama::Template)]
#[template(path = "partials/recipes-delete-modal.html")]
pub struct DeleteModalTemplate {
    pub id: String,
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-delete-button.html")]
pub struct DeleteButtonTemplate<'a> {
    pub id: &'a str,
    pub loading: bool,
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
    let recipe = crate::try_page_opt_response!(app.recipe_query.find(&id), template);

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

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn delete_action(
    template: Template,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let recipe = crate::try_response!(anyhow_opt:
        app.recipe_command.load_optional(&id),
        template,
        None::<DeleteButtonTemplate>
    );

    if recipe.item.deleted {
        return template.render(ToastErrorTemplate {
            message: NOT_FOUND,
            description: None,
            original: None,
        });
    }

    if recipe.item.user_id != user.id {
        return template.render(ToastErrorTemplate {
            message: FORBIDDEN,
            description: None,
            original: None,
        });
    }

    crate::try_response!(
        app.recipe_command
            .delete_with(recipe, &Metadata::by(user.id.to_owned())),
        template,
        None::<DeleteButtonTemplate>
    );

    template
        .render(DeleteButtonTemplate {
            id: &id,
            loading: true,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn delete_status(
    template: Template,
    State(app): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    match crate::try_response!(anyhow:
        app.recipe_query.find(&id),
        template,
        Some(DeleteButtonTemplate {
            id: &id,
            loading: false
        })
    ) {
        Some(_) => template
            .render(DeleteButtonTemplate {
                id: &id,
                loading: true,
            })
            .into_response(),
        _ => Redirect::to("/recipes").into_response(),
    }
}

pub async fn delete_modal(template: Template, Path((id,)): Path<(String,)>) -> impl IntoResponse {
    template.render(DeleteModalTemplate { id })
}
