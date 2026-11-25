use std::time::Duration;

use anyhow::Result;
use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::health::health;

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http())
}

/// Serves until `shutdown` resolves, then stops accepting connections and
/// gives the requests in flight `shutdown_timeout` to finish.
pub async fn serve(
    listener: TcpListener,
    shutdown_timeout: Duration,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<()> {
    let (stopping_tx, stopping_rx) = tokio::sync::oneshot::channel::<()>();
    let server = axum::serve(listener, router()).with_graceful_shutdown(async move {
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
