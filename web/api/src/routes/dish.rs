use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use imkitchen_core::dish::query::CandidateQuery;
use imkitchen_web_shared::{AppState, services::dish};

use crate::{
    ApiError, ApiJson, ApiResult,
    auth::ApiUser,
    dto::dish::{Candidates, CandidatesQuery, ComposeRequest, Created, Dish, Dishes},
};

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn list(State(app): State<AppState>, user: ApiUser) -> ApiResult<Dishes> {
    let dishes = dish::list(&app, &user.id).await?;
    Ok(Json(Dishes {
        dishes: dishes.into_iter().map(Dish::from).collect(),
    }))
}

/// A page of the pool by name: `?recipe_type=MainCourse&search=pota&limit=20`.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn candidates(
    State(app): State<AppState>,
    user: ApiUser,
    Query(query): Query<CandidatesQuery>,
) -> ApiResult<Candidates> {
    let page = dish::candidates(
        &app,
        &user.id,
        CandidateQuery {
            recipe_type: query.recipe_type,
            search: query.search,
            exclude_ids: vec![],
            limit: query.limit.unwrap_or(dish::CANDIDATE_PAGE),
        },
    )
    .await?;
    Ok(Json(Candidates {
        candidates: page.items.into_iter().map(Into::into).collect(),
        total: page.total,
    }))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn create(
    State(app): State<AppState>,
    user: ApiUser,
    ApiJson(input): ApiJson<ComposeRequest>,
) -> Result<(StatusCode, Json<Created>), ApiError> {
    let id = dish::create(&app, &user.id, input.into()).await?;
    Ok((StatusCode::CREATED, Json(Created { id })))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn detail(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> ApiResult<Dish> {
    let view = dish::find(&app, &user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(Dish::from(view)))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn update(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
    ApiJson(input): ApiJson<ComposeRequest>,
) -> Result<StatusCode, ApiError> {
    dish::update(&app, &user.id, &id, input.into()).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn delete(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> Result<StatusCode, ApiError> {
    dish::delete(&app, &user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Adds every recipe of the dish to the kitchen list.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn add_to_shopping(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> Result<StatusCode, ApiError> {
    dish::add_to_list(&app, &user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
