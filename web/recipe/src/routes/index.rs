use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::Query;
use evento::cursor::{Args, ReadResult, Value};
use imkitchen_core::recipe::query::user::{RecipesQuery, SortBy, UserViewList};
use imkitchen_types::recipe::RecipeType;
use serde::Deserialize;
use std::str::FromStr;
use strum::VariantArray;

use imkitchen_web_shared::{
    AppState,
    auth::{AuthUser, RequireChef, RequirePremium},
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

    let in_meal_plan = input.in_meal_plan.unwrap_or(false);
    let mine = input.mine.unwrap_or(false);

    let dietary_restrictions = if in_meal_plan || mine {
        vec![]
    } else {
        let preferences = imkitchen_web_shared::try_page_response!(
            app.identity.meal_preferences.load(&user.id),
            template
        );
        preferences.dietary_restrictions
    };

    let (user_id, is_shared) = if mine {
        (Some(user.id.to_owned()), None)
    } else {
        (None, Some(true))
    };

    // `in_meal_plan: Some((_, false))` is an exclusion (NOT EXISTS), used by the
    // community browse default to hide already-planned recipes from the picker.
    // When filtering by ownership, that exclusion would hide every recipe the
    // user has already added — drop the plan filter unless In plan is on.
    let in_meal_plan_filter = if in_meal_plan {
        Some((user.id.to_owned(), true))
    } else if mine {
        None
    } else {
        Some((user.id.to_owned(), false))
    };

    let has_thumbnail = if input.no_image.unwrap_or(false) {
        Some(false)
    } else {
        None
    };

    let recipes = imkitchen_web_shared::try_page_response!(
        app.core.recipe.filter_user(RecipesQuery {
            exclude_ids: None,
            user_id,
            recipe_type,
            is_shared,
            has_thumbnail,
            dietary_restrictions,
            dietary_where_any: false,
            in_meal_plan: in_meal_plan_filter,
            sort_by: input.sort_by.unwrap_or_default(),
            args: args.limit(20),
            search: input.search,
        }),
        template
    );

    // Drives the Share All / Make All Private toggle on the chef toolbar.
    // Only considers the loaded page of recipes.
    let has_shared = recipes.edges.iter().any(|r| r.node.is_shared);

    template
        .render(IndexTemplate {
            user,
            recipes,
            query,
            has_shared,
            ..Default::default()
        })
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
        .render(ShareAllButtonTemplate { has_shared: true })
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
        .render(ShareAllButtonTemplate { has_shared: false })
        .into_response()
}
