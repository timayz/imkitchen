//! The harness aggregate, subscription and API tests run on: api nodes over
//! stores created the way a deployment creates them (`db::migrate`, then
//! `db::open`), alone or as a cluster talking over TCP on localhost.
#![allow(dead_code)]

pub mod api;

use std::collections::HashMap;
use std::future::Future;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use evento::cluster::Settings;
use imkitchen_api::auth::Auth;
use imkitchen_api::cluster::{self, Cluster, Executor, NodeTls};
use imkitchen_api::config::{AuthConfig, RootConfig, Secret};
use imkitchen_api::db::{self, Db, Store};
use imkitchen_api::images::port::{self, PeerPort};
use imkitchen_api::images::{Images, Peers};
use imkitchen_api::recipes::{PageError, Pages};
use imkitchen_api::server::AppState;
use tempfile::TempDir;
use tokio::net::TcpListener;

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

/// The web of the tests: the pages and the pictures put on it. Nothing
/// answers anywhere else.
#[derive(Default)]
pub struct TestWeb {
    pages: Mutex<HashMap<String, Result<String, PageError>>>,
    images: Mutex<HashMap<String, Bytes>>,
}

impl TestWeb {
    /// Answers `url` with the page `html`.
    pub fn put(&self, url: &str, html: &str) {
        self.answer(url, Ok(html.to_owned()));
    }

    pub fn answer(&self, url: &str, answer: Result<String, PageError>) {
        self.pages.lock().unwrap().insert(url.to_owned(), answer);
    }

    /// Answers `url` with `bytes`, as a site serves a picture.
    pub fn put_image(&self, url: &str, bytes: &[u8]) {
        let bytes = Bytes::copy_from_slice(bytes);
        self.images.lock().unwrap().insert(url.to_owned(), bytes);
    }
}

#[async_trait::async_trait]
impl Pages for TestWeb {
    async fn fetch(&self, url: &reqwest::Url) -> Result<String, PageError> {
        let nothing = || Err(PageError::Unreachable("nothing answers there".to_owned()));
        let pages = self.pages.lock().unwrap();
        pages.get(url.as_str()).cloned().unwrap_or_else(nothing)
    }

    async fn image(&self, url: &reqwest::Url) -> Result<Bytes, PageError> {
        let nothing = || PageError::Unreachable("nothing answers there".to_owned());
        let images = self.images.lock().unwrap();
        images.get(url.as_str()).cloned().ok_or_else(nothing)
    }
}

pub struct TestNode {
    pub cluster: Arc<Cluster>,
    /// Writes go through the cluster; with peers, a read of one aggregate
    /// waits for every write committed before it, whichever node took it.
    pub executor: Executor,
    /// This node's own file, as it stands: what the node holds itself.
    pub store: Store,
    pub store_path: PathBuf,
    /// Where this node imports recipes from.
    pub web: Arc<TestWeb>,
    /// The pictures this node keeps, in `images_dir`.
    pub images: Arc<Images>,
    pub images_dir: PathBuf,
    /// Where the other nodes copy pictures to this one, when there are any.
    images_port: Option<PeerPort>,
}

/// A node's place among those that copy pictures to each other: where it
/// takes theirs, and where each of them does, itself included.
struct ImagePeers {
    listener: TcpListener,
    urls: Vec<String>,
}

impl TestNode {
    /// A node alone over a fresh store. Keep the returned directory: the
    /// store is gone with it.
    pub async fn alone() -> (Self, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let node = Self::open(dir.path().join("imkitchen.db"), None, None, None).await;
        (node, dir)
    }

    async fn open(
        store_path: PathBuf,
        settings: Option<Settings>,
        tls: Option<NodeTls>,
        image_peers: Option<ImagePeers>,
    ) -> Self {
        drop(db::migrate(&store_path).await.unwrap());
        let store = db::open(&store_path).await.unwrap();
        let images_dir = store_path.with_file_name("images");
        let node_id = settings.as_ref().map(|settings| settings.node_id as usize);
        let (cluster, executor) = cluster::open(settings, tls.clone(), &store_path, store.clone())
            .await
            .unwrap();
        let (images, images_port) = match (image_peers, node_id) {
            (Some(ImagePeers { listener, mut urls }), Some(node_id)) => {
                let nodes = urls.len();
                urls.remove(node_id);
                let peers = Peers::new(urls, nodes, tls.clone()).unwrap();
                let images = Arc::new(Images::new(&images_dir, Some(peers)));
                let members = tls.map(|tls| (tls, cluster.clone()));
                let port = port::serve(listener, members, images.clone());
                (images, Some(port))
            }
            _ => (Arc::new(Images::new(&images_dir, None)), None),
        };

        Self {
            cluster,
            executor,
            store,
            store_path,
            web: Arc::default(),
            images,
            images_dir,
            images_port,
        }
    }

    /// Closes the port the other nodes copy pictures to this one on.
    pub async fn stop_taking_pictures(&mut self) {
        if let Some(port) = self.images_port.take() {
            port.shutdown().await;
        }
    }

    /// How many pictures this node's read model knows the files of.
    pub async fn images_known(&self) -> i64 {
        Db::from(self.store.clone())
            .fetch_scalar("SELECT COUNT(*) FROM images", ())
            .await
            .unwrap()
    }

    /// The files this node keeps of the pictures, as `<size>/<name>`, in
    /// order.
    pub fn image_files(&self) -> Vec<String> {
        let mut files = Vec::new();
        for size in ["400", "1200"] {
            let Ok(entries) = std::fs::read_dir(self.images_dir.join(size)) else {
                continue;
            };
            for entry in entries {
                let name = entry.unwrap().file_name();
                files.push(format!("{size}/{}", name.to_string_lossy()));
            }
        }
        files.sort();
        files
    }

    pub fn state(&self) -> AppState {
        AppState {
            cluster: self.cluster.clone(),
            executor: self.executor.clone(),
            db: Db::from(self.store.clone()),
            auth: auth(3600),
            sign_ins: Arc::default(),
            pages: self.web.clone(),
            images: self.images.clone(),
        }
    }
}

/// Several api nodes replicating one store between them, each over a file
/// of its own.
pub struct TestCluster {
    pub nodes: Vec<TestNode>,
    /// Where each node takes the pictures of the others.
    pub image_urls: Vec<String>,
    _dir: TempDir,
}

impl TestCluster {
    /// Starts `size` nodes and waits until each takes writes.
    pub async fn start(size: usize) -> Self {
        Self::start_with(size, None).await
    }

    /// The same, the nodes talking over mutual TLS with certificates of
    /// `authority`: to the store's port and to the pictures'.
    pub async fn start_over_tls(size: usize, authority: &Authority) -> Self {
        Self::start_with(size, Some(authority)).await
    }

    async fn start_with(size: usize, authority: Option<&Authority>) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let peers = free_addrs(size);
        // Where each node takes the pictures of the others, open from the
        // start: nothing else may take the address meanwhile.
        let mut image_listeners = Vec::new();
        for _ in 0..size {
            image_listeners.push(TcpListener::bind("127.0.0.1:0").await.unwrap());
        }
        let scheme = if authority.is_some() { "https" } else { "http" };
        let image_urls: Vec<String> = image_listeners
            .iter()
            .map(|listener| format!("{scheme}://{}", listener.local_addr().unwrap()))
            .collect();

        let opens: Vec<_> = image_listeners
            .into_iter()
            .enumerate()
            .map(|(id, listener)| {
                let store_path = dir.path().join(format!("node{id}")).join("imkitchen.db");
                let settings = Settings::new(id as u64, peers.clone());
                let image_peers = ImagePeers {
                    listener,
                    urls: image_urls.clone(),
                };
                // Every node is at 127.0.0.1, which its certificate names.
                let tls = authority.map(|authority| authority.node("127.0.0.1"));
                tokio::spawn(TestNode::open(
                    store_path,
                    Some(settings),
                    tls,
                    Some(image_peers),
                ))
            })
            .collect();
        let mut nodes = Vec::new();
        for open in opens {
            nodes.push(open.await.unwrap());
        }
        for node in &nodes {
            node.cluster.until_confirmed(SETTLE).await.unwrap();
        }

        Self {
            nodes,
            image_urls,
            _dir: dir,
        }
    }

    pub async fn shutdown(self) {
        for node in self.nodes {
            if let Some(port) = node.images_port {
                port.shutdown().await;
            }
            node.cluster.shutdown().await;
        }
    }
}

/// The authority of a test cluster: what issues its nodes' certificates.
pub struct Authority {
    params: rcgen::CertificateParams,
    key: rcgen::KeyPair,
    pem: String,
}

impl Authority {
    pub fn new() -> Self {
        let key = rcgen::KeyPair::generate().unwrap();
        let mut params = rcgen::CertificateParams::new(Vec::<String>::new()).unwrap();
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        let pem = params.self_signed(&key).unwrap().pem();
        Self { params, key, pem }
    }

    /// A node's side of TLS, with a certificate for `name`: a host, or an
    /// address.
    pub fn node(&self, name: &str) -> NodeTls {
        self.node_trusting(self, name)
    }

    /// The same, for a node that takes the certificates `trusted` issues
    /// for its peers' while its own is of this authority.
    pub fn node_trusting(&self, trusted: &Authority, name: &str) -> NodeTls {
        let key = rcgen::KeyPair::generate().unwrap();
        let params = rcgen::CertificateParams::new(vec![name.to_owned()]).unwrap();
        let issuer = rcgen::Issuer::from_params(&self.params, &self.key);
        let cert = params.signed_by(&key, &issuer).unwrap().pem();
        let key = key.serialize_pem();
        NodeTls::from_pem(trusted.pem.as_bytes(), cert.as_bytes(), key.as_bytes()).unwrap()
    }
}

/// The ports the nodes are given: below 32768, where Linux starts handing
/// out the local ports of outgoing connections
/// (`net.ipv4.ip_local_port_range`), and above the ones the development
/// services listen on.
const FIRST_PORT: u32 = 20_000;
const PORTS: u32 = 12_000;

static NEXT_PORT: AtomicU32 = AtomicU32::new(0);

/// `count` addresses nothing listens on, for the nodes to talk to each
/// other, none of them given before in this process.
///
/// A node binds its own address, so it is named here and let go of first.
/// A port the system picks and gets back is not safe to name: it comes from
/// the range outgoing connections and the other tests' listeners are served
/// from, and now and then one of them was given it before the node bound it
/// (`cannot bind the consensus listener … Address already in use`). So the
/// ports are taken below that range, each once: a counter says which comes
/// next, started from the process id so that two test binaries running at
/// once do not walk the same ports.
fn free_addrs(count: usize) -> Vec<SocketAddr> {
    let start = std::process::id().wrapping_mul(7919);
    let mut found = Vec::with_capacity(count);
    // Every port of the span tried once at most: more would mean none is
    // free, which no retry cures.
    for _ in 0..PORTS {
        let next = NEXT_PORT.fetch_add(1, Ordering::Relaxed);
        let port = FIRST_PORT + start.wrapping_add(next) % PORTS;
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port as u16));
        // Bound to see that nothing holds it, and let go of at once.
        if std::net::TcpListener::bind(addr).is_ok() {
            found.push(addr);
            if found.len() == count {
                return found;
            }
        }
    }
    panic!(
        "no {count} free ports between {FIRST_PORT} and {}",
        FIRST_PORT + PORTS
    );
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
