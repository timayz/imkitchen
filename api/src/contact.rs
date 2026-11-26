//! `/v1/contact-messages`: what is written to imkitchen.
//!
//! Anyone writes: a visitor of www, through its contact form, has no
//! account, so the route takes no [`crate::auth::Session`]. A message is
//! an aggregate of its own, written once. The api keeps no list of them:
//! reading them is for admin, from the events.

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use imkitchen_aggregates::contact::Sent;
use serde::{Deserialize, Serialize};
use timada_identity::normalize_email;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use validator::Validate;

use crate::account::valid_email;
use crate::error::{ApiResult, Body, trimmed};
use crate::locale::Locale;
use crate::openapi::CONTACT;
use crate::server::AppState;

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(send))
}

/// What someone writes to imkitchen.
#[derive(Deserialize, Validate, ToSchema)]
struct Message {
    /// Who writes, as they name themselves.
    #[serde(deserialize_with = "trimmed")]
    #[validate(length(min = 1, max = 100))]
    #[schema(example = "Alice")]
    name: String,
    /// Where they are answered.
    #[validate(custom(function = "valid_email"))]
    #[schema(example = "alice@example.com")]
    email: String,
    #[serde(deserialize_with = "trimmed")]
    #[validate(length(min = 1, max = 4000))]
    message: String,
}

/// A message that was taken.
#[derive(Serialize, ToSchema)]
struct MessageSent {
    id: String,
}

/// Writes to imkitchen
///
/// Needs nobody signed in. The message is kept with the language the
/// request asks for (`Accept-Language`), the one to answer in. Nothing
/// proves the address is the sender's: no code is mailed.
#[utoipa::path(
    post,
    path = "/contact-messages",
    tag = CONTACT,
    request_body = Message,
    responses((status = CREATED, description = "The message is taken", body = MessageSent))
)]
async fn send(
    State(state): State<AppState>,
    headers: HeaderMap,
    Body(message): Body<Message>,
) -> ApiResult<(StatusCode, Json<MessageSent>)> {
    let mut write = evento::create();
    write.event(&Sent {
        name: message.name,
        email: normalize_email(&message.email)?,
        message: message.message,
        language: Locale::from_headers(&headers).as_str().to_owned(),
    });
    let id = write.commit(&state.executor).await?;
    Ok((StatusCode::CREATED, Json(MessageSent { id })))
}
