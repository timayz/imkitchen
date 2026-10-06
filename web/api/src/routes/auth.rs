use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use imkitchen_identity::{LoginInput, RegisterInput, password::RequestInput};
use imkitchen_web_shared::{
    AppState,
    auth::{decode_claims, encode_token, resolve_login},
    services,
};

use crate::{
    ApiError, ApiJson, ApiResult,
    auth::{ApiClaims, ApiLocale, ApiUser, client_identity},
    dto::auth::{LoginRequest, Me, PasswordResetRequest, RegisterRequest, SessionResponse, Token},
};

#[tracing::instrument(skip_all)]
pub async fn login(
    State(app): State<AppState>,
    headers: HeaderMap,
    locale: ApiLocale,
    ApiJson(input): ApiJson<LoginRequest>,
) -> ApiResult<SessionResponse> {
    let identity = client_identity(&headers)?.to_owned();
    let session = sign_in(&app, identity, locale, input.email, input.password).await?;

    Ok(Json(session))
}

/// Creates the account and signs it in on this device, so the app lands in
/// the kitchen with a token instead of bouncing to the login screen like
/// the web does.
#[tracing::instrument(skip_all)]
pub async fn register(
    State(app): State<AppState>,
    headers: HeaderMap,
    locale: ApiLocale,
    ApiJson(input): ApiJson<RegisterRequest>,
) -> Result<(StatusCode, Json<SessionResponse>), ApiError> {
    let identity = client_identity(&headers)?.to_owned();

    services::auth::register(
        &app,
        RegisterInput {
            email: input.email.to_owned(),
            password: input.password.to_owned(),
            lang: locale.lang.to_owned(),
            timezone: locale.timezone.to_owned(),
        },
    )
    .await?;

    // The root address is signed up with the configured root password, not
    // the submitted one (see `services::auth::register`), so its session is
    // only issued when both agree.
    let session = sign_in(&app, identity, locale, input.email, input.password).await?;

    Ok((StatusCode::CREATED, Json(session)))
}

/// Verifies the credentials, records this device and resolves the login so
/// the response carries the same `user` the app would get from `GET /me`.
async fn sign_in(
    app: &AppState,
    identity: String,
    locale: ApiLocale,
    email: String,
    password: String,
) -> Result<SessionResponse, ApiError> {
    let session = services::auth::login(
        app,
        LoginInput {
            email,
            password,
            lang: locale.lang,
            timezone: locale.timezone,
            user_agent: identity.to_owned(),
        },
    )
    .await?;

    let claims = decode_claims(&app.config.jwt, &session.token).ok_or_else(|| {
        ApiError::Server(anyhow::anyhow!("freshly signed token failed to decode"))
    })?;
    let login = resolve_login(app, &claims, &identity)
        .await
        .ok_or_else(|| ApiError::Server(anyhow::anyhow!("login record missing after login")))?;

    Ok(SessionResponse {
        token: Token {
            token: session.token,
            expires_at: session.expires_at,
        },
        user: Me::new(app, &login),
    })
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn logout(
    State(app): State<AppState>,
    claims: ApiClaims,
    user: ApiUser,
) -> Result<StatusCode, ApiError> {
    app.identity.logout(&user.id, claims.acc.to_owned()).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Re-signs the current session (same user, same device) with a fresh
/// expiry. The HTML kitchen page does the same on every render.
#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn refresh(
    State(app): State<AppState>,
    claims: ApiClaims,
    user: ApiUser,
) -> ApiResult<Token> {
    let (token, expires_at) =
        encode_token(&app.config.jwt, user.id.to_owned(), claims.acc.to_owned())?;

    Ok(Json(Token { token, expires_at }))
}

/// Emails a reset link. Always 202, whether or not the address exists.
#[tracing::instrument(skip_all)]
pub async fn password_reset(
    State(app): State<AppState>,
    locale: ApiLocale,
    ApiJson(input): ApiJson<PasswordResetRequest>,
) -> Result<StatusCode, ApiError> {
    app.identity
        .password
        .request(RequestInput {
            email: input.email,
            lang: locale.lang,
            host: app.config.server.url.to_owned(),
        })
        .await?;

    Ok(StatusCode::ACCEPTED)
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn me(State(app): State<AppState>, user: ApiUser) -> ApiResult<Me> {
    Ok(Json(Me::new(&app, &user)))
}
