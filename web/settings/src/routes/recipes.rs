use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use evento::cursor::{Args, ReadResult, Value};
use imkitchen_core::recipe::query::user::{RecipesQuery, SortBy, UserViewList};
use imkitchen_types::recipe::{CuisineType, RecipeType};
use serde::Deserialize;
use std::str::FromStr;
use strum::VariantArray;

use imkitchen_web_shared::{
    AppState,
    auth::{AuthUser, RequireChef, RequirePremium},
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
    pub has_shared: bool,
}

impl Default for RecipesTemplate {
    fn default() -> Self {
        Self {
            current_path: "settings".to_owned(),
            settings_path: "recipes".to_owned(),
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
    pub cuisine_type: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<SortBy>,
    pub no_image: Option<bool>,
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

    let recipes = imkitchen_web_shared::try_page_response!(
        app.core.recipe.filter_user(RecipesQuery {
            exclude_ids: None,
            user_id: Some(user.id.to_owned()),
            recipe_type,
            cuisine_type,
            is_shared: None,
            has_thumbnail: if input.no_image.unwrap_or(false) {
                Some(false)
            } else {
                None
            },
            dietary_restrictions: vec![],
            dietary_where_any: false,
            in_meal_plan: None,
            sort_by: input.sort_by.unwrap_or_default(),
            args: args.limit(20),
            search: input.search,
        }),
        template
    );

    let has_shared = recipes.edges.iter().any(|r| r.node.is_shared);

    template
        .render(RecipesTemplate {
            user,
            recipes,
            query,
            has_shared,
            ..Default::default()
        })
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/set-username-modal.html")]
pub struct SetUsernameModalTemplate;

#[derive(askama::Template)]
#[template(path = "partials/settings-recipes-share-button.html")]
pub struct ShareButtonTemplate {
    pub has_shared: bool,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn share_all(
    template: Template,
    State(app): State<AppState>,
    RequireChef(user): RequireChef,
) -> impl IntoResponse {
    let Some(ref username) = user.username else {
        return (
            [("ts-swap", "skip")],
            template.render(SetUsernameModalTemplate),
        )
            .into_response();
    };

    imkitchen_web_shared::try_response!(
        app.core.recipe.share_all_to_community(&user.id, username),
        template
    );

    template
        .render(ShareButtonTemplate { has_shared: true })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn make_all_private(
    template: Template,
    State(app): State<AppState>,
    RequireChef(user): RequireChef,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(app.core.recipe.make_all_private(&user.id), template);

    template
        .render(ShareButtonTemplate { has_shared: false })
        .into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn create(
    template: Template,
    RequirePremium(user): RequirePremium,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let id = match imkitchen_web_shared::try_response!(anyhow: app.core.recipe.find_user_draft(&user.id), template)
    {
        Some(id) => id,
        _ => imkitchen_web_shared::try_response!(
            app.core.recipe.create(&user.id, user.username.to_owned()),
            template
        ),
    };

    Redirect::to(&format!("/recipes/{id}/edit")).into_response()
}
