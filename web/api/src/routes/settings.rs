use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use imkitchen_identity::{meal_preferences, password::RequestInput, user_profile};
use imkitchen_web_shared::{AppState, services::settings};

use crate::{
    ApiError, ApiJson, ApiResult,
    auth::{ApiClaims, ApiLocale, ApiUser},
    dto::settings::{
        AisleOrderRequest, DeleteAccountRequest, General, PreferencesRequest, ProfileRequest,
        Session, UsernameRequest,
    },
};

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn general(State(app): State<AppState>, user: ApiUser) -> ApiResult<General> {
    let general = settings::general(&app, &user.id).await?;
    Ok(Json(General::from(general)))
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn preferences(
    State(app): State<AppState>,
    user: ApiUser,
    ApiJson(input): ApiJson<PreferencesRequest>,
) -> Result<StatusCode, ApiError> {
    app.identity
        .meal_preferences
        .update(
            &user.id,
            meal_preferences::UpdateInput {
                household_size: input.household_size,
                dietary_restrictions: input.dietary_restrictions,
                cuisine_variety_weight: input.cuisine_variety_weight,
                recipe_types: input.recipe_types,
            },
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn aisle_order(
    State(app): State<AppState>,
    user: ApiUser,
    ApiJson(input): ApiJson<AisleOrderRequest>,
) -> Result<StatusCode, ApiError> {
    app.identity
        .meal_preferences
        .set_aisle_order(&user.id, input.aisles)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn profile(
    State(app): State<AppState>,
    user: ApiUser,
    ApiJson(input): ApiJson<ProfileRequest>,
) -> Result<StatusCode, ApiError> {
    app.identity
        .user_profile
        .update(
            &user.id,
            user_profile::UpdateInput {
                description: input.description,
            },
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn username(
    State(app): State<AppState>,
    user: ApiUser,
    ApiJson(input): ApiJson<UsernameRequest>,
) -> Result<StatusCode, ApiError> {
    settings::set_username(&app, &user, input.username).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Emails the signed-in user a password reset link (the web "account" page
/// does the same: passwords are never changed in-app).
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn password_reset(
    State(app): State<AppState>,
    user: ApiUser,
    locale: ApiLocale,
) -> Result<StatusCode, ApiError> {
    app.identity
        .password
        .request(RequestInput {
            email: user.email.to_owned(),
            lang: locale.lang,
            host: app.config.server.url.to_owned(),
        })
        .await?;
    Ok(StatusCode::ACCEPTED)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn sessions(
    State(app): State<AppState>,
    claims: ApiClaims,
    user: ApiUser,
) -> ApiResult<Vec<Session>> {
    let logins = settings::sessions(&app, &user.id).await?;
    Ok(Json(
        logins
            .iter()
            .map(|l| Session::new(l, &claims.acc))
            .collect(),
    ))
}

/// Signs out one device. Signing out the current one works too; the app then
/// gets 401 on its next request.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn revoke_session(
    State(app): State<AppState>,
    user: ApiUser,
    Path((acc,)): Path<(String,)>,
) -> Result<StatusCode, ApiError> {
    app.identity.logout(&user.id, acc).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Deletes the account for good. The password is re-checked; a wrong one is
/// a 400 `user` error. On 204 every session is gone and the app returns to
/// the login screen.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn delete_account(
    State(app): State<AppState>,
    user: ApiUser,
    ApiJson(input): ApiJson<DeleteAccountRequest>,
) -> Result<StatusCode, ApiError> {
    settings::delete_account(&app, &user, &input.password).await?;
    Ok(StatusCode::NO_CONTENT)
}
