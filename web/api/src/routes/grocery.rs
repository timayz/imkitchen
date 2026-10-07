use axum::{Json, extract::State, http::StatusCode};
use imkitchen_web_shared::{AppState, services::grocery};

use crate::{
    ApiError, ApiJson, ApiResult,
    auth::ApiUser,
    dto::grocery::{CheckRequest, Groceries, ToggleRequest},
};

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn list(State(app): State<AppState>, user: ApiUser) -> ApiResult<Groceries> {
    let groceries = grocery::load(&app, &user.id).await?;
    Ok(Json(Groceries::from(groceries)))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn toggle(
    State(app): State<AppState>,
    user: ApiUser,
    ApiJson(input): ApiJson<ToggleRequest>,
) -> Result<StatusCode, ApiError> {
    grocery::toggle(&app, &user.id, input.key).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Absolute (idempotent) form of `toggle` for clients that replay queued
/// changes.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn check(
    State(app): State<AppState>,
    user: ApiUser,
    ApiJson(input): ApiJson<CheckRequest>,
) -> Result<StatusCode, ApiError> {
    grocery::set_checked(&app, &user.id, input.key, input.checked).await?;
    Ok(StatusCode::NO_CONTENT)
}
