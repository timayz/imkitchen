use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use imkitchen_web_shared::{AppState, services::tour};

use crate::{
    ApiError, ApiJson, ApiResult,
    auth::ApiUser,
    dto::tour::{StepRequest, Tours},
};

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn tours(State(app): State<AppState>, user: ApiUser) -> ApiResult<Tours> {
    let state = tour::state(&app, &user.id).await?;
    Ok(Json(Tours::from(state)))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn advance(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
    ApiJson(input): ApiJson<StepRequest>,
) -> Result<StatusCode, ApiError> {
    tour::advance(&app, &user.id, &id, input.step).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn complete(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> Result<StatusCode, ApiError> {
    tour::complete(&app, &user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn skip(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
    ApiJson(input): ApiJson<StepRequest>,
) -> Result<StatusCode, ApiError> {
    tour::skip(&app, &user.id, &id, input.step).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn reset(State(app): State<AppState>, user: ApiUser) -> Result<StatusCode, ApiError> {
    tour::reset(&app, &user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
