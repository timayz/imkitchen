use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};

use evento::cursor::{Args, ReadResult};
use imkitchen_recipe::{
    rating,
    user::{RecipesQuery, UserView, UserViewList},
    user_stat::UserStatView,
};
use imkitchen_shared::recipe::{IngredientUnitFormat, RecipeType};

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{NotFoundTemplate, Status, Template, filters},
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
#[template(path = "partials/recipes-detail-share-button.html")]
pub struct CommunityDetailShareButtonTemplate<'a> {
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
    pub recipe: UserView,
    pub stat: UserStatView,
    pub rating: rating::Rating,
    pub cook_recipes: ReadResult<UserViewList>,
    pub similar_recipes: ReadResult<UserViewList>,
}

impl Default for DetailTemplate {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: AuthUser::default(),
            recipe: Default::default(),
            stat: UserStatView::default(),
            cook_recipes: Default::default(),
            similar_recipes: Default::default(),
            rating: Default::default(),
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
    let recipe = crate::try_page_response!(opt: app.recipe_query.user(&id), template);

    if recipe.owner_id != user.id && !recipe.is_shared {
        return template.render(NotFoundTemplate).into_response();
    }

    let stat =
        crate::try_page_response!(app.recipe_query.find_user_stat(&recipe.owner_id), template)
            .unwrap_or_default();

    let rating = crate::try_page_response!(
        app.recipe_cmd.rating.load(&recipe.id, &recipe.owner_id),
        template
    )
    .to_owned();

    let exclude_ids = vec![recipe.id.to_owned()];

    let cook_recipes = crate::try_page_response!(
        app.recipe_query.filter_user(RecipesQuery {
            exclude_ids: Some(exclude_ids),
            user_id: Some(recipe.owner_id.to_owned()),
            recipe_type: None,
            cuisine_type: None,
            is_shared: Some(true),
            dietary_restrictions: vec![],
            dietary_where_any: false,
            sort_by: imkitchen_recipe::user::SortBy::RecentlyAdded,
            args: Args::forward(2, None),
        }),
        template
    );

    let mut exclude_ids = cook_recipes
        .edges
        .iter()
        .map(|n| n.node.id.to_owned())
        .collect::<Vec<_>>();

    exclude_ids.push(recipe.id.to_owned());

    let mut similar_recipes = crate::try_page_response!(
        app.recipe_query.filter_user(RecipesQuery {
            exclude_ids: Some(exclude_ids.to_vec()),
            user_id: None,
            recipe_type: Some(recipe.recipe_type.0.to_owned()),
            cuisine_type: Some(recipe.cuisine_type.0.to_owned()),
            is_shared: Some(true),
            dietary_restrictions: recipe.dietary_restrictions.0.to_vec(),
            dietary_where_any: false,
            sort_by: imkitchen_recipe::user::SortBy::RecentlyAdded,
            args: Args::forward(6, None),
        }),
        template
    );

    if similar_recipes.edges.len() < 6 {
        let mut similar_ids = similar_recipes
            .edges
            .iter()
            .map(|n| n.node.id.to_owned())
            .collect::<Vec<_>>();
        similar_ids.extend(exclude_ids.to_vec());

        let more_recipes = crate::try_page_response!(
            app.recipe_query.filter_user(RecipesQuery {
                exclude_ids: Some(similar_ids),
                user_id: None,
                recipe_type: Some(recipe.recipe_type.0.to_owned()),
                cuisine_type: Some(recipe.cuisine_type.0.to_owned()),
                is_shared: Some(true),
                dietary_restrictions: recipe.dietary_restrictions.0.to_vec(),
                dietary_where_any: true,
                sort_by: imkitchen_recipe::user::SortBy::RecentlyAdded,
                args: Args::forward(6, None),
            }),
            template
        );

        similar_recipes.edges.extend(more_recipes.edges);
    }

    if similar_recipes.edges.len() < 6 {
        let mut similar_ids = similar_recipes
            .edges
            .iter()
            .map(|n| n.node.id.to_owned())
            .collect::<Vec<_>>();
        similar_ids.extend(exclude_ids.to_vec());

        let more_recipes = crate::try_page_response!(
            app.recipe_query.filter_user(RecipesQuery {
                exclude_ids: Some(similar_ids),
                user_id: None,
                recipe_type: Some(recipe.recipe_type.0.to_owned()),
                cuisine_type: None,
                is_shared: Some(true),
                dietary_restrictions: recipe.dietary_restrictions.0.to_vec(),
                dietary_where_any: false,
                sort_by: imkitchen_recipe::user::SortBy::RecentlyAdded,
                args: Args::forward(6, None),
            }),
            template
        );

        similar_recipes.edges.extend(more_recipes.edges);
    }

    if similar_recipes.edges.len() < 6 {
        let mut similar_ids = similar_recipes
            .edges
            .iter()
            .map(|n| n.node.id.to_owned())
            .collect::<Vec<_>>();
        similar_ids.extend(exclude_ids.to_vec());

        let more_recipes = crate::try_page_response!(
            app.recipe_query.filter_user(RecipesQuery {
                exclude_ids: Some(similar_ids),
                user_id: None,
                recipe_type: Some(recipe.recipe_type.0.to_owned()),
                cuisine_type: None,
                is_shared: Some(true),
                dietary_restrictions: recipe.dietary_restrictions.0.to_vec(),
                dietary_where_any: true,
                sort_by: imkitchen_recipe::user::SortBy::RecentlyAdded,
                args: Args::forward(6, None),
            }),
            template
        );

        similar_recipes.edges.extend(more_recipes.edges);
    }

    if similar_recipes.edges.len() < 6 {
        let mut similar_ids = similar_recipes
            .edges
            .iter()
            .map(|n| n.node.id.to_owned())
            .collect::<Vec<_>>();
        similar_ids.extend(exclude_ids);

        let more_recipes = crate::try_page_response!(
            app.recipe_query.filter_user(RecipesQuery {
                exclude_ids: Some(similar_ids),
                user_id: None,
                recipe_type: Some(recipe.recipe_type.0.to_owned()),
                cuisine_type: None,
                is_shared: Some(true),
                dietary_restrictions: vec![],
                dietary_where_any: false,
                sort_by: imkitchen_recipe::user::SortBy::RecentlyAdded,
                args: Args::forward(6, None),
            }),
            template
        );

        similar_recipes.edges.extend(more_recipes.edges);
    }

    template
        .render(DetailTemplate {
            user,
            recipe,
            stat,
            cook_recipes,
            similar_recipes,
            rating,
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
    let Some(ref username) = user.username else {
        return (
            [("ts-swap", "skip")],
            template.render(SetUsernameModalTemplate),
        )
            .into_response();
    };

    crate::try_response!(
        app.recipe_cmd.share_to_community(&id, &user.id, username),
        template
    );

    template
        .render(CommunityDetailShareButtonTemplate {
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
    crate::try_response!(app.recipe_cmd.make_private(&id, &user.id), template);

    template
        .render(CommunityDetailShareButtonTemplate {
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
    crate::try_response!(app.recipe_cmd.delete(&id, &user.id), template);

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
        app.recipe_query.find_user(&id),
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

pub async fn check_in(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    crate::try_response!(app.recipe_cmd.rating.view(&id, &user.id), template);
    "<div></div>".into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-detail-like-button.html")]
pub struct LikeButtonTemplate {
    pub id: String,
    pub total: u64,
    pub liked: bool,
    pub unliked: bool,
}

pub async fn check_like(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    crate::try_response!(app.recipe_cmd.rating.check_like(&id, &user.id), template);

    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&id), template);

    (
        [("ts-swap", "skip")],
        template.render(LikeButtonTemplate {
            id,
            total: recipe.total_ulikes(),
            liked: true,
            unliked: false,
        }),
    )
        .into_response()
}

pub async fn uncheck_like(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    crate::try_response!(app.recipe_cmd.rating.uncheck_like(&id, &user.id), template);

    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&id), template);

    (
        [("ts-swap", "skip")],
        template.render(LikeButtonTemplate {
            id,
            total: recipe.total_ulikes(),
            liked: false,
            unliked: false,
        }),
    )
        .into_response()
}

pub async fn check_unlike(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    crate::try_response!(app.recipe_cmd.rating.check_unlike(&id, &user.id), template);

    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&id), template);

    (
        [("ts-swap", "skip")],
        template.render(LikeButtonTemplate {
            id,
            total: recipe.total_ulikes(),
            liked: false,
            unliked: true,
        }),
    )
        .into_response()
}

pub async fn uncheck_unlike(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    crate::try_response!(
        app.recipe_cmd.rating.uncheck_unlike(&id, &user.id),
        template
    );

    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&id), template);

    (
        [("ts-swap", "skip")],
        template.render(LikeButtonTemplate {
            id,
            total: recipe.total_ulikes(),
            liked: false,
            unliked: false,
        }),
    )
        .into_response()
}
