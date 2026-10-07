use axum::{
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
};
use serde::de::DeserializeOwned;

use crate::ApiError;

/// `axum::Json` for request bodies, but every rejection (missing content
/// type, malformed JSON, unknown enum variant…) answers with the API error
/// envelope instead of axum's plain-text body.
pub struct ApiJson<T>(pub T);

impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(ApiJson(value)),
            Err(rejection) => Err(match rejection.status() {
                StatusCode::PAYLOAD_TOO_LARGE => ApiError::User(rejection.body_text()),
                _ => ApiError::Validation(describe(&rejection)),
            }),
        }
    }
}

fn describe(rejection: &JsonRejection) -> String {
    match rejection {
        JsonRejection::MissingJsonContentType(_) => {
            "Expected a JSON body (Content-Type: application/json)".to_owned()
        }
        other => other.body_text(),
    }
}
