use std::str::FromStr;

use axum::{
    Form,
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
};

use evento::cursor::{self, Args, ReadResult};
use imkitchen_recipe::{
    comment::{AddCommentInput, ReplyCommentInput},
    comment_rating, favorite,
    query::{
        self,
        comment::{self, CommentView, CommentsQuery},
        user::{RecipesQuery, SortBy, UserView, UserViewList},
        user_stat::UserStatView,
    },
    rating,
};
use imkitchen_shared::recipe::{IngredientUnitFormat, RecipeType};
use serde::Deserialize;

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
pub struct DetailTemplate<'a> {
    pub current_path: String,
    pub user: AuthUser,
    pub username: &'a str,
    pub recipe: UserView,
    pub stat: UserStatView,
    pub rating: rating::Rating,
    pub favorite: favorite::Favorite,
    pub comment: Option<query::comment::CommentView>,
    pub comment_rating: Option<comment_rating::CommentRating>,
    pub cook_recipes: ReadResult<UserViewList>,
    pub similar_recipes: ReadResult<UserViewList>,
}

impl<'a> Default for DetailTemplate<'a> {
    fn default() -> Self {
        Self {
            current_path: "recipes".to_owned(),
            user: AuthUser::default(),
            recipe: Default::default(),
            stat: UserStatView::default(),
            cook_recipes: Default::default(),
            similar_recipes: Default::default(),
            rating: Default::default(),
            favorite: Default::default(),
            comment: Default::default(),
            comment_rating: Default::default(),
            username: "john_doe",
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

    let rating =
        crate::try_page_response!(app.recipe_cmd.rating.load(&recipe.id, &user.id), template)
            .to_owned();

    let favorite =
        crate::try_page_response!(app.recipe_cmd.favorite.load(&recipe.id, &user.id), template)
            .to_owned();

    let comment =
        crate::try_page_response!(app.recipe_query.comment(&recipe.id, &user.id), template);

    let comment_rating = match comment {
        Some(ref comment) => Some(
            crate::try_page_response!(
                app.recipe_cmd.comment_rating.load(&comment.id, &user.id),
                template
            )
            .to_owned(),
        ),
        _ => None,
    };

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
            sort_by: SortBy::RecentlyAdded,
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
            sort_by: SortBy::RecentlyAdded,
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
                sort_by: SortBy::RecentlyAdded,
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
                sort_by: SortBy::RecentlyAdded,
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
                sort_by: SortBy::RecentlyAdded,
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
                dietary_restrictions: vec![],
                dietary_where_any: false,
                sort_by: SortBy::RecentlyAdded,
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
                cuisine_type: Some(recipe.cuisine_type.0.to_owned()),
                recipe_type: None,
                is_shared: Some(true),
                dietary_restrictions: vec![],
                dietary_where_any: false,
                sort_by: SortBy::RecentlyAdded,
                args: Args::forward(6, None),
            }),
            template
        );

        similar_recipes.edges.extend(more_recipes.edges);
    }

    let username = user.username();

    template
        .render(DetailTemplate {
            user,
            recipe,
            stat,
            cook_recipes,
            similar_recipes,
            rating,
            favorite,
            comment,
            comment_rating,
            username: username.as_str(),
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
    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&id), template);
    if recipe.owner_id == user.id {
        return "<div></div>".into_response();
    }

    if !recipe.is_shared {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::NotFound("recipe".to_owned())
        ), template);
    }

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
    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&id), template);

    if !recipe.is_shared {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::NotFound("recipe".to_owned())
        ), template);
    }

    crate::try_response!(app.recipe_cmd.rating.check_like(&id, &user.id), template);

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
    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&id), template);

    if !recipe.is_shared {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::NotFound("recipe".to_owned())
        ), template);
    }

    crate::try_response!(app.recipe_cmd.rating.uncheck_like(&id, &user.id), template);

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
    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&id), template);

    if !recipe.is_shared {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::NotFound("recipe".to_owned())
        ), template);
    }

    crate::try_response!(app.recipe_cmd.rating.check_unlike(&id, &user.id), template);

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
    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&id), template);

    if !recipe.is_shared {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::NotFound("recipe".to_owned())
        ), template);
    }

    crate::try_response!(
        app.recipe_cmd.rating.uncheck_unlike(&id, &user.id),
        template
    );

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

#[derive(askama::Template)]
#[template(path = "partials/recipes-detail-save-button.html")]
pub struct SaveButtonTemplate {
    pub id: String,
    pub saved: bool,
}

pub async fn save(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&id),template);

    if !recipe.is_shared {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::NotFound("recipe".to_owned())
        ), template);
    }

    crate::try_response!(
        app.recipe_cmd.favorite.save(&id, recipe.owner_id, &user.id),
        template
    );

    (
        [("ts-swap", "skip")],
        template.render(SaveButtonTemplate { id, saved: true }),
    )
        .into_response()
}

pub async fn unsave(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    crate::try_response!(app.recipe_cmd.favorite.unsave(&id, &user.id), template);

    (
        [("ts-swap", "skip")],
        template.render(SaveButtonTemplate { id, saved: false }),
    )
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-add-comment-btn.html")]
pub struct AddCommentBtnTemplate<'a> {
    pub id: &'a str,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn add_comment_btn(
    template: Template,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    template
        .render(AddCommentBtnTemplate { id: &id })
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-add-comment-form.html")]
pub struct AddCommentFormTemplate<'a> {
    pub id: &'a str,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn add_comment_form(
    template: Template,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    template
        .render(AddCommentFormTemplate { id: &id })
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-add-comment.html")]
pub struct AddCommentTemplate<'a> {
    pub username: &'a str,
    pub body: &'a str,
    pub created_at: &'a u64,
}

#[derive(Deserialize, Default, Clone)]
pub struct AddCommentForm {
    pub body: String,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn add_comment_action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
    Form(input): Form<AddCommentForm>,
) -> impl IntoResponse {
    let Some(ref username) = user.username else {
        return (
            [("ts-swap", "skip")],
            template.render(SetUsernameModalTemplate),
        )
            .into_response();
    };

    crate::try_response!(
        app.recipe_cmd.comment.add(
            &id,
            &user.id,
            AddCommentInput {
                body: input.body.to_owned(),
                owner_name: username.to_owned(),
            }
        ),
        template
    );

    let created_at = time::UtcDateTime::now().unix_timestamp() as u64;

    template
        .render(AddCommentTemplate {
            username,
            body: input.body.as_str(),
            created_at: &created_at,
        })
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-reply-btn.html")]
pub struct ReplyBtnTemplate<'a> {
    pub comment_id: &'a str,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn cancel_reply(
    template: Template,
    user: AuthUser,
    Path((_, comment_id)): Path<(String, String)>,
) -> impl IntoResponse {
    template
        .render(ReplyBtnTemplate {
            comment_id: &comment_id,
        })
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-reply-form.html")]
pub struct ReplyFormTemplate<'a> {
    pub recipe_id: &'a str,
    pub comment_id: &'a str,
    pub owner_name: Option<&'a str>,
}

#[derive(Deserialize, Default, Clone)]
pub struct ReplyFormQuery {
    pub owner_name: Option<String>,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn reply_form(
    template: Template,
    user: AuthUser,
    Query(query): Query<ReplyFormQuery>,
    Path((recipe_id, comment_id)): Path<(String, String)>,
) -> impl IntoResponse {
    template
        .render(ReplyFormTemplate {
            recipe_id: &recipe_id,
            comment_id: &comment_id,
            owner_name: query.owner_name.as_deref(),
        })
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-reply.html")]
pub struct ReplyTemplate<'a> {
    pub username: &'a str,
    pub body: &'a str,
    pub created_at: &'a u64,
    pub comment_id: &'a str,
}

#[derive(Deserialize, Default, Clone)]
pub struct ReplyForm {
    pub body: String,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn reply_action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((recipe_id, comment_id)): Path<(String, String)>,
    Form(input): Form<ReplyForm>,
) -> impl IntoResponse {
    let Some(ref username) = user.username else {
        return (
            [("ts-swap", "skip")],
            template.render(SetUsernameModalTemplate),
        )
            .into_response();
    };

    crate::try_response!(
        app.recipe_cmd.comment.reply(
            &recipe_id,
            &user.id,
            ReplyCommentInput {
                body: input.body.to_owned(),
                owner_name: username.to_owned(),
                comment_id: comment_id.to_owned(),
            }
        ),
        template
    );

    let created_at = time::UtcDateTime::now().unix_timestamp() as u64;

    template
        .render(ReplyTemplate {
            username,
            body: input.body.as_str(),
            created_at: &created_at,
            comment_id: &comment_id,
        })
        .into_response()
}

#[derive(Deserialize, Default, Clone)]
pub struct PageQuery {
    pub first: Option<u16>,
    pub after: Option<cursor::Value>,
    pub last: Option<u16>,
    pub before: Option<cursor::Value>,
    pub reply_to: Option<String>,
    #[serde(default)]
    pub include_current_user: bool,
    pub sort_by: Option<String>,
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-comments.html")]
pub struct CommentsTemplate {
    pub recipe_id: String,
    pub comments: ReadResult<CommentView>,
    pub ratings: Vec<comment_rating::CommentRating>,
    pub reply_to: Option<String>,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn comments(
    template: Template,
    user: AuthUser,
    Query(query): Query<PageQuery>,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let args = Args {
        first: query.first,
        after: query.after,
        last: query.last,
        before: query.before,
    };

    let exclude_owner = if !query.include_current_user {
        Some(user.id.to_owned())
    } else {
        None
    };

    let sort_by = comment::SortBy::from_str(&query.sort_by.unwrap_or("".to_owned()))
        .unwrap_or(comment::SortBy::RecentlyAdded);

    let comments = crate::try_page_response!(
        app.recipe_query.filter_comment(CommentsQuery {
            recipe_id: id.to_owned(),
            reply_to: query.reply_to.to_owned(),
            exclude_owner,
            sort_by,
            args: args.limit(5),
        }),
        template
    );

    let mut ratings = vec![];
    for comment in comments.edges.iter() {
        let rating = crate::try_response!(anyhow:
            app.recipe_cmd.comment_rating.load(&comment.node.id, &user.id),
            template
        );
        ratings.push(rating);
    }

    template
        .render(CommentsTemplate {
            recipe_id: id,
            comments,
            ratings,
            reply_to: query.reply_to,
        })
        .into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/recipes-detail-comment-like-button.html")]
pub struct CommentLikeButtonTemplate {
    pub recipe_id: String,
    pub comment_id: String,
    pub total: u64,
    pub liked: bool,
    pub unliked: bool,
}

pub async fn comment_check_like(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((recipe_id, comment_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&recipe_id), template);

    if !recipe.is_shared {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::NotFound("recipe".to_owned())
        ), template);
    }

    crate::try_response!(
        app.recipe_cmd
            .comment_rating
            .check_like(&comment_id, &user.id),
        template
    );

    (
        [("ts-swap", "skip")],
        template.render(CommentLikeButtonTemplate {
            recipe_id,
            comment_id,
            total: recipe.total_ulikes(),
            liked: true,
            unliked: false,
        }),
    )
        .into_response()
}

pub async fn comment_uncheck_like(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((recipe_id, comment_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&recipe_id), template);

    if !recipe.is_shared {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::NotFound("recipe".to_owned())
        ), template);
    }

    crate::try_response!(
        app.recipe_cmd
            .comment_rating
            .uncheck_like(&comment_id, &user.id),
        template
    );

    (
        [("ts-swap", "skip")],
        template.render(CommentLikeButtonTemplate {
            recipe_id,
            comment_id,
            total: recipe.total_ulikes(),
            liked: false,
            unliked: false,
        }),
    )
        .into_response()
}

pub async fn comment_check_unlike(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((recipe_id, comment_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&recipe_id), template);

    if !recipe.is_shared {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::NotFound("recipe".to_owned())
        ), template);
    }

    crate::try_response!(
        app.recipe_cmd
            .comment_rating
            .check_unlike(&comment_id, &user.id),
        template
    );

    (
        [("ts-swap", "skip")],
        template.render(CommentLikeButtonTemplate {
            recipe_id,
            comment_id,
            total: recipe.total_ulikes(),
            liked: false,
            unliked: true,
        }),
    )
        .into_response()
}

pub async fn comment_uncheck_unlike(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((recipe_id, comment_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let recipe = crate::try_response!(anyhow_opt: app.recipe_query.user(&recipe_id), template);

    if !recipe.is_shared {
        crate::try_response!(sync:
            Err(imkitchen_shared::Error::NotFound("recipe".to_owned())
        ), template);
    }

    crate::try_response!(
        app.recipe_cmd
            .comment_rating
            .uncheck_unlike(&comment_id, &user.id),
        template
    );

    (
        [("ts-swap", "skip")],
        template.render(CommentLikeButtonTemplate {
            recipe_id,
            comment_id,
            total: recipe.total_ulikes(),
            liked: false,
            unliked: false,
        }),
    )
        .into_response()
}
