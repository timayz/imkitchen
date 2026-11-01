use axum::{extract::State, response::IntoResponse};
use axum_extra::extract::Query;
use evento::cursor::{ReadResult, Value};
use imkitchen_core::recipe::query::{
    user::{SortBy, UserViewList},
    user_stat::UserStatView,
};
use imkitchen_types::recipe::RecipeType;
use serde::Deserialize;
use std::str::FromStr;
use strum::VariantArray;

use imkitchen_web_shared::{
    AppState,
    auth::AuthUser,
    services::recipe::{self, CookQuery, FollowedChef},
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
    /// The viewer is the chef: no Follow button.
    pub is_self: bool,
    pub following: bool,
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
            is_self: false,
            following: false,
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
    let query = input.clone();
    let cook_query = CookQuery {
        first: input.first,
        after: input.after,
        last: input.last,
        before: input.before,
        recipe_type: input
            .recipe_type
            .and_then(|v| RecipeType::from_str(v.as_str()).ok()),
        search: input.search,
        sort_by: input.sort_by.unwrap_or_default(),
    };

    let viewer = user.as_ref().map(|u| u.id.as_str());
    let Some(profile) = imkitchen_web_shared::try_page_response!(
        recipe::cook_profile(&app, &username, viewer, cook_query),
        template
    ) else {
        return template.render(NotFoundTemplate).into_response();
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

    template
        .render(CookTemplate {
            user,
            username,
            stat: profile.stat,
            owner_description: profile.description,
            recipes: profile.recipes,
            query,
            is_self: profile.is_self,
            following: profile.following,
            ..Default::default()
        })
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/cooks-follow-button.html")]
pub struct FollowButtonTemplate {
    pub username: String,
    pub following: bool,
}

pub async fn follow(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    axum::extract::Path((username,)): axum::extract::Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(recipe::follow(&app, &user.id, &username), template);

    (
        [("ts-swap", "skip")],
        template.render(FollowButtonTemplate {
            username,
            following: true,
        }),
    )
        .into_response()
}

pub async fn unfollow(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
    axum::extract::Path((username,)): axum::extract::Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(recipe::unfollow(&app, &user.id, &username), template);

    (
        [("ts-swap", "skip")],
        template.render(FollowButtonTemplate {
            username,
            following: false,
        }),
    )
        .into_response()
}

/// The chefs the user follows.
#[derive(askama::Template)]
#[template(path = "cooks-index.html")]
pub struct CooksIndexTemplate {
    pub current_path: String,
    pub user: AuthUser,
    pub chefs: Vec<FollowedChef>,
}

#[tracing::instrument(skip_all)]
pub async fn index(
    template: Template,
    AuthUser(user): AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let chefs =
        imkitchen_web_shared::try_page_response!(recipe::followed_chefs(&app, &user.id), template);

    template
        .render(CooksIndexTemplate {
            current_path: "recipes".to_owned(),
            user: AuthUser(user),
            chefs,
        })
        .into_response()
}
