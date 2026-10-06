use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::Query;
use evento::cursor::{ReadResult, Value};
use imkitchen_core::recipe::query::user::{SortBy, UserViewList};
use imkitchen_types::recipe::RecipeType;
use serde::Deserialize;
use std::str::FromStr;
use strum::VariantArray;

use imkitchen_web_shared::{
    AppState,
    auth::{AuthUser, RequireChef},
    services::recipe::{self, BrowseQuery, ShareError},
    template::{Template, filters},
};

use super::detail::SetUsernameModalTemplate;

#[derive(askama::Template)]
#[template(path = "recipes-index.html")]
pub struct IndexTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub recipes: ReadResult<UserViewList>,
    pub query: PageQuery,
    pub has_shared: bool,
}

impl Default for IndexTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: AuthUser::default(),
            recipes: ReadResult::default(),
            query: Default::default(),
            has_shared: false,
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
    pub in_meal_plan: Option<bool>,
    pub mine: Option<bool>,
    pub no_image: Option<bool>,
    pub view: Option<String>,
}

impl From<PageQuery> for BrowseQuery {
    fn from(q: PageQuery) -> Self {
        Self {
            first: q.first,
            after: q.after,
            last: q.last,
            before: q.before,
            recipe_type: q
                .recipe_type
                .and_then(|v| RecipeType::from_str(v.as_str()).ok()),
            search: q.search,
            sort_by: q.sort_by.unwrap_or_default(),
            in_meal_plan: q.in_meal_plan.unwrap_or(false),
            mine: q.mine.unwrap_or(false),
            no_image: q.no_image.unwrap_or(false),
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Query(input): Query<PageQuery>,
) -> impl IntoResponse {
    let query = input.clone();

    let browse = imkitchen_web_shared::try_page_response!(
        recipe::browse(&app, &user.id, BrowseQuery::from(input)),
        template
    );

    template
        .render(IndexTemplate {
            user,
            recipes: browse.recipes,
            query,
            has_shared: browse.has_shared,
            ..Default::default()
        })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn create(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let id = imkitchen_web_shared::try_response!(
        recipe::create_or_resume_draft(&app, &user.id, user.username.to_owned()),
        template
    );

    Redirect::to(&format!("/recipes/{id}/edit")).into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-share-all-button.html")]
pub struct ShareAllButtonTemplate {
    pub has_shared: bool,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn share_all(
    template: Template,
    State(app): State<AppState>,
    RequireChef(user): RequireChef,
) -> impl IntoResponse {
    match recipe::share_all(&app, &user.id, user.username.as_deref()).await {
        Ok(()) => {}
        Err(ShareError::UsernameRequired) => {
            return (
                [("ts-swap", "skip")],
                template.render(SetUsernameModalTemplate),
            )
                .into_response();
        }
        Err(ShareError::Core(err)) => {
            imkitchen_web_shared::try_response!(sync: Err::<(), _>(err), template);
        }
    }

    template
        .render(ShareAllButtonTemplate { has_shared: true })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn make_all_private(
    template: Template,
    State(app): State<AppState>,
    RequireChef(user): RequireChef,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(recipe::unshare_all(&app, &user.id), template);

    template
        .render(ShareAllButtonTemplate { has_shared: false })
        .into_response()
}
