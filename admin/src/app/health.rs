//! `GET /health`, for the probes: admin is up and answers HTTP.

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
    /// How admin stands with the api's events: `connected`, `connecting` or
    /// `refused`. admin serves what its store holds either way.
    api: &'static str,
}

#[route(GET)]
async fn health(cx: &Cx) -> Result<Json<Health>> {
    Ok(Json(Health {
        status: "ok",
        api: context::<Events>(cx).state(),
    }))
}
