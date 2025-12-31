use std::{
    ops::Deref,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{
    TypedHeader,
    extract::{
        CookieJar,
        cookie::{Cookie, Expiration, SameSite},
    },
    headers::UserAgent,
};
use imkitchen_user::State;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    config::JwtConfig,
    template::{ForbiddenTemplate, Template},
};

const AUTH_COOKIE_NAME: &str = "auth_token";

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    aud: String,            // Optional. Audience
    exp: u64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: u64, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    pub(crate) sub: String, // Optional. Subject (whom token refers to)
    pub(crate) acc: String, // Optional. Subject (whom token refers to)
}

pub fn build_cookie<'a>(config: JwtConfig, sub: String, acc: String) -> anyhow::Result<Cookie<'a>> {
    let now = OffsetDateTime::now_utc();
    let expire_days = time::Duration::days(config.expiration_days.into());
    let auth_expires = Expiration::from(now + expire_days);
    let claims = Claims {
        aud: config.audience.to_owned(),
        exp: (now + expire_days).unix_timestamp().try_into()?,
        iat: now.unix_timestamp().try_into()?,
        iss: config.issuer.to_owned(),
        sub,
        acc,
    };

    let auth_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )?;

    Ok(Cookie::build((AUTH_COOKIE_NAME, auth_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .expires(auth_expires)
        .build())
}

pub fn auth_cookie<'a>() -> Cookie<'a> {
    Cookie::from(AUTH_COOKIE_NAME)
}

#[derive(Clone, Default)]
pub struct AuthToken(Claims);

impl Deref for AuthToken {
    type Target = Claims;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<crate::routes::AppState> for AuthToken {
    type Rejection = Redirect;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::routes::AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(AuthToken(claims.clone()));
        }

        // Extract cookie jar
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| Redirect::to("/login"))?;

        // Get JWT token from cookie
        let token = jar
            .get(AUTH_COOKIE_NAME)
            .map(|cookie| cookie.value())
            .ok_or(Redirect::to("/login"))?;

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[state.config.jwt.issuer.to_owned()]);
        validation.set_audience(&[state.config.jwt.audience.to_owned()]);

        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(state.config.jwt.secret.as_bytes()),
            &validation,
        )
        .map_err(|_| Redirect::to("/login"))?;

        parts.extensions.insert(token_data.claims.clone());

        Ok(AuthToken(token_data.claims))
    }
}

impl FromRequestParts<crate::routes::AppState> for Option<AuthToken> {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::routes::AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(AuthToken::from_request_parts(parts, state).await.ok())
    }
}

#[derive(Clone, Default)]
pub struct AuthUser(imkitchen_user::login::Login);

impl Deref for AuthUser {
    type Target = imkitchen_user::login::Login;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<crate::routes::AppState> for AuthUser {
    type Rejection = Redirect;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::routes::AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_agent = TypedHeader::<UserAgent>::from_request_parts(parts, state)
            .await
            .map_err(|_| Redirect::to("/login"))?;

        let claims = AuthToken::from_request_parts(parts, state).await?;

        let Some(user) = imkitchen_user::login::load(&state.executor, &state.read_db, &claims.sub)
            .await
            .map_err(|e| {
                tracing::error!("{e}");
                Redirect::to("/login")
            })?
        else {
            return Err(Redirect::to("/login"));
        };

        let Some(login) = user
            .logins
            .iter()
            .find(|l| l.id == claims.acc && l.user_agent == user_agent.to_string())
        else {
            return Err(Redirect::to("/login"));
        };

        let mut login = login.clone();

        login.id = user.id;

        if login.state.0 == State::Suspended {
            return Err(Redirect::to("/login"));
        }

        if !state.config.features.premium || login.is_admin() {
            login.subscription_expire_at = (SystemTime::now() + time::Duration::weeks(10 * 52))
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs());
        }

        parts.extensions.insert(login.clone());

        Ok(AuthUser(login))
    }
}

impl FromRequestParts<crate::routes::AppState> for Option<AuthUser> {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::routes::AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(AuthUser::from_request_parts(parts, state).await.ok())
    }
}

pub struct AuthAdmin(imkitchen_user::login::Login);

impl Deref for AuthAdmin {
    type Target = imkitchen_user::login::Login;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<crate::routes::AppState> for AuthAdmin {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::routes::AppState,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        if user.is_admin() {
            return Ok(AuthAdmin(user));
        }

        let template = Template::from_request_parts(parts, state)
            .await
            .expect("Infallible");

        Err(template.render(ForbiddenTemplate).into_response())
    }
}
