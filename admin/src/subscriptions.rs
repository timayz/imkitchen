//! What admin does on its own time, following the events of the api.

use anyhow::{Context, Result};
use evento::subscription::Subscription;

use crate::db::Db;
use crate::events::Executor;

/// Starts every subscription of admin over `executor`, the read models
/// written to `db`. Each starts whether the api answers or not, and handles
/// its first event once it does. Keep the handles and hand them to
/// [`shutdown`].
pub async fn start(executor: &Executor, db: &Db) -> Result<Vec<Subscription>> {
    // The recipes and the pictures: each row the snapshot of a projection.
    let mut subscriptions = vec![
        crate::recipes::subscription(db.clone())
            .supervised()
            .start(executor)
            .await
            .context("cannot start the recipes subscription")?,
        crate::images::subscription(db.clone())
            .supervised()
            .start(executor)
            .await
            .context("cannot start the images subscription")?,
    ];
    // The users, whose rows several streams make, and the imports and the
    // messages, a row for each event.
    for (what, subscription) in [
        ("users", crate::users::subscription(db.clone())),
        ("imports", crate::imports::subscription(db.clone())),
        ("messages", crate::messages::subscription(db.clone())),
    ] {
        subscriptions.push(
            subscription
                .supervised()
                .start(executor)
                .await
                .with_context(|| format!("cannot start the {what} subscription"))?,
        );
    }
    Ok(subscriptions)
}

/// Stops them, each once the event it is on is handled.
pub async fn shutdown(subscriptions: Vec<Subscription>) {
    for subscription in subscriptions {
        if let Err(err) = subscription.shutdown().await {
            tracing::warn!(%err, "a subscription did not stop cleanly");
        }
    }
}
