//! The sidecars' listener: where www, admin and app read the api's
//! events from.
//!
//! They keep read models of their own, each in a store of its own, and build
//! them by following the events of the api (an `evento::Sidecar`:
//! the events come from here, its cursors and snapshots stay there). This is
//! the listener they connect to, evento's remote protocol over TCP, wrapped
//! in TLS when the configuration gives a certificate.
//!
//! Every connection opens with the token of its sidecar (`[sidecars.tokens]`)
//! and is refused without one. It is then served the store with every way of
//! changing it taken away, and only the aggregates its sidecar reads
//! ([`Sidecar::reads`]): see [`ReadOnly`]. A sidecar's commands go through
//! the HTTP routes like anyone's.
//!
//! What a connection reads comes from this node's file as it stands, without
//! a consensus round: a subscription only reads up to what the node knows is
//! settled, and what its handlers load is behind the event they handle. A
//! sidecar may connect to any node, and to another one the next time.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use evento::cursor::{Args, ReadResult, Value};
use evento::remote::{Hello, Rejection, Resolver, ServerBuilder, ServerHandle, Session};
use evento::{
    Event, EventFilter, RoutingKey, SubscriberStatus, TenantFilter, WriteError, WriteInterest,
};
use imkitchen_aggregates::Sidecar;
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::crypto::CryptoProvider;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::{self, ServerConfig};
use ulid::Ulid;

use crate::cluster::Executor;
use crate::config::SidecarsConfig;

/// Serves the sidecars on `listener` until [`ServerHandle::shutdown`], from
/// what `executor`'s node holds.
pub fn serve(
    listener: TcpListener,
    config: &SidecarsConfig,
    executor: &Executor,
) -> Result<ServerHandle> {
    let addr = listener.local_addr()?;
    let server = ServerBuilder::new(Tokens::new(config, executor.local_reads()));
    Ok(match config.tls() {
        Some((cert, key)) => {
            let acceptor = tls_acceptor(cert, key)?;
            tracing::info!(%addr, "sidecars listening (TLS)");
            server.tls(acceptor).serve(listener)
        }
        None => {
            tracing::info!(%addr, "sidecars listening (no encryption)");
            server.serve(listener)
        }
    })
}

/// Turns the token a connection opens with into what its sidecar is served.
struct Tokens<E> {
    /// Each configured sidecar, by the hash of its token.
    sidecars: Vec<([u8; 32], Sidecar)>,
    store: E,
}

impl<E> Tokens<E> {
    fn new(config: &SidecarsConfig, store: E) -> Self {
        let sidecars = config
            .tokens()
            .map(|(sidecar, token)| (hash(token.expose()), sidecar))
            .collect();
        Self { sidecars, store }
    }

    /// Whose `token` is. Hashes are compared, so how long the comparison
    /// takes says nothing of the token.
    fn sidecar(&self, token: &str) -> Option<Sidecar> {
        let presented = hash(token);
        self.sidecars
            .iter()
            .find(|(expected, _)| *expected == presented)
            .map(|(_, sidecar)| *sidecar)
    }
}

fn hash(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

#[async_trait::async_trait]
impl<E: evento::Executor + Clone> Resolver for Tokens<E> {
    type Executor = ReadOnly<E>;

    async fn resolve(&self, hello: &Hello) -> Result<Session<ReadOnly<E>>, Rejection> {
        let Some(sidecar) = hello.token.as_deref().and_then(|token| self.sidecar(token)) else {
            tracing::warn!(peer = %hello.peer, "sidecar refused: no token, or not one of ours");
            return Err(Rejection::Unauthorized);
        };
        tracing::info!(sidecar = sidecar.name(), peer = %hello.peer, "sidecar connected");
        Ok(
            Session::new(ReadOnly::new(self.store.clone(), sidecar)).keep(Connected {
                sidecar,
                peer: hello.peer,
            }),
        )
    }
}

/// Kept by a connection for as long as it lasts, to say when it ended.
struct Connected {
    sidecar: Sidecar,
    peer: std::net::SocketAddr,
}

impl Drop for Connected {
    fn drop(&mut self) {
        tracing::info!(sidecar = self.sidecar.name(), peer = %self.peer, "sidecar disconnected");
    }
}

/// The executor a sidecar is served: `inner`, where nothing can be changed
/// and only the aggregates of the sidecar are read.
///
/// - Reads are confined to [`Sidecar::reads`]. A read that names no
///   aggregate is narrowed to that list; one that names an aggregate outside
///   it fails with the reason.
/// - Writes are refused: a sidecar sends its commands to the HTTP routes.
/// - The subscriber calls are refused: a sidecar keeps the cursors of its
///   subscriptions in its own store, and a cursor kept here would be a write
///   (and one shared by every replica of the sidecar).
/// - There is no snapshot to load, and the ones saved are dropped: a
///   snapshot is state the api trusts when it loads the same view. Dropping
///   rather than refusing keeps the loads of a client without a store of its
///   own working, without the cache.
/// - Dead letters and deleting a stream are not this executor's
///   (the trait's defaults refuse them).
#[derive(Clone)]
pub struct ReadOnly<E> {
    inner: E,
    sidecar: Sidecar,
    reads: Arc<[&'static str]>,
}

impl<E> ReadOnly<E> {
    pub fn new(inner: E, sidecar: Sidecar) -> Self {
        Self {
            inner,
            sidecar,
            reads: sidecar.reads().into(),
        }
    }

    /// `Err` naming the aggregate when the sidecar may not read it.
    fn check_read(&self, aggregate_type: &str) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.reads.contains(&aggregate_type),
            "{} may not read {aggregate_type} events",
            self.sidecar.name()
        );
        Ok(())
    }

    /// The filters a read goes down with: the caller's, each checked, or the
    /// whole list of the sidecar when it named none.
    fn narrow(
        &self,
        aggregators: Option<Arc<[EventFilter]>>,
    ) -> anyhow::Result<Option<Arc<[EventFilter]>>> {
        match aggregators {
            Some(filters) => {
                for filter in filters.iter() {
                    self.check_read(&filter.aggregate_type)?;
                }
                Ok(Some(filters))
            }
            // An empty list matches nothing.
            None => Ok(Some(
                self.reads
                    .iter()
                    .map(|aggregate_type| EventFilter {
                        aggregate_type: (*aggregate_type).to_owned(),
                        aggregate_id: None,
                        name: None,
                    })
                    .collect(),
            )),
        }
    }

    fn no_write(&self) -> anyhow::Error {
        anyhow::anyhow!(
            "{} may not write events: only the api does, send it the command",
            self.sidecar.name()
        )
    }

    fn no_subscriber(&self) -> anyhow::Error {
        anyhow::anyhow!(
            "{} keeps no subscription in the api: follow it through an evento Sidecar, \
             its cursors in a store of its own",
            self.sidecar.name()
        )
    }
}

#[async_trait::async_trait]
impl<E: evento::Executor> evento::Executor for ReadOnly<E> {
    fn default_routing_key(&self) -> Option<&str> {
        self.inner.default_routing_key()
    }

    async fn resolve_default_routing_key(&self) -> anyhow::Result<Option<String>> {
        self.inner.resolve_default_routing_key().await
    }

    async fn write(&self, _events: Vec<Event>) -> Result<(), WriteError> {
        Err(WriteError::Unknown(self.no_write()))
    }

    async fn replicate(&self, _events: Vec<Event>) -> Result<(), WriteError> {
        Err(WriteError::Unknown(self.no_write()))
    }

    fn write_watch(&self) -> Option<tokio::sync::watch::Receiver<u64>> {
        self.inner.write_watch()
    }

    fn write_watch_for(
        &self,
        interest: &WriteInterest,
    ) -> Option<tokio::sync::watch::Receiver<u64>> {
        self.inner.write_watch_for(interest)
    }

    fn hears_every_write(&self) -> bool {
        self.inner.hears_every_write()
    }

    async fn stable_timestamp(&self) -> anyhow::Result<Option<u64>> {
        self.inner.stable_timestamp().await
    }

    async fn read(
        &self,
        aggregators: Option<Arc<[EventFilter]>>,
        routing_key: Option<RoutingKey>,
        tenant: TenantFilter,
        args: Args,
        to_micros: Option<u64>,
    ) -> anyhow::Result<ReadResult<Event>> {
        let aggregators = self.narrow(aggregators)?;
        self.inner
            .read(aggregators, routing_key, tenant, args, to_micros)
            .await
    }

    async fn latest_timestamp(
        &self,
        aggregators: Option<Arc<[EventFilter]>>,
        routing_key: Option<RoutingKey>,
        tenant: TenantFilter,
    ) -> anyhow::Result<u64> {
        let aggregators = self.narrow(aggregators)?;
        self.inner
            .latest_timestamp(aggregators, routing_key, tenant)
            .await
    }

    async fn latest_version(
        &self,
        tenant: String,
        aggregate_type: String,
        aggregate_id: String,
    ) -> anyhow::Result<u16> {
        self.check_read(&aggregate_type)?;
        self.inner
            .latest_version(tenant, aggregate_type, aggregate_id)
            .await
    }

    async fn stream_routing_key(
        &self,
        tenant: String,
        aggregate_type: String,
        aggregate_id: String,
    ) -> anyhow::Result<Option<Option<String>>> {
        self.check_read(&aggregate_type)?;
        self.inner
            .stream_routing_key(tenant, aggregate_type, aggregate_id)
            .await
    }

    async fn get_subscriber_cursor(&self, _key: String) -> anyhow::Result<Option<Value>> {
        Err(self.no_subscriber())
    }

    async fn is_subscriber_running(&self, _key: String, _worker_id: Ulid) -> anyhow::Result<bool> {
        Err(self.no_subscriber())
    }

    async fn subscriber_status(
        &self,
        _key: String,
        _worker_id: Ulid,
    ) -> anyhow::Result<SubscriberStatus> {
        Err(self.no_subscriber())
    }

    async fn upsert_subscriber(&self, _key: String, _worker_id: Ulid) -> anyhow::Result<()> {
        Err(self.no_subscriber())
    }

    async fn acknowledge(
        &self,
        _key: String,
        _worker_id: Ulid,
        _cursor: Value,
        _lag: u64,
    ) -> anyhow::Result<bool> {
        Err(self.no_subscriber())
    }

    async fn get_snapshot(
        &self,
        _tenant: String,
        _aggregate_type: String,
        _projection: String,
        _aggregate_revision: String,
        _id: String,
    ) -> anyhow::Result<Option<(Vec<u8>, Value)>> {
        Ok(None)
    }

    async fn save_snapshot(
        &self,
        _tenant: String,
        _aggregate_type: String,
        _projection: String,
        _aggregate_revision: String,
        _id: String,
        _data: Vec<u8>,
        _cursor: Value,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete_snapshot(
        &self,
        _tenant: String,
        _aggregate_type: String,
        _projection: String,
        _id: String,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

/// The server side of TLS from PEM files: the certificate chain and its key.
/// Who connects is told by its token, not by a certificate.
fn tls_acceptor(cert: &Path, key: &Path) -> Result<TlsAcceptor> {
    // The process's provider when something installed one.
    let provider = CryptoProvider::get_default()
        .cloned()
        .unwrap_or_else(|| Arc::new(rustls::crypto::aws_lc_rs::default_provider()));
    let certs: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(cert)
        .with_context(|| format!("cannot read {}", cert.display()))?
        .collect::<std::result::Result<_, _>>()
        .with_context(|| format!("cannot parse {}", cert.display()))?;
    let key = PrivateKeyDer::from_pem_file(key)
        .with_context(|| format!("cannot read {}", key.display()))?;
    let config = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .context("the TLS provider supports no protocol version")?
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("sidecars.tls_cert and sidecars.tls_key do not go together")?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}
