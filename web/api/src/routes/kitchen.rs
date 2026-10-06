use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use imkitchen_web_shared::{AppState, services::kitchen};

use crate::{
    ApiError, ApiJson, ApiResult,
    auth::ApiUser,
    dto::kitchen::{CookingScreen, Dish, GenerateRequest, Overview, StepRequest},
};

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn overview(State(app): State<AppState>, user: ApiUser) -> ApiResult<Overview> {
    let view = kitchen::overview(&app, &user.id).await?;
    Ok(Json(Overview::from(view)))
}

/// Replaces the list and returns the refreshed kitchen (the list is read from
/// the aggregate, so it is already up to date).
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn generate(
    State(app): State<AppState>,
    user: ApiUser,
    ApiJson(input): ApiJson<GenerateRequest>,
) -> ApiResult<Overview> {
    kitchen::generate(&app, &user.id, input.count).await?;
    let view = kitchen::overview(&app, &user.id).await?;
    Ok(Json(Overview::from(view)))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn remove(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> Result<StatusCode, ApiError> {
    kitchen::remove(&app, &user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn dish(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> ApiResult<Dish> {
    let view = kitchen::dish(&app, &user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(Dish::from(view)))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn cook(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
) -> ApiResult<CookingScreen> {
    let view = kitchen::cooking_screen(&app, &user.id, &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(CookingScreen::from(view)))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn step(
    State(app): State<AppState>,
    user: ApiUser,
    Path((id,)): Path<(String,)>,
    ApiJson(input): ApiJson<StepRequest>,
) -> ApiResult<CookingScreen> {
    let view = kitchen::step(&app, &user.id, &id, input.direction.as_str())
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(CookingScreen::from(view)))
}
