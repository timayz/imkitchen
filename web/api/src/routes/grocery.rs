use axum::{Json, extract::State, http::StatusCode};
use imkitchen_web_shared::{AppState, services::grocery};

use crate::{
    ApiError, ApiJson, ApiResult,
    auth::ApiUser,
    dto::grocery::{Groceries, ToggleRequest},
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
