//! What the node does on its own time, following the events of its store.

use std::sync::Arc;

use anyhow::{Context, Result};
use evento::subscription::Subscription;

use crate::auth::Auth;
use crate::cluster::Executor;
use crate::mail::{Mailer, code_mails_subscription};

/// Starts every subscription of the api over `executor`. `node` names this
/// node among those of the cluster, for the claims it writes. Keep the
/// handles and hand them to [`shutdown`].
pub async fn start(
    executor: &Executor,
    auth: &Auth,
    mailer: Arc<dyn Mailer>,
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
