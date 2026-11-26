use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::account::SignInAttempts;
use crate::auth::Auth;
use crate::cluster::{Cluster, Executor};
use crate::db::Db;
use crate::error::{method_not_allowed, not_found};
use crate::health::{__path_health, __path_ready, health, ready};
use crate::images::Images;
use crate::locale::localize;
use crate::openapi::{self, Document, VERSION};
use crate::recipes::Pages;

#[derive(Clone)]
pub struct AppState {
    /// This node of the cluster, for what it says about itself.
    pub cluster: Arc<Cluster>,
    /// The event store: writes go through the cluster, reads come from this
    /// node's file.
    pub executor: Executor,
    /// The read models of this node: what the lists are read from.
    pub db: Db,
    pub auth: Arc<Auth>,
    /// The password sign-ins tried on this node, per address.
    pub sign_ins: Arc<SignInAttempts>,
    /// The pages of the web recipes are imported from.
    pub pages: Arc<dyn Pages>,
    /// The files of the recipes' pictures, on this node and the others.
    pub images: Arc<Images>,
}

/// Every route of the api: the probes of the node, then the commands and
/// reads under their version, each described in the document served next to
/// them (see `api/README.md` for what they all follow).
pub fn router(state: AppState) -> Router {
    let (routes, mut document) = OpenApiRouter::with_openapi(Document::openapi())
        .routes(routes!(health))
        .routes(routes!(ready))
        .nest(
            VERSION,
            OpenApiRouter::new()
                .merge(crate::account::routes())
                .merge(crate::admin::routes())
                .merge(crate::recipes::routes())
                .merge(crate::plats::routes())
                .merge(crate::settings::routes())
                .merge(crate::list::routes())
                .merge(crate::groceries::routes())
                .merge(crate::tutorial::routes())
                .merge(crate::contact::routes())
                .merge(crate::images::routes()),
        )
        .split_for_parts();
    openapi::describe_refusals(&mut document);

    routes
        .merge(openapi::routes(document))
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .layer(axum::middleware::from_fn(localize))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Serves until `shutdown` resolves, then stops accepting connections and
/// gives the requests in flight `shutdown_timeout` to finish.
pub async fn serve(
    listener: TcpListener,
    state: AppState,
    shutdown_timeout: Duration,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<()> {
    let (stopping_tx, stopping_rx) = tokio::sync::oneshot::channel::<()>();
    let server = axum::serve(listener, router(state)).with_graceful_shutdown(async move {
        shutdown.await;
        tracing::info!("shutting down");
        let _ = stopping_tx.send(());
    });

    tokio::select! {
        served = server => served?,
        _ = async {
            let _ = stopping_rx.await;
            tokio::time::sleep(shutdown_timeout).await;
        } => {
            tracing::warn!(
                "requests still in flight after {}s, exiting anyway",
                shutdown_timeout.as_secs()
            );
        }
    }

    Ok(())
}

/// Resolves on Ctrl+C or SIGTERM.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
