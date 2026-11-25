use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct Health {
    status: &'static str,
}

/// Liveness: the process is up and answers HTTP.
pub(crate) async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}
