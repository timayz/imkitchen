//! What the node does on its own time, following the events of its store.

use std::sync::Arc;

use anyhow::{Context, Result};
use evento::subscription::Subscription;

use crate::auth::Auth;
use crate::cluster::Executor;
use crate::db::Db;
use crate::images::Images;
use crate::mail::{Mailer, code_mails_subscription};

/// Starts every subscription of the api over `executor`, the read models'
/// writing to `db` and the pictures' files kept in `images`. `node` names this node among those of the cluster, for
/// the claims it writes. Keep the handles and hand them to [`shutdown`].
pub async fn start(
    executor: &Executor,
    db: &Db,
    auth: &Auth,
    mailer: Arc<dyn Mailer>,
    images: Arc<Images>,
    node: &str,
) -> Result<Vec<Subscription>> {
    Ok(vec![
        // Frees the sign-in identities an account let go of, should the
        // command that removed them not have.
        timada_identity::index_release_subscription()
            .supervised()
            .start(executor)
            .await
            .context("cannot start the identity indexes subscription")?,
        code_mails_subscription(executor, mailer, auth.codes.clone(), node)
            .supervised()
            .start(executor)
            .await
            .context("cannot start the code mails subscription")?,
        // The lists' rows read this node's file as it stands: what they
        // load is behind the event they handle, and needs no round of the
        // cluster.
        crate::recipes::subscription(db.clone())
            .supervised()
            .start(&executor.local_reads())
            .await
            .context("cannot start the recipes subscription")?,
        crate::plats::subscription(db.clone())
            .supervised()
            .start(&executor.local_reads())
            .await
            .context("cannot start the plats subscription")?,
        // The recipes that were deleted taken out of the plats they were
        // part of: writes, so through the cluster.
        crate::plats::saga(db.clone())
            .supervised()
            .start(executor)
            .await
            .context("cannot start the plats saga")?,
        // The recipes and the plats that were deleted taken off the lists
        // they were on: writes, so through the cluster.
        crate::list::subscription()
            .supervised()
            .start(executor)
            .await
            .context("cannot start the list subscription")?,
        // The steps of the tutorials, noted as the users do them: writes,
        // so through the cluster.
        crate::tutorial::subscription()
            .supervised()
            .start(executor)
            .await
            .context("cannot start the tutorial subscription")?,
        // This node's copies of the pictures, from its own file too.
        crate::images::subscription(db.clone(), images)
            .supervised()
            .start(&executor.local_reads())
            .await
            .context("cannot start the images subscription")?,
        // Gives back the pictures the recipes let go of: writes, so through
        // the cluster.
        crate::images::release_subscription()
            .supervised()
            .start(executor)
            .await
            .context("cannot start the image release subscription")?,
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
