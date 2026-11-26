//! The other nodes of the cluster, as the pictures every node keeps reach
//! them. A file is copied to a majority of the cluster before the event
//! naming it is recorded, the same majority a commit needs, and each of
//! those nodes syncs the file to its disk before it answers
//! ([`super::disk`]), as the store syncs each write: the file survives
//! whatever the events survive.
//!
//! They are asked on the port each keeps for it (`images.listen`), where
//! nothing else is served ([`super::port`]). With `[cluster.tls]` that is
//! mutual TLS with this node's certificate, as on the store's port: a node
//! is let in as the node its certificate names, and no secret travels.
//! Without it the nodes talk in clear, on a network that is theirs alone.

use std::sync::{Arc, PoisonError, RwLock};
use std::time::Duration;

use anyhow::Context;
use bytes::Bytes;
use evento::cluster::NodeTls;
use evento::cluster::tls::tokio_rustls::rustls::ClientConfig;

/// How long one copy to or from a peer may take.
const PEER_TIMEOUT: Duration = Duration::from_secs(30);

pub struct Peers {
    /// Where the other nodes take pictures, no trailing slash.
    urls: Vec<String>,
    /// Copies the other nodes must acknowledge before a file is recorded:
    /// with this node's own, a majority of the cluster.
    quorum: usize,
    client: Client,
}

/// What the requests to the other nodes go through.
enum Client {
    /// Mutual TLS with this node's certificate, built again once the
    /// certificate is renewed, which the node reads again as it runs.
    Tls {
        tls: NodeTls,
        built: RwLock<Built>,
    },
    Plain(reqwest::Client),
}

/// A client, and the TLS it was built with.
struct Built {
    config: Arc<ClientConfig>,
    client: reqwest::Client,
}

impl Peers {
    /// The other nodes of a cluster of `nodes`, at `urls`, reached with
    /// this node's certificate when it has one.
    pub fn new(urls: Vec<String>, nodes: usize, tls: Option<NodeTls>) -> anyhow::Result<Self> {
        let client = match tls {
            Some(tls) => {
                let config = tls.connector().config().clone();
                let client = build(Some(&config))?;
                Client::Tls {
                    tls,
                    built: RwLock::new(Built { config, client }),
                }
            }
            None => Client::Plain(build(None)?),
        };
        Ok(Self {
            urls: urls
                .into_iter()
                .map(|url| url.trim_end_matches('/').to_owned())
                .collect(),
            quorum: nodes / 2,
            client,
        })
    }

    pub fn urls(&self) -> &[String] {
        &self.urls
    }

    /// The client a request to a peer goes through, presenting this node's
    /// certificate as it stands.
    pub fn client(&self) -> reqwest::Client {
        match &self.client {
            Client::Tls { tls, built } => {
                let config = tls.connector().config().clone();
                {
                    let built = built.read().unwrap_or_else(PoisonError::into_inner);
                    if Arc::ptr_eq(&built.config, &config) {
                        return built.client.clone();
                    }
                }
                // Renewed: the requests from now on present the new one.
                let mut built = built.write().unwrap_or_else(PoisonError::into_inner);
                if !Arc::ptr_eq(&built.config, &config) {
                    match build(Some(&config)) {
                        Ok(client) => built.client = client,
                        Err(err) => tracing::warn!(
                            err = format!("{err:#}"),
                            "cannot reach the other nodes with the renewed certificate; the requests go on with the one before"
                        ),
                    }
                    // Once a renewal: a failure is not tried again each time.
                    built.config = config;
                }
                built.client.clone()
            }
            Client::Plain(client) => client.clone(),
        }
    }

    /// PUTs `body` at `path` on every other node, returning once a majority
    /// of the cluster (this node included) holds it. The copies to the
    /// remaining nodes carry on in the background. Fails when too many nodes
    /// refused or could not be reached for a majority; what names the file
    /// must then not be recorded.
    pub async fn put_to_quorum(&self, path: &str, body: Bytes) -> anyhow::Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(self.urls.len().max(1));
        let client = self.client();
        for url in &self.urls {
            let request = client.put(format!("{url}{path}")).body(body.clone());
            let (tx, url) = (tx.clone(), url.clone());
            tokio::spawn(async move {
                let copied = match request.send().await {
                    Ok(response) if response.status().is_success() => Ok(()),
                    Ok(response) => Err(anyhow::anyhow!("answered {}", response.status())),
                    Err(err) => Err(err.into()),
                };
                if let Err(err) = &copied {
                    tracing::warn!(peer = %url, err = format!("{err:#}"), "cannot copy a picture to a peer");
                }
                // The upload may already have its majority and be gone.
                let _ = tx.send(copied.is_ok()).await;
            });
        }
        drop(tx);
        let (mut copied, mut failed) = (0, 0);
        while copied < self.quorum {
            match rx.recv().await {
                Some(true) => copied += 1,
                Some(false) => failed += 1,
                None => break,
            }
            if failed > self.urls.len().saturating_sub(self.quorum) {
                break;
            }
        }
        anyhow::ensure!(
            copied >= self.quorum,
            "only {copied} of {} other nodes took the picture, {} needed",
            self.urls.len(),
            self.quorum
        );
        Ok(())
    }
}

/// A client for the other nodes, presenting this node's certificate as
/// `tls` holds it when there is one.
fn build(tls: Option<&ClientConfig>) -> anyhow::Result<reqwest::Client> {
    let mut client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        // The nodes are reached where the configuration says, not through
        // a proxy of the environment.
        .no_proxy()
        .timeout(PEER_TIMEOUT);
    if let Some(tls) = tls {
        client = client.tls_backend_preconfigured(tls.clone());
    }
    client
        .build()
        .context("cannot build the client for the other nodes")
}
