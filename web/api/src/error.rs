use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Error envelope: `{"error":{"code":"…","message":"…"}}`.
///
/// `code` is stable and machine-readable; `message` is for display and
/// already in the requester's language where the domain provides one.
#[derive(Debug)]
pub enum ApiError {
    Unauthorized,
    Forbidden,
    NotFound,
    Validation(String),
    User(String),
    UsernameRequired,
    Server(anyhow::Error),
}

pub type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Serialize)]
struct Envelope<'a> {
    error: Body<'a>,
}

#[derive(Serialize)]
struct Body<'a> {
    code: &'static str,
    message: &'a str,
}

impl ApiError {
    fn status(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::User(_) => StatusCode::BAD_REQUEST,
            Self::UsernameRequired => StatusCode::CONFLICT,
            Self::Server(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
            Self::Validation(_) => "validation",
            Self::User(_) => "user",
            Self::UsernameRequired => "username_required",
            Self::Server(_) => "server",
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::Unauthorized => "Authentication required",
            Self::Forbidden => "You are not allowed to do this",
            Self::NotFound => "Not found",
            Self::Validation(message) | Self::User(message) => message,
            Self::UsernameRequired => "A username is required first",
            Self::Server(_) => imkitchen_web_shared::template::SERVER_ERROR_MESSAGE,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if let Self::Server(err) = &self {
            tracing::error!("{err:?}");
        }

        let mut response = (
            self.status(),
            Json(Envelope {
                error: Body {
                    code: self.code(),
                    message: self.message(),
                },
            }),
        )
            .into_response();

        if matches!(self, Self::Unauthorized) {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }

        response
    }
}

impl From<imkitchen_core::Error> for ApiError {
    fn from(err: imkitchen_core::Error) -> Self {
        use imkitchen_core::Error;

        match err {
            Error::Validate(errors) => Self::Validation(errors.to_string()),
            Error::Forbidden(_) => Self::Forbidden,
            Error::NotFound(_) => Self::NotFound,
            Error::User(message) => Self::User(message),
            Error::Server(err) => Self::Server(err),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::Server(err)
    }
}
