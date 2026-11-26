use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;
use utoipa::ToSchema;

use crate::openapi::NODE;

use crate::server::AppState;

#[derive(Serialize, ToSchema)]
pub(crate) struct Health {
    status: &'static str,
}

/// Liveness: the process is up and answers HTTP
#[utoipa::path(get, path = "/health", tag = NODE, responses((status = OK, body = Health)))]
pub(crate) async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

#[derive(Serialize, ToSchema)]
pub(crate) struct Ready {
    status: &'static str,
    node: u64,
    /// The members of the cluster, this node included.
    nodes: usize,
    member: bool,
    confirmed: bool,
    /// The node is to copy its store back from a peer: restart it.
    restart_required: bool,
}

/// Readiness: this node takes writes
///
/// A node whose peers have not confirmed it yet, that was removed from the
/// cluster, or that fell too far behind answers 503 so a balancer leaves it
/// out while the others serve.
#[utoipa::path(
    get,
    path = "/ready",
    tag = NODE,
    responses(
        (status = OK, body = Ready),
        (status = SERVICE_UNAVAILABLE, description = "The node does not take writes", body = Ready),
    )
)]
pub(crate) async fn ready(State(state): State<AppState>) -> (StatusCode, Json<Ready>) {
    let node = state.cluster.health();
    let serving = node.serving();

    (
        if serving {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        Json(Ready {
            status: if serving { "ok" } else { "unavailable" },
            node: node.id.0,
            nodes: node.nodes,
            member: node.member,
            confirmed: node.confirmed,
            restart_required: node.rejoin_required || node.rolled_back || node.copy_pending,
        }),
    )
}
