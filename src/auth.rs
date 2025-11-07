use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use imkitchen_user::Role;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{
    config::JwtConfig,
    template::{ForbiddenTemplate, Template},
};

const AUTH_COOKIE_NAME: &str = "auth_token";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    aud: String, // Optional. Audience
    exp: u64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: u64, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    sub: String, // Optional. Subject (whom token refers to)
}

pub fn generate_token(config: JwtConfig, sub: String) -> anyhow::Result<String> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let claims = Claims {
        aud: config.audience,
        exp: now + config.expiration_days * 24 * 60 * 60,
        iat: now,
        iss: config.issuer,
        sub,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn build_cookie<'a>(config: JwtConfig, sub: String) -> anyhow::Result<Cookie<'a>> {
    let token = generate_token(config, sub)?;

    Ok(Cookie::build((AUTH_COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .build())
}

pub struct AuthUser(pub imkitchen_user::AuthUser);

impl FromRequestParts<crate::routes::AppState> for AuthUser {
    type Rejection = Redirect;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::routes::AppState,
    ) -> Result<Self, Self::Rejection> {
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

        let Some(user) = state
            .user_command
            .get_user_by_id(&token_data.claims.sub)
            .await
            .map_err(|e| {
                tracing::error!("{e}");
                Redirect::to("/login")
            })?
        else {
            return Err(Redirect::to("/login"));
        };

        if user.role == Role::Suspend.to_string() {
            return Err(Redirect::to("/login"));
        }

        Ok(AuthUser(user))
    }
}

pub struct AuthAdmin(pub imkitchen_user::AuthUser);

impl FromRequestParts<crate::routes::AppState> for AuthAdmin {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::routes::AppState,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        if user.role == Role::Admin.to_string() {
            return Ok(AuthAdmin(user));
        }

        let template = Template::<ForbiddenTemplate>::from_request_parts(parts, state)
            .await
            .expect("Infallible");

        Err(template.render(ForbiddenTemplate).into_response())
    }
}
