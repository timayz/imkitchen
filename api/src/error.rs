//! What the api answers when it does not do what was asked.
//!
//! A refusal is JSON, `{"error": code}`, the code a stable word a client
//! shows in its own language. The status says whose doing it is: 400 for a
//! body that is not one, 401 for a request that no longer signs anyone in
//! (`sign_in_required` without a token, `signed_out` with one that stopped
//! working: the client forgets it), 403 for a role the user does not have,
//! 422 for what the person can correct (a wrong code, a weak password), 429
//! for a code asked again too soon and 503 for a store that is not taking
//! writes for the moment. Anything else is a 500 that says nothing, logged
//! here.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use serde_json::json;
use timada_identity::IdentityError;

/// The request carries no token.
pub const SIGN_IN_REQUIRED: &str = "sign_in_required";
/// The token the request carries no longer signs anyone in.
pub const SIGNED_OUT: &str = "signed_out";
pub const NOT_ALLOWED: &str = "not_allowed";
pub const INVALID_REQUEST: &str = "invalid_request";
/// What was asked no longer fits the account as it stands (the address
/// changed hands while its code was on the way, say): start again.
pub const INVALID_STATE: &str = "invalid_state";

#[derive(Debug)]
pub enum ApiError {
    Refused(StatusCode, &'static str),
    Internal(anyhow::Error),
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub fn refused(status: StatusCode, code: &'static str) -> Self {
        Self::Refused(status, code)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err)
    }
}

impl From<IdentityError> for ApiError {
    fn from(err: IdentityError) -> Self {
        let status = match &err {
            IdentityError::Unauthorized => StatusCode::UNAUTHORIZED,
            IdentityError::Forbidden => StatusCode::FORBIDDEN,
            IdentityError::UserNotFound => StatusCode::NOT_FOUND,
            IdentityError::CodeRecentlySent => StatusCode::TOO_MANY_REQUESTS,
            IdentityError::InvalidState => {
                return Self::Refused(StatusCode::CONFLICT, INVALID_STATE);
            }
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        };
        match err.code() {
            // The store's doing, not the request's: sent again, it very
            // likely goes through.
            Some(code) if timada_core::is_write_refusal(code) => {
                Self::Refused(StatusCode::SERVICE_UNAVAILABLE, code)
            }
            Some(code) => Self::Refused(status, code),
            None => Self::Internal(err.into()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Self::Refused(status, code) => (status, code),
            Self::Internal(err) => {
                tracing::error!(err = format!("{err:#}"), "request failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal")
            }
        };
        (status, Json(json!({ "error": code }))).into_response()
    }
}

/// A JSON body, refused the way everything else is when it is not one.
pub struct Body<T>(pub T);

impl<S, T> FromRequest<S> for Body<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(body)) => Ok(Self(body)),
            Err(JsonRejection::JsonDataError(_)) => Err(ApiError::refused(
                StatusCode::UNPROCESSABLE_ENTITY,
                INVALID_REQUEST,
            )),
            Err(_) => Err(ApiError::refused(StatusCode::BAD_REQUEST, INVALID_REQUEST)),
        }
    }
}
