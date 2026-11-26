//! `GET /health`, for the probes: www is up and answers HTTP.

use serde::Serialize;
use topcoat::{
    Result,
    context::{Cx, context},
    router::{content::Json, route},
};

use crate::events::Events;

#[derive(Serialize)]
pub struct Health {
    status: &'static str,
    /// How www stands with the api's events: `connected`, `connecting` or
    /// `refused`. www serves what its store holds either way.
    api: &'static str,
}

#[route(GET)]
async fn health(cx: &Cx) -> Result<Json<Health>> {
    Ok(Json(Health {
        status: "ok",
        api: context::<Events>(cx).state(),
    }))
}
