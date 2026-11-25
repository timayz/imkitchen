//! The harness aggregate, subscription and API tests run on: api nodes over
//! stores created the way a deployment creates them (`db::migrate`, then
//! `db::open`), alone or as a cluster talking over TCP on localhost.
#![allow(dead_code)]

use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use evento::cluster::Settings;
use imkitchen_api::auth::Auth;
use imkitchen_api::cluster::{self, Cluster, Executor};
use imkitchen_api::config::{AuthConfig, RootConfig, Secret};
use imkitchen_api::db::{self, Store};
use imkitchen_api::server::AppState;
use tempfile::TempDir;

/// How long a test waits for what the nodes do on their own time: confirming
/// each other, applying a write another node took, a subscription's pass.
pub const SETTLE: Duration = Duration::from_secs(30);

/// The root account of the tests' configuration.
pub const ROOT_EMAIL: &str = "root@example.com";
pub const ROOT_PASSWORD: &str = "root test password";
/// The argon2 hash of [`ROOT_PASSWORD`].
const ROOT_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$diXOV+PTSdOCfHj2wDAlMA$iVEeQUJsww/h81mHmrcmcJRWuZs2mY3bopNrchI/yog";

/// What signing in goes by on every test node: one secret, so the nodes of
/// a cluster check each other's codes, and the root account above.
pub fn auth(session_lifetime_secs: u64) -> Arc<Auth> {
    Arc::new(
        Auth::new(&AuthConfig {
            secret: Some(Secret::new("a secret of the tests, 32 characters")),
            session_lifetime_secs,
            root: Some(RootConfig {
                email: ROOT_EMAIL.to_owned(),
                password_hash: Secret::new(ROOT_PASSWORD_HASH),
            }),
        })
        .unwrap(),
    )
}

pub struct TestNode {
    pub cluster: Arc<Cluster>,
    /// Writes go through the cluster; with peers, a read of one aggregate
    /// waits for every write committed before it, whichever node took it.
    pub executor: Executor,
    /// This node's own file, as it stands: what the node holds itself.
    pub store: Store,
    pub store_path: PathBuf,
}

impl TestNode {
    /// A node alone over a fresh store. Keep the returned directory: the
    /// store is gone with it.
    pub async fn alone() -> (Self, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let node = Self::open(dir.path().join("imkitchen.db"), None).await;
        (node, dir)
    }

    async fn open(store_path: PathBuf, settings: Option<Settings>) -> Self {
        drop(db::migrate(&store_path).await.unwrap());
        let store = db::open(&store_path).await.unwrap();
        let (cluster, executor) = cluster::open(settings, &store_path, store.clone())
            .await
            .unwrap();

        Self {
            cluster,
            executor,
            store,
            store_path,
        }
    }

    pub fn state(&self) -> AppState {
        AppState {
            cluster: self.cluster.clone(),
            executor: self.executor.clone(),
            auth: auth(3600),
            sign_ins: Arc::default(),
        }
    }
}

/// Several api nodes replicating one store between them, each over a file
/// of its own.
pub struct TestCluster {
    pub nodes: Vec<TestNode>,
    _dir: TempDir,
}

impl TestCluster {
    /// Starts `size` nodes and waits until each takes writes.
    pub async fn start(size: usize) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let peers = free_addrs(size).await;

        let opens: Vec<_> = (0..size)
            .map(|id| {
                let store_path = dir.path().join(format!("node{id}")).join("imkitchen.db");
                let settings = Settings::new(id as u64, peers.clone());
                tokio::spawn(TestNode::open(store_path, Some(settings)))
            })
            .collect();
        let mut nodes = Vec::new();
        for open in opens {
            nodes.push(open.await.unwrap());
        }
        for node in &nodes {
            node.cluster.until_confirmed(SETTLE).await.unwrap();
        }

        Self { nodes, _dir: dir }
    }

    pub async fn shutdown(self) {
        for node in &self.nodes {
            node.cluster.shutdown().await;
        }
    }
}

/// Addresses nothing listens on, for the nodes to talk to each other.
async fn free_addrs(count: usize) -> Vec<SocketAddr> {
    let mut listeners = Vec::new();
    for _ in 0..count {
        listeners.push(tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap());
    }
    listeners
        .iter()
        .map(|listener| listener.local_addr().unwrap())
        .collect()
}

/// Polls `check` until it answers `Some`, for what settles on the nodes' own
/// time. Panics with `what` after [`SETTLE`].
pub async fn eventually<T, F, Fut>(what: &str, check: F) -> T
where
    F: Fn() -> Fut,
    Fut: Future<Output = Option<T>>,
{
    let started = tokio::time::Instant::now();
    loop {
        if let Some(value) = check().await {
            return value;
        }
        assert!(started.elapsed() < SETTLE, "timed out waiting for {what}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}
