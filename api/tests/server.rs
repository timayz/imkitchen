mod helpers;

use std::net::SocketAddr;
use std::time::Duration;

use helpers::{TestCluster, TestNode};
use imkitchen_api::server;
use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

struct TestServer {
    addr: SocketAddr,
    shutdown: oneshot::Sender<()>,
    served: JoinHandle<anyhow::Result<()>>,
    /// The node's store lives here.
    _dir: TempDir,
}

/// The api over a node alone.
async fn start(shutdown_timeout: Duration) -> TestServer {
    let (node, dir) = TestNode::alone().await;
    let (addr, shutdown, served) = serve(&node, shutdown_timeout).await;

    TestServer {
        addr,
        shutdown,
        served,
        _dir: dir,
    }
}

async fn serve(
    node: &TestNode,
    shutdown_timeout: Duration,
) -> (
    SocketAddr,
    oneshot::Sender<()>,
    JoinHandle<anyhow::Result<()>>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (shutdown, stop) = oneshot::channel::<()>();
    let served = tokio::spawn(server::serve(
        listener,
        node.state(),
        shutdown_timeout,
        async {
            let _ = stop.await;
        },
    ));

    (addr, shutdown, served)
}

/// One request on its own connection; returns the raw response.
async fn get(addr: SocketAddr, path: &str) -> String {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(
            format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n").as_bytes(),
        )
        .await
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response
}

#[tokio::test]
async fn health_answers_ok() {
    let server = start(Duration::from_secs(5)).await;

    let response = get(server.addr, "/health").await;

    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(
        response.contains("content-type: application/json"),
        "{response}"
    );
    assert!(response.ends_with(r#"{"status":"ok"}"#), "{response}");
}

#[tokio::test]
async fn a_node_alone_is_ready() {
    let server = start(Duration::from_secs(5)).await;

    let response = get(server.addr, "/ready").await;

    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(response.contains(r#""status":"ok""#), "{response}");
    assert!(response.contains(r#""nodes":1"#), "{response}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_node_of_a_cluster_is_ready() {
    let cluster = TestCluster::start(3).await;

    for (id, node) in cluster.nodes.iter().enumerate() {
        let (addr, _shutdown, _served) = serve(node, Duration::from_secs(5)).await;

        let response = get(addr, "/ready").await;

        assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
        assert!(response.contains(&format!(r#""node":{id},"#)), "{response}");
        assert!(response.contains(r#""nodes":3"#), "{response}");
    }

    cluster.shutdown().await;
}

#[tokio::test]
async fn unknown_route_is_not_found() {
    let server = start(Duration::from_secs(5)).await;

    let response = get(server.addr, "/nope").await;

    assert!(response.starts_with("HTTP/1.1 404"), "{response}");
}

#[tokio::test]
async fn shutdown_stops_the_server() {
    let server = start(Duration::from_secs(5)).await;
    get(server.addr, "/health").await;

    server.shutdown.send(()).unwrap();

    server.served.await.unwrap().unwrap();
    assert!(TcpStream::connect(server.addr).await.is_err());
}

#[tokio::test]
async fn shutdown_does_not_wait_past_the_timeout() {
    let server = start(Duration::from_millis(100)).await;
    // A request that never completes keeps its connection in flight.
    let mut stalled = TcpStream::connect(server.addr).await.unwrap();
    stalled
        .write_all(b"GET /health HTTP/1.1\r\n")
        .await
        .unwrap();
    // Let the server accept it before it stops accepting.
    get(server.addr, "/health").await;

    server.shutdown.send(()).unwrap();

    tokio::time::timeout(Duration::from_secs(5), server.served)
        .await
        .expect("the server outlived its shutdown timeout")
        .unwrap()
        .unwrap();
}
