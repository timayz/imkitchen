//! The reference of the api, generated from its handlers.
//!
//! A handler is described where it is written (`#[utoipa::path]`) and the
//! router only takes described handlers (`routes!`), so the document says
//! what the api serves and nothing else. `GET /v1/openapi.json` answers it,
//! `GET /v1/docs` shows it.

use std::sync::Arc;

use axum::Router;
use axum::http::header::CONTENT_TYPE;
use axum::response::Html;
use axum::routing::get;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::openapi::{ContentBuilder, Ref, ResponseBuilder};
use utoipa::{Modify, OpenApi};
use utoipa_scalar::Scalar;

use crate::error::ErrorBody;
use crate::server::AppState;

/// The version every endpoint of the api lives under.
pub const VERSION: &str = "/v1";

pub const AUTH: &str = "auth";
pub const ACCOUNT: &str = "account";
pub const ADMIN: &str = "admin";
pub const NODE: &str = "node";

/// The security scheme of the endpoints that need someone signed in, as
/// their `security(("session" = []))` names it.
const SESSION: &str = "session";

#[derive(OpenApi)]
#[openapi(
    info(title = "imkitchen api", description = include_str!("../README.md")),
    tags(
        (name = AUTH, description = "Creating an account, signing in and out"),
        (name = ACCOUNT, description = "What a user does with their own account"),
        (name = ADMIN, description = "What an administrator does with the accounts of others"),
        (name = NODE, description = "The probes of the node; no version, not for clients"),
    ),
    components(schemas(ErrorBody)),
    modifiers(&SessionToken)
)]
pub struct Document;

struct SessionToken;

impl Modify for SessionToken {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi
            .components
            .get_or_insert_default()
            .add_security_scheme(
                SESSION,
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            );
    }
}

/// Says of every versioned endpoint what it answers when it refuses: the
/// one shape of [`crate::error`], whatever the status.
pub fn describe_refusals(openapi: &mut utoipa::openapi::OpenApi) {
    let refusal = ResponseBuilder::new()
        .description("A refusal: the status says whose doing it is, the body what and why")
        .content(
            "application/json",
            ContentBuilder::new()
                .schema(Some(Ref::from_schema_name("ErrorBody")))
                .build(),
        )
        .build();
    for (path, item) in openapi.paths.paths.iter_mut() {
        if !path.starts_with(VERSION) {
            continue;
        }
        for operation in [
            &mut item.get,
            &mut item.put,
            &mut item.post,
            &mut item.delete,
        ]
        .into_iter()
        .flatten()
        {
            operation
                .responses
                .responses
                .insert("default".to_owned(), refusal.clone().into());
        }
    }
}

/// The routes the document and its page are served on.
pub fn routes(openapi: utoipa::openapi::OpenApi) -> Router<AppState> {
    // Both are the same for as long as the node runs: written once.
    let json: Arc<str> = openapi
        .to_json()
        .expect("the OpenAPI document is JSON")
        .into();
    let page: Arc<str> = Scalar::new(openapi).title("imkitchen api").to_html().into();

    Router::new()
        .route(
            &format!("{VERSION}/openapi.json"),
            get(move || async move { ([(CONTENT_TYPE, "application/json")], json.to_string()) }),
        )
        .route(
            &format!("{VERSION}/docs"),
            get(move || async move { Html(page.to_string()) }),
        )
}
