//! Bearer-token authentication for the API. Mirrors the typestate extractors
//! of the HTML stack (`AuthUser`, `RequireChef`) but rejects with a JSON 401
//! or 403 instead of redirecting, and reads the token from `Authorization`
//! rather than a cookie.

use std::{convert::Infallible, ops::Deref};

use axum::{
    RequestPartsExt,
    extract::FromRequestParts,
    http::{HeaderMap, header, request::Parts},
};
use imkitchen_identity::login::Login;
use imkitchen_web_shared::{
    AppState,
    auth::{Claims, decode_claims, resolve_login},
    language::UserLanguage,
};

use crate::ApiError;

/// The client identity the app sends as `User-Agent`. It is stored verbatim
/// on the login record and must match on every later request, so the app
/// keeps it stable across updates (see `mobile/src/lib/client-identity.ts`).
pub fn client_identity(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .filter(|ua| !ua.trim().is_empty())
        .ok_or(ApiError::Unauthorized)
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|t| !t.is_empty())
}

/// Validated claims of the Bearer token. Cached in the request extensions so
/// `ApiUser` and handlers that need `acc` share one decode.
#[derive(Clone)]
pub struct ApiClaims(pub Claims);

impl Deref for ApiClaims {
    type Target = Claims;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<AppState> for ApiClaims {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(ApiClaims(claims.clone()));
        }

        let token = bearer_token(&parts.headers).ok_or(ApiError::Unauthorized)?;
        let claims = decode_claims(&state.config.jwt, token).ok_or(ApiError::Unauthorized)?;

        parts.extensions.insert(claims.clone());

        Ok(ApiClaims(claims))
    }
}

/// The signed-in user. Same resolution rules as the HTML `AuthUser`.
#[derive(Clone)]
pub struct ApiUser(pub Login);

impl Deref for ApiUser {
    type Target = Login;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<AppState> for ApiUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(login) = parts.extensions.get::<Login>() {
            return Ok(ApiUser(login.clone()));
        }

        let identity = client_identity(&parts.headers)?.to_owned();
        let claims = ApiClaims::from_request_parts(parts, state).await?;

        let login = resolve_login(state, &claims, &identity)
            .await
            .ok_or(ApiError::Unauthorized)?;

        parts.extensions.insert(login.clone());

        Ok(ApiUser(login))
    }
}

/// A signed-in chef (or admin). 403 otherwise.
pub struct ApiChef(pub Login);

impl Deref for ApiChef {
    type Target = Login;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<AppState> for ApiChef {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ApiUser(user) = ApiUser::from_request_parts(parts, state).await?;

        if user.is_chef() {
            Ok(ApiChef(user))
        } else {
            Err(ApiError::Forbidden)
        }
    }
}

/// Language and timezone of the request: `Accept-Language` (same resolution
/// as the HTML pages) and `X-Timezone`, falling back to the twinspark
/// `TS-Timezone` header and then UTC.
#[derive(Debug, Clone)]
pub struct ApiLocale {
    /// Two-letter language code, e.g. `fr`.
    pub lang: String,
    /// IANA timezone name, e.g. `Europe/Paris`.
    pub timezone: String,
}

impl<S> FromRequestParts<S> for ApiLocale
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user_language = parts
            .extract::<UserLanguage>()
            .await
            .expect("UserLanguage is infallible");

        let preferred = user_language
            .preferred_languages()
            .first()
            .cloned()
            .unwrap_or_else(|| "en".to_owned());
        let lang = preferred
            .split_once('-')
            .map_or(preferred.as_str(), |(iso, _)| iso)
            .to_owned();

        let timezone = ["x-timezone", "ts-timezone"]
            .iter()
            .find_map(|name| parts.headers.get(*name))
            .and_then(|v| v.to_str().ok())
            .filter(|tz| !tz.trim().is_empty())
            .map(String::from)
            .unwrap_or_else(|| "UTC".to_owned());

        Ok(ApiLocale { lang, timezone })
    }
}
