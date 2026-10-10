//! Guided tour progress for the web client (`static/js/tour.js`): the
//! same operations as `/api/v1/tours`, cookie-authenticated. Writes answer
//! an empty `ts-swap: skip` body; the script never renders the response.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum_extra::extract::Form;
use serde::Deserialize;

use imkitchen_web_shared::AppState;
use imkitchen_web_shared::auth::AuthUser;
use imkitchen_web_shared::services::tour;
use imkitchen_web_shared::template::Template;

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn state(State(app): State<AppState>, user: AuthUser) -> impl IntoResponse {
    match tour::state(&app, &user.id).await {
        Ok(state) => ([(header::CACHE_CONTROL, "no-store")], Json(state)).into_response(),
        Err(err) => {
            tracing::error!("{err:?}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct StepInput {
    pub step: u16,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn advance(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
    Form(input): Form<StepInput>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(tour::advance(&app, &user.id, &id, input.step), template);
    ([("ts-swap", "skip")], "").into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn complete(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(tour::complete(&app, &user.id, &id), template);
    ([("ts-swap", "skip")], "").into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn skip(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Path((id,)): Path<(String,)>,
    Form(input): Form<StepInput>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(tour::skip(&app, &user.id, &id, input.step), template);
    ([("ts-swap", "skip")], "").into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn reset(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(tour::reset(&app, &user.id), template);
    ([("ts-swap", "skip")], "").into_response()
}
