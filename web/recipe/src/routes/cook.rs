use axum::{extract::State, response::IntoResponse};
use axum_extra::extract::Query;
use evento::cursor::{Args, ReadResult, Value};
use imkitchen_core::recipe::query::{
    user::{RecipesQuery, SortBy, UserViewList},
    user_stat::UserStatView,
};
use imkitchen_types::recipe::RecipeType;
use serde::Deserialize;
use std::str::FromStr;
use strum::VariantArray;

use imkitchen_web_shared::{
    AppState,
    auth::AuthUser,
    template::{NotFoundTemplate, Template, filters},
};

#[derive(askama::Template)]
#[template(path = "recipes-cook.html")]
pub struct CookTemplate {
    pub current_path: String,
    pub user: AuthUser,
    /// The cook's username — used for the header and to build filter / load-more URLs.
    pub username: String,
    pub stat: UserStatView,
    pub owner_description: String,
    pub recipes: ReadResult<UserViewList>,
    pub query: PageQuery,
}

impl Default for CookTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: AuthUser::default(),
            username: String::new(),
            stat: UserStatView::default(),
            owner_description: String::new(),
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
    pub search: Option<String>,
    pub sort_by: Option<SortBy>,
    pub view: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn page(
    template: Template,
    user: Option<AuthUser>,
    axum::extract::Path((username,)): axum::extract::Path<(String,)>,
    State(app): State<AppState>,
    Query(input): Query<PageQuery>,
) -> impl IntoResponse {
    let owner_id = match imkitchen_web_shared::try_page_response!(
        app.core.recipe.find_owner_id_by_name(&username),
        template
    ) {
        Some(id) => id,
        None => return template.render(NotFoundTemplate).into_response(),
    };

    // Public page: anonymous visitors get a demo "guest" identity and the page
    // renders in demo mode — links stay under /demo and actions funnel to
    // sign-up (mirrors the public recipe detail page).
    let is_anonymous = user.is_none();
    let user = user.unwrap_or_else(AuthUser::demo);
    let template = if is_anonymous {
        template.demo()
    } else {
        template
    };

    let stat = imkitchen_web_shared::try_page_response!(
        app.core.recipe.find_user_stat(&owner_id),
        template
    )
    .unwrap_or_default();

    let owner_profile = imkitchen_web_shared::try_page_response!(
        app.identity.user_profile.load(&owner_id),
        template
    );

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

    let recipes = imkitchen_web_shared::try_page_response!(
        app.core.recipe.filter_user(RecipesQuery {
            exclude_ids: None,
            user_id: Some(owner_id),
            recipe_type,
            is_shared: Some(true),
            has_thumbnail: None,
            dietary_restrictions: vec![],
            dietary_where_any: false,
            in_meal_plan: None,
            sort_by: input.sort_by.unwrap_or_default(),
            args: args.limit(20),
            search: input.search,
        }),
        template
    );

    template
        .render(CookTemplate {
            user,
            username,
            stat,
            owner_description: owner_profile.description,
            recipes,
            query,
            ..Default::default()
        })
        .into_response()
}
