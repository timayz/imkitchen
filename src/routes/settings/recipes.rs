use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use evento::cursor::{Args, ReadResult, Value};
use imkitchen_core::recipe::query::user::{RecipesQuery, SortBy, UserViewList};
use imkitchen_shared::recipe::{CuisineType, RecipeType};
use serde::Deserialize;
use std::str::FromStr;
use strum::VariantArray;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "settings-recipes.html")]
pub struct RecipesTemplate {
    pub current_path: String,
    pub settings_path: String,
    pub user: AuthUser,
    pub recipes: ReadResult<UserViewList>,
    pub query: PageQuery,
}

impl Default for RecipesTemplate {
    fn default() -> Self {
        Self {
            current_path: "settings".to_owned(),
            settings_path: "recipes".to_owned(),
            user: AuthUser::default(),
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
    pub search: Option<String>,
    pub sort_by: Option<SortBy>,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Query(input): Query<PageQuery>,
) -> impl IntoResponse {
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
        app.core.recipe.filter_user(RecipesQuery {
            exclude_ids: None,
            user_id: Some(user.id.to_owned()),
            recipe_type,
            cuisine_type,
            is_shared: None,
            dietary_restrictions: vec![],
            dietary_where_any: false,
            sort_by: input.sort_by.unwrap_or_default(),
            args: args.limit(20),
            search: input.search,
        }),
        template
    );

    template
        .render(RecipesTemplate {
            user,
            recipes,
            query,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn create(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let id = match crate::try_response!(anyhow: app.core.recipe.find_user_draft(&user.id), template)
    {
        Some(id) => id,
        _ => crate::try_response!(
            app.core.recipe.create(&user.id, user.username.to_owned()),
            template
        ),
    };

    Redirect::to(&format!("/recipes/{id}/edit")).into_response()
}
