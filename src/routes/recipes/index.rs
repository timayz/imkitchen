use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
};
use evento::cursor::{Args, ReadResult, Value};
use imkitchen_recipe::{CuisineType, RecipeListRow, RecipeType, RecipesQuery, SortBy, UserStat};
use imkitchen_shared::Metadata;
use serde::Deserialize;
use std::str::FromStr;
use strum::VariantArray;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{Status, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "recipes-index.html")]
pub struct IndexTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub stat: UserStat,
    pub recipes: ReadResult<RecipeListRow>,
    pub query: PageQuery,
}

impl Default for IndexTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: AuthUser::default(),
            stat: UserStat::default(),
            recipes: ReadResult::default(),
            query: Default::default(),
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct PageQuery {
    pub first: Option<u16>,
    pub after: Option<Value>,
    pub last: Option<u16>,
    pub before: Option<Value>,
    pub recipe_type: Option<String>,
    pub cuisine_type: Option<String>,
    pub sort_by: Option<SortBy>,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Query(input): Query<PageQuery>,
) -> impl IntoResponse {
    let stat = crate::try_page_response!(app.recipe_query.find_user_stat(&user.id), template)
        .unwrap_or_default();

    let query = input.clone();

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

    let recipes = crate::try_page_response!(
        app.recipe_query.filter(RecipesQuery {
            user_id: Some(user.id.to_owned()),
            recipe_type,
            cuisine_type,
            is_shared: None,
            sort_by: input.sort_by.unwrap_or_default(),
            args: args.limit(20),
        }),
        template
    );

    template
        .render(IndexTemplate {
            user,
            recipes,
            stat,
            query,
            ..Default::default()
        })
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-create-button.html")]
pub struct CreateButtonTemplate<'a> {
    pub id: &'a str,
    pub status: Status,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn create(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let id = crate::try_response!(
        app.recipe_command.create(&Metadata::by(user.id.to_owned())),
        template
    );

    template
        .render(CreateButtonTemplate {
            id: &id,
            status: Status::Pending,
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn create_status(
    template: Template,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
    State(app): State<AppState>,
) -> impl IntoResponse {
    match crate::try_response!(anyhow:
        app.recipe_query.find(&id),
        template,
        Some(CreateButtonTemplate { id: &id, status: Status::Idle })
    ) {
        Some(_) => Redirect::to(&format!("/recipes/{id}/edit")).into_response(),
        _ => template
            .render(CreateButtonTemplate {
                id: &id,
                status: Status::Checking,
            })
            .into_response(),
    }
}
