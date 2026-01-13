use axum::{extract::State, response::IntoResponse};
use axum_extra::extract::Query;
use evento::cursor::{Args, ReadResult, Value};
use imkitchen_recipe::{
    user::{RecipesQuery, SortBy, UserViewList},
    user_stat::UserStatView,
};
use imkitchen_shared::recipe::{CuisineType, DietaryRestriction, RecipeType};
use serde::Deserialize;
use std::str::FromStr;
use strum::VariantArray;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "recipes-community.html")]
pub struct CommunityTemplate {
    pub current_path: String,
    pub recipes_path: String,
    pub user: AuthUser,
    pub stat: UserStatView,
    pub recipes: ReadResult<UserViewList>,
    pub query: PageQuery,
}

impl Default for CommunityTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            recipes_path: "community".to_owned(),
            user: AuthUser::default(),
            stat: UserStatView::default(),
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
    #[serde(default)]
    pub dietary_restrictions: Vec<DietaryRestriction>,
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
        app.recipe_query.filter_user(RecipesQuery {
            exclude_ids: None,
            user_id: None,
            recipe_type,
            cuisine_type,
            is_shared: Some(true),
            dietary_restrictions: input.dietary_restrictions,
            dietary_where_any: false,
            sort_by: input.sort_by.unwrap_or_default(),
            args: args.limit(20),
        }),
        template
    );

    template
        .render(CommunityTemplate {
            user,
            recipes,
            stat,
            query,
            ..Default::default()
        })
        .into_response()
}
