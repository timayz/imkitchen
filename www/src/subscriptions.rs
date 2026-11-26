//! What www does on its own time, following the events of the api.

use anyhow::{Context, Result};
use evento::subscription::Subscription;

use crate::db::Db;
use crate::events::Executor;

/// Starts every subscription of www over `executor`, the read models
/// written to `db`. Each starts whether the api answers or not, and handles
/// its first event once it does. Keep the handles and hand them to
/// [`shutdown`].
pub async fn start(executor: &Executor, db: &Db) -> Result<Vec<Subscription>> {
    Ok(vec![
        crate::recipes::subscription(db.clone())
            .supervised()
            .start(executor)
            .await
            .context("cannot start the recipes subscription")?,
    ])
}

/// Stops them, each once the event it is on is handled.
pub async fn shutdown(subscriptions: Vec<Subscription>) {
    for subscription in subscriptions {
        if let Err(err) = subscription.shutdown().await {
            tracing::warn!(%err, "a subscription did not stop cleanly");
        }
    }
}
