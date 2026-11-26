//! What the api answers when it does not do what was asked.
//!
//! A refusal is JSON, always of the one shape ([`ErrorBody`]):
//!
//! ```json
//! { "error": { "code": "validation_failed",
//!              "message": "Some fields are not valid.",
//!              "fields": [ { "field": "name", "code": "too_long",
//!                            "message": "This must be at most 255 characters.",
//!                            "params": { "max": 255 } } ] } }
//! ```
//!
//! `code` is a stable word a client acts on, `message` the same said to a
//! person, in the language the request asks for ([`crate::locale`]).
//! `fields` is only there under `validation_failed`: what each field of the
//! body does not hold.
//!
//! The status says whose doing it is: 400 for a body that is not one, 401
//! for a request that no longer signs anyone in (`sign_in_required` without
//! a token, `signed_out` with one that stopped working: the client forgets
//! it), 403 for a role the user does not have, 404 and 405 for an address
//! or a method the api does not serve, 409 for what no longer fits the
//! state of things, 422 for what the person can correct (a field, a wrong
//! code), 429 for what is asked again too soon and 503 for a store that is
//! not taking writes for the moment. Anything else is a 500 that says
//! nothing, logged here.

use std::borrow::Cow;
use std::collections::BTreeMap;

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, FromRequestParts, Query, Request};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use evento::WriteError;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use timada_core::page::Cursor;
use timada_identity::IdentityError;
use utoipa::ToSchema;
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

use crate::locale::Locale;

/// The request carries no token.
pub const SIGN_IN_REQUIRED: &str = "sign_in_required";
/// The token the request carries no longer signs anyone in.
pub const SIGNED_OUT: &str = "signed_out";
pub const NOT_ALLOWED: &str = "not_allowed";
pub const INVALID_REQUEST: &str = "invalid_request";
/// Fields of the body do not hold what the command asks: see `fields`.
pub const VALIDATION_FAILED: &str = "validation_failed";
/// What was asked no longer fits the account as it stands (the address
/// changed hands while its code was on the way, say): start again.
pub const INVALID_STATE: &str = "invalid_state";
pub const NOT_FOUND: &str = "not_found";
pub const METHOD_NOT_ALLOWED: &str = "method_not_allowed";
pub const INTERNAL: &str = "internal";

#[derive(Debug)]
pub enum ApiError {
    Refused(StatusCode, &'static str),
    /// A body whose fields do not hold what the command asks.
    Invalid(Vec<FieldError>),
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

impl From<WriteError> for ApiError {
    fn from(err: WriteError) -> Self {
        match timada_core::write_refusal(&err) {
            // The store's doing, not the request's: sent again, it very
            // likely goes through.
            Some(code) => Self::Refused(StatusCode::SERVICE_UNAVAILABLE, code),
            None => Self::Internal(err.into()),
        }
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        let mut fields = Vec::new();
        flatten("", &errors, &mut fields);
        fields.sort_by(|a, b| (&a.field, &a.code).cmp(&(&b.field, &b.code)));
        Self::Invalid(fields)
    }
}

/// The errors of `errors` as one list, each under the path of its field
/// from the body: `name`, `address.city`, `ingredients[2].quantity`.
fn flatten(prefix: &str, errors: &ValidationErrors, into: &mut Vec<FieldError>) {
    for (name, kind) in errors.errors() {
        let path = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix}.{name}")
        };
        match kind {
            ValidationErrorsKind::Field(errors) => {
                into.extend(errors.iter().map(|error| FieldError::new(&path, error)));
            }
            ValidationErrorsKind::Struct(inner) => flatten(&path, inner, into),
            ValidationErrorsKind::List(items) => {
                for (index, inner) in items {
                    flatten(&format!("{path}[{index}]"), inner, into);
                }
            }
        }
    }
}

/// What one field of a body does not hold.
#[derive(Debug, Clone)]
pub struct FieldError {
    pub field: String,
    pub code: Cow<'static, str>,
    /// The bounds the value is held to (`min`, `max`), for its sentence.
    pub params: BTreeMap<String, Value>,
}

impl FieldError {
    fn new(field: &str, error: &ValidationError) -> Self {
        let params: BTreeMap<String, Value> = error
            .params
            .iter()
            // What was sent is not said back: it may be a password.
            .filter(|(name, _)| *name != "value")
            .map(|(name, value)| (name.to_string(), value.clone()))
            .collect();
        let bounds = (params.contains_key("min"), params.contains_key("max"));
        let code = match (error.code.as_ref(), bounds) {
            ("length", (true, false)) => "too_short".into(),
            ("length", (false, true)) => "too_long".into(),
            ("range", (true, false)) => "too_small".into(),
            ("range", (false, true)) => "too_large".into(),
            _ => error.code.clone(),
        };
        Self {
            field: field.to_owned(),
            code,
            params,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let refusal = match self {
            Self::Refused(status, code) => Refusal {
                status,
                code,
                fields: Vec::new(),
            },
            Self::Invalid(fields) => Refusal {
                status: StatusCode::UNPROCESSABLE_ENTITY,
                code: VALIDATION_FAILED,
                fields,
            },
            Self::Internal(err) => {
                tracing::error!(err = format!("{err:#}"), "request failed");
                Refusal {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: INTERNAL,
                    fields: Vec::new(),
                }
            }
        };
        let mut response = refusal.render(Locale::default());
        // For `locale::localize` to word it in the language of the request.
        response.extensions_mut().insert(refusal);
        response
    }
}

/// A refusal on its way out, before it is worded.
#[derive(Debug, Clone)]
pub struct Refusal {
    status: StatusCode,
    code: &'static str,
    fields: Vec<FieldError>,
}

impl Refusal {
    pub fn render(&self, locale: Locale) -> Response {
        let message = locale
            .sentence(&format!("errors.{}", self.code))
            .or_else(|| locale.sentence("errors.refused"))
            .unwrap_or_default();
        let fields = self
            .fields
            .iter()
            .map(|field| ErrorField {
                field: field.field.clone(),
                code: field.code.to_string(),
                message: locale
                    .sentence_with(&format!("fields.{}", field.code), &field.params)
                    .or_else(|| locale.sentence("fields.invalid").map(Cow::into_owned))
                    .unwrap_or_default(),
                params: field.params.clone(),
            })
            .collect();
        let body = ErrorBody {
            error: ErrorDetail {
                code: self.code.to_owned(),
                message: message.into_owned(),
                fields,
            },
        };
        (self.status, Json(body)).into_response()
    }
}

/// What every refusal answers.
#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorDetail {
    /// A stable word for the refusal, for the client to act on.
    #[schema(example = "validation_failed")]
    pub code: String,
    /// The refusal said to a person, in the language of `Accept-Language`.
    #[schema(example = "Some fields are not valid.")]
    pub message: String,
    /// Under `validation_failed`: what each field of the body does not hold.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<ErrorField>,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorField {
    /// The path of the field from the body: `name`, `ingredients[2].quantity`.
    #[schema(example = "name")]
    pub field: String,
    #[schema(example = "too_long")]
    pub code: String,
    #[schema(example = "This must be at most 255 characters.")]
    pub message: String,
    /// The bounds the value is held to, when it has some.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[schema(example = json!({ "max": 255 }))]
    pub params: BTreeMap<String, Value>,
}

/// The JSON body of a command, read and checked: refused the way everything
/// else is when it is not JSON, not of the command's shape, or when its
/// fields do not hold what the command asks.
pub struct Body<T>(pub T);

impl<S, T> FromRequest<S> for Body<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(body)) => {
                body.validate()?;
                Ok(Self(body))
            }
            Err(JsonRejection::JsonDataError(_)) => Err(ApiError::refused(
                StatusCode::UNPROCESSABLE_ENTITY,
                INVALID_REQUEST,
            )),
            Err(_) => Err(ApiError::refused(StatusCode::BAD_REQUEST, INVALID_REQUEST)),
        }
    }
}

/// What a read is asked with after the `?` of its address, read and checked
/// the way a [`Body`] is: refused when it cannot be read as the read's
/// parameters, or when they do not hold what the read asks.
pub struct Params<T>(pub T);

impl<S, T> FromRequestParts<S> for Params<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Ok(Query(params)) = Query::<T>::from_request_parts(parts, state).await else {
            return Err(ApiError::refused(StatusCode::BAD_REQUEST, INVALID_REQUEST));
        };
        params.validate()?;
        Ok(Self(params))
    }
}

/// Reads a text without the blanks around it (`#[serde(deserialize_with)]`),
/// so that its length is checked on what is kept.
pub fn trimmed<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(String::deserialize(deserializer)?.trim().to_owned())
}

/// Reads a text that may be left out, without the blanks around it
/// (`#[serde(default, deserialize_with)]`): nothing but blanks is left out
/// too.
pub fn trimmed_opt<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<String>::deserialize(deserializer)?
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty()))
}

/// Reads a field left out or set to `null` as its default
/// (`#[serde(default, deserialize_with)]`): a list as an empty one.
pub fn or_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

/// Holds the `after` of a list to what a `next` reads as
/// (`#[validate(custom)]`).
pub fn valid_cursor(after: &str) -> Result<(), ValidationError> {
    Cursor::parse(after)
        .map(drop)
        .ok_or(ValidationError::new("invalid_cursor"))
}

/// What an address the api does not serve answers.
pub async fn not_found() -> ApiError {
    ApiError::refused(StatusCode::NOT_FOUND, NOT_FOUND)
}

/// What a method an address does not take answers.
pub async fn method_not_allowed() -> ApiError {
    ApiError::refused(StatusCode::METHOD_NOT_ALLOWED, METHOD_NOT_ALLOWED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Validate)]
    struct Ingredient {
        #[validate(range(min = 1))]
        quantity: u32,
    }

    #[derive(Validate)]
    struct Recipe {
        #[validate(length(min = 1, max = 5))]
        name: String,
        #[validate(length(max = 3))]
        note: String,
        #[validate(nested)]
        ingredients: Vec<Ingredient>,
    }

    #[test]
    fn the_fields_are_listed_by_path_with_their_bounds() {
        let recipe = Recipe {
            name: String::new(),
            note: "too long".to_owned(),
            ingredients: vec![Ingredient { quantity: 2 }, Ingredient { quantity: 0 }],
        };
        let ApiError::Invalid(fields) = recipe.validate().unwrap_err().into() else {
            panic!("not a validation refusal");
        };
        let listed: Vec<_> = fields
            .iter()
            .map(|field| {
                (
                    field.field.as_str(),
                    field.code.as_ref(),
                    serde_json::to_value(&field.params).unwrap(),
                )
            })
            .collect();
        assert_eq!(
            listed,
            [
                (
                    "ingredients[1].quantity",
                    "too_small",
                    serde_json::json!({ "min": 1 })
                ),
                ("name", "length", serde_json::json!({ "min": 1, "max": 5 })),
                ("note", "too_long", serde_json::json!({ "max": 3 })),
            ]
        );
    }

    #[test]
    fn every_code_of_the_api_has_its_sentence() {
        for code in [
            SIGN_IN_REQUIRED,
            SIGNED_OUT,
            NOT_ALLOWED,
            INVALID_REQUEST,
            VALIDATION_FAILED,
            INVALID_STATE,
            NOT_FOUND,
            METHOD_NOT_ALLOWED,
            INTERNAL,
            crate::account::TOO_MANY_SIGN_INS,
            crate::recipes::RECIPE_NOT_FOUND,
            crate::recipes::PAGE_UNREACHABLE,
            crate::recipes::NO_RECIPE_ON_PAGE,
            crate::plats::PLAT_NOT_FOUND,
            crate::list::LIST_ENTRY_NOT_FOUND,
            crate::list::LIST_FULL,
            crate::list::NOTHING_TO_GENERATE,
            crate::images::UNSUPPORTED_IMAGE,
            crate::images::IMAGE_TOO_LARGE,
            crate::images::IMAGE_TOO_MANY_PIXELS,
            timada_core::WRITES_PAUSED,
            timada_core::WRITE_NOT_TAKEN,
            "refused",
        ] {
            assert!(
                Locale::En.sentence(&format!("errors.{code}")).is_some(),
                "{code}"
            );
        }
    }
}
