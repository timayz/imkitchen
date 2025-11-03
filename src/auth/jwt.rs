//! JWT token generation and validation

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT claims payload
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// User ID
    pub sub: String,
    /// Is admin
    pub is_admin: bool,
    /// Expiration timestamp
    pub exp: u64,
}

/// User information extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub is_admin: bool,
}

/// Generate a JWT token for a user
pub fn generate_token(
    user_id: String,
    is_admin: bool,
    secret: &str,
    lifetime_seconds: u64,
) -> anyhow::Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let claims = Claims {
        sub: user_id,
        is_admin,
        exp: now + lifetime_seconds,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

/// Validate and decode a JWT token
pub fn validate_token(token: &str, secret: &str) -> anyhow::Result<AuthUser> {
    let validation = Validation::default();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(AuthUser {
        user_id: token_data.claims.sub,
        is_admin: token_data.claims.is_admin,
    })
}

/// Implement FromRequestParts for AuthUser to extract it from cookies
impl FromRequestParts<crate::routes::AppState> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::routes::AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract cookie jar
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        // Get JWT token from cookie
        let token = jar
            .get(super::AUTH_COOKIE_NAME)
            .map(|cookie| cookie.value())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        // Validate token using secret from app state
        validate_token(token, &state.jwt_secret).map_err(|_| StatusCode::UNAUTHORIZED)
    }
}
