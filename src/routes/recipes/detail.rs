use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

use imkitchen_recipe::{IngredientUnitFormat, RecipeRow, RecipeType};
use imkitchen_shared::Metadata;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{ForbiddenTemplate, Status, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "partials/set-username-modal.html")]
pub struct SetUsernameModalTemplate;

#[derive(askama::Template)]
#[template(path = "partials/recipes-delete-modal.html")]
pub struct DeleteModalTemplate {
    pub id: String,
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-share-button.html")]
pub struct ShareButtonTemplate<'a> {
    pub id: &'a str,
    pub is_shared: bool,
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-delete-button.html")]
pub struct DeleteButtonTemplate<'a> {
    pub id: &'a str,
    pub status: crate::template::Status,
}

#[derive(askama::Template)]
#[template(path = "recipes-detail.html")]
pub struct DetailTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub recipe: RecipeRow,
}

impl Default for DetailTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: AuthUser::default(),
            recipe: RecipeRow::default(),
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let recipe = crate::try_page_response!(opt: app.recipe_query.find(&id), template);

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
pub async fn share_to_community_action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let recipe = crate::try_response!(anyhow_opt:
        app.recipe_command.load_optional(&id),
        template
    );

    if recipe.item.deleted {
        crate::try_response!(sync: Ok(None::<()>), template);
    }

    if recipe.item.user_id != user.id {
        crate::try_response!(sync: Err(imkitchen_shared::Error::Forbidden), template);
    }

    if user.username.is_none() {
        return (
            [("ts-swap", "skip")],
            template.render(SetUsernameModalTemplate),
        )
            .into_response();
    }

    crate::try_response!(
        app.recipe_command
            .share_to_community_with(recipe, &Metadata::by(user.id.to_owned())),
        template
    );

    template
        .render(ShareButtonTemplate {
            id: &id,
            is_shared: true,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn make_private_action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let recipe = crate::try_response!(anyhow_opt:
        app.recipe_command.load_optional(&id),
        template
    );

    if recipe.item.deleted {
        crate::try_response!(sync: Ok(None::<()>), template);
    }

    if recipe.item.user_id != user.id {
        crate::try_response!(sync: Err(imkitchen_shared::Error::Forbidden), template);
    }

    crate::try_response!(
        app.recipe_command
            .make_private_with(recipe, &Metadata::by(user.id.to_owned())),
        template
    );

    template
        .render(ShareButtonTemplate {
            id: &id,
            is_shared: false,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn delete_action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let recipe = crate::try_response!(anyhow_opt:
        app.recipe_command.load_optional(&id),
        template
    );

    if recipe.item.deleted {
        crate::try_response!(sync: Ok(None::<()>), template);
    }

    if recipe.item.user_id != user.id {
        crate::try_response!(sync: Err(imkitchen_shared::Error::Forbidden), template);
    }

    crate::try_response!(
        app.recipe_command
            .delete_with(recipe, &Metadata::by(user.id.to_owned())),
        template
    );

    template
        .render(DeleteButtonTemplate {
            id: &id,
            status: Status::Pending,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn delete_status(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    match crate::try_response!(anyhow:
        app.recipe_query.find(&id),
        template,
        Some(DeleteButtonTemplate {
            id: &id,
            status: Status::Idle,
        })
    ) {
        Some(_) => template
            .render(DeleteButtonTemplate {
                id: &id,
                status: Status::Checking,
            })
            .into_response(),
        _ => Redirect::to("/recipes").into_response(),
    }
}

pub async fn delete_modal(template: Template, Path((id,)): Path<(String,)>) -> impl IntoResponse {
    template.render(DeleteModalTemplate { id })
}
