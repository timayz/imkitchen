//! The node: the one member of the evento cluster this process runs, over
//! its own store.
//!
//! Every api instance holds every event in a file of its own. A write is
//! agreed on by a majority of the instances before it returns (leaderless,
//! strictly serializable), so several of them write at once without losing
//! or reordering events; reads and subscriptions are served from this
//! node's file. Without a `[cluster]` section the node talks to itself in
//! memory, so the same code runs one instance and several.
//!
//! A node that was away for too long copies its store back from a peer at
//! its next start. Its read models (`db::reset_read_models`) are emptied
//! first, and its subscriptions fill them again from the copied events.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use evento::cluster::Settings;
pub use evento::cluster::{Cluster, NodeHealth};

use crate::db::Store;

/// What commands write through and aggregates are loaded from: a drop-in
/// `evento::Executor`.
pub type Executor = evento::ClusterExecutor<Store>;

/// Opens the node over `store`, the file at `store_path`, and starts it.
/// With peers it returns once the node opened its journal; it takes writes
/// once a majority of them confirmed it ([`Cluster::until_confirmed`]).
pub async fn open(
    settings: Option<Settings>,
    store_path: &Path,
    store: Store,
) -> Result<(Arc<Cluster>, Executor)> {
    // Derived from the events, in the same file: emptied with them, filled
    // again by the subscriptions from the copy.
    let read_models = store.clone();
    Cluster::builder(store)
        .settings(settings)
        .state_dir(state_dir(store_path))
        .before_copy(move || async move {
            crate::db::reset_read_models(&read_models)
                .await
                .context("cannot empty the read models before copying the store")
        })
        .open()
        .await
}

/// Where the node keeps what it knows about its store (the mark that has it
/// copy the store back from a peer at the next start): next to the file.
fn state_dir(store_path: &Path) -> PathBuf {
    match store_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}
