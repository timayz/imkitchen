use axum::Json;
use serde::Serialize;

use crate::ApiError;

#[derive(Serialize)]
pub struct Health {
    pub status: &'static str,
    pub version: &'static str,
}

pub async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Unknown `/api/v1` paths answer with the JSON envelope instead of the HTML
/// 404 page the rest of the site renders.
pub async fn not_found() -> ApiError {
    ApiError::NotFound
}
