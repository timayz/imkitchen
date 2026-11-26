//! The port each node keeps for the other nodes, `images.listen`: where a
//! node copies a picture to this one before it records it, and takes from
//! this one a picture it lacks. Nothing else is served there, and the
//! public port serves none of it, so no request from outside the cluster
//! reaches these routes.
//!
//! With `[cluster.tls]` a connection is mutual TLS with this node's
//! certificate, the one the store's port presents, and is taken from a node
//! of the cluster only: the handshake asks for a certificate issued by the
//! cluster's authority, and that certificate must name a member as the
//! store's port dials it. Anything else ends before a request is read.
//! Without it the port is as open as the store's: for a network that is the
//! nodes' alone.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use axum::Router;
use axum::extract::{DefaultBodyLimit, Path, State};
use axum::http::StatusCode;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use axum::routing::put;
use bytes::Bytes;
use evento::cluster::tls::tokio_rustls::server::TlsStream;
use evento::cluster::{NodeTls, tls};
use timada_core::RENDITION_MIME;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use super::store::{Images, MAX_RENDITION_BYTES, PEER_ROUTE, parse_file};
use crate::cluster::Cluster;

/// How long a connection may take to say which node it is.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// How often a refused connection is logged, at most: a node with a wrong
/// certificate tries again every few seconds.
const SAY_REFUSED_EVERY: Duration = Duration::from_secs(30);

/// The nodes' port, served until [`shutdown`](Self::shutdown).
pub struct PeerPort {
    stop: oneshot::Sender<()>,
    task: JoinHandle<()>,
}

impl PeerPort {
    /// Stops taking requests and waits for those under way.
    pub async fn shutdown(self) {
        let _ = self.stop.send(());
        let _ = self.task.await;
    }
}

/// Serves the other nodes' requests for `images` on `listener`. With
/// `nodes`, over this node's certificate and to the members of its cluster
/// only; without, in clear to whoever connects.
pub fn serve(
    listener: TcpListener,
    nodes: Option<(NodeTls, Arc<Cluster>)>,
    images: Arc<Images>,
) -> PeerPort {
    let router = Router::new()
        .route(
            &format!("{PEER_ROUTE}/{{size}}/{{file}}"),
            put(take).get(give),
        )
        .layer(DefaultBodyLimit::max(MAX_RENDITION_BYTES))
        .with_state(images);
    let (stop, stopped) = oneshot::channel();
    let stopped = async {
        let _ = stopped.await;
    };
    let task = tokio::spawn(async move {
        let served = match nodes {
            Some((tls, cluster)) => {
                axum::serve(NodesOnly::new(listener, tls, cluster), router)
                    .with_graceful_shutdown(stopped)
                    .await
            }
            None => {
                axum::serve(listener, router)
                    .with_graceful_shutdown(stopped)
                    .await
            }
        };
        if let Err(err) = served {
            tracing::error!(%err, "the port of the nodes stopped");
        }
    });
    PeerPort { stop, task }
}

/// Stores the copy a node sends. 422 for a name or bytes that are not a
/// rendition of a picture.
async fn take(
    State(images): State<Arc<Images>>,
    Path((size, file)): Path<(String, String)>,
    bytes: Bytes,
) -> StatusCode {
    let Some((rendition, file_id)) = parse_file(&size, &file) else {
        return StatusCode::UNPROCESSABLE_ENTITY;
    };
    match images.accept(rendition, &file_id, bytes).await {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::UNPROCESSABLE_ENTITY,
        Err(err) => {
            tracing::error!(
                err = format!("{err:#}"),
                "a node's copy of a picture was not stored"
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// The file, for a node that lacks it. 404 when this node does not hold
/// it. The node checks the bytes against what the store recorded.
async fn give(
    State(images): State<Arc<Images>>,
    Path((size, file)): Path<(String, String)>,
) -> Response {
    let Some((rendition, file_id)) = parse_file(&size, &file) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match images.read(rendition, &file_id).await {
        Ok(Some(bytes)) => (
            [(CONTENT_TYPE, RENDITION_MIME), (CACHE_CONTROL, "no-store")],
            bytes,
        )
            .into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(err) => {
            tracing::error!(
                err = format!("{err:#}"),
                "a picture could not be read for a node"
            );
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// The connections of the nodes' port: each handshake on a task of its own,
/// so a slow one holds up no other, and only those of the cluster's members
/// handed to the server.
struct NodesOnly {
    addr: std::io::Result<SocketAddr>,
    taken: mpsc::Receiver<(TlsStream<TcpStream>, SocketAddr)>,
    task: JoinHandle<()>,
}

impl NodesOnly {
    fn new(listener: TcpListener, tls: NodeTls, cluster: Arc<Cluster>) -> Self {
        let addr = listener.local_addr();
        let (tx, taken) = mpsc::channel(64);
        let task = tokio::spawn(take_connections(listener, tls, cluster, tx));
        Self { addr, taken, task }
    }
}

impl Drop for NodesOnly {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl axum::serve::Listener for NodesOnly {
    type Io = TlsStream<TcpStream>;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        match self.taken.recv().await {
            Some(taken) => taken,
            // The port takes no more connections: the server is stopping.
            None => std::future::pending().await,
        }
    }

    fn local_addr(&self) -> std::io::Result<Self::Addr> {
        match &self.addr {
            Ok(addr) => Ok(*addr),
            Err(err) => Err(std::io::Error::new(err.kind(), err.to_string())),
        }
    }
}

/// Takes the connections on `listener` and hands on those of the nodes. A
/// failure to take one (no file descriptor left, say) is waited out: the
/// port stays open.
async fn take_connections(
    listener: TcpListener,
    tls: NodeTls,
    cluster: Arc<Cluster>,
    tx: mpsc::Sender<(TlsStream<TcpStream>, SocketAddr)>,
) {
    let refusals = Arc::new(Refusals::default());
    loop {
        let (tcp, from) = match listener.accept().await {
            Ok(taken) => taken,
            Err(err) => {
                tracing::warn!(%err, "the port of the nodes cannot take a connection");
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
        };
        // The certificate as it stands: one renewed is presented from the
        // next connection on.
        let acceptor = tls.acceptor();
        let (cluster, tx, refusals) = (cluster.clone(), tx.clone(), refusals.clone());
        tokio::spawn(async move {
            let stream = match tokio::time::timeout(HANDSHAKE_TIMEOUT, acceptor.accept(tcp)).await {
                Ok(Ok(stream)) => stream,
                Ok(Err(err)) => {
                    return refusals.say(from, &format!("the TLS handshake failed: {err}"));
                }
                Err(_) => return refusals.say(from, "the TLS handshake took too long"),
            };
            if !is_member(&cluster, &stream) {
                return refusals.say(from, "its certificate names no node of the cluster");
            }
            // The server may have stopped meanwhile.
            let _ = tx.send((stream, from)).await;
        });
    }
}

/// Whether the certificate a connection presented, which the handshake
/// checked against the cluster's authority, names a member of the cluster
/// or a node it is changing to: one valid for the address the store's port
/// dials that node at, as the store's port lets a node in.
fn is_member(cluster: &Cluster, stream: &TlsStream<TcpStream>) -> bool {
    let presented = stream.get_ref().1.peer_certificates();
    let members = cluster.members();
    let changing_to = members.change.iter().flat_map(|change| change.to.iter());
    members
        .nodes
        .iter()
        .chain(changing_to)
        .filter_map(|node| cluster.address(*node))
        .any(|address| tls::names(presented, &address).is_ok())
}

/// When a refused connection was last logged.
#[derive(Default)]
struct Refusals(Mutex<Option<Instant>>);

impl Refusals {
    fn say(&self, from: SocketAddr, why: &str) {
        let mut last = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if last.is_some_and(|last| last.elapsed() < SAY_REFUSED_EVERY) {
            return;
        }
        *last = Some(Instant::now());
        tracing::warn!(%from, why, "a connection on the port of the nodes is refused");
    }
}
