//! www and admin following the api's events through the sidecars' listener:
//! a read model built from scratch and kept current, a sidecar or the api
//! going away and coming back, and what a connection is refused.

mod helpers;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use evento::cursor::Args;
use evento::metadata::RawEvent;
use evento::remote::{ClientBuilder, Rejection, ServerHandle};
use evento::subscription::{Context, Subscription, SubscriptionBuilder};
use evento::{Executor, RemoteClient, Sidecar, TenantFilter, Turso};
use helpers::{TestCluster, TestNode, eventually};
use imkitchen_aggregates::identity::{
    AdminGranted, Claimed, EmailAccount, EmailRegistered, ProfileUpdated, User,
};
use imkitchen_aggregates::recipe;
use imkitchen_api::config::{Secret, SidecarTokens, SidecarsConfig};
use imkitchen_api::sidecars;
use tempfile::TempDir;
use timada_identity::{AccountView, user_projection};
use tokio::net::TcpListener;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName};
use tokio_rustls::rustls::{self, ClientConfig, RootCertStore};

const WWW_TOKEN: &str = "the token of www in the tests, 32 characters";
const ADMIN_TOKEN: &str = "the token of admin in the tests, 32 characters";

fn config() -> SidecarsConfig {
    SidecarsConfig {
        listen: "127.0.0.1:0".parse().unwrap(),
        tls_cert: None,
        tls_key: None,
        tokens: SidecarTokens {
            www: Some(Secret::new(WWW_TOKEN)),
            admin: Some(Secret::new(ADMIN_TOKEN)),
            app: None,
        },
    }
}

/// The sidecars' listener over `node`, at `addr`.
async fn listen_at(node: &TestNode, addr: SocketAddr, config: &SidecarsConfig) -> ServerHandle {
    let listener = TcpListener::bind(addr).await.unwrap();
    sidecars::serve(listener, config, &node.executor).unwrap()
}

/// The sidecars' listener over `node`, and where it listens.
async fn listen(node: &TestNode) -> (ServerHandle, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = sidecars::serve(listener, &config(), &node.executor).unwrap();
    (server, addr)
}

/// The users as a sidecar keeps them: a table in a store of its own, next
/// to the cursor of the subscription that fills it.
#[derive(Clone)]
struct Users {
    store: Turso,
    /// How many events were handled since the store was opened.
    handled: Arc<AtomicUsize>,
}

impl Users {
    async fn open(path: &Path) -> Self {
        let store = Turso::open(path).await.unwrap();
        store
            .connect()
            .unwrap()
            .execute(
                "CREATE TABLE IF NOT EXISTS users (
                    id TEXT PRIMARY KEY,
                    account TEXT NOT NULL,
                    label TEXT NOT NULL,
                    is_admin INTEGER NOT NULL
                )",
                (),
            )
            .await
            .unwrap();
        Self {
            store,
            handled: Arc::default(),
        }
    }

    async fn upsert(&self, user: &AccountView) -> anyhow::Result<()> {
        self.store
            .connect()?
            .execute(
                "INSERT INTO users (id, account, label, is_admin) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (id) DO UPDATE SET
                    account = excluded.account,
                    label = excluded.label,
                    is_admin = excluded.is_admin",
                (
                    user.id.as_str(),
                    user.account.as_str(),
                    user.label.as_str(),
                    i64::from(user.is_admin),
                ),
            )
            .await?;
        Ok(())
    }

    /// Every user as `(account, label, is_admin)`, by account.
    async fn all(&self) -> Vec<(String, String, bool)> {
        let conn = self.store.connect().unwrap();
        let mut rows = conn
            .query(
                "SELECT account, label, is_admin FROM users ORDER BY account",
                (),
            )
            .await
            .unwrap();
        let mut users = Vec::new();
        while let Some(row) = rows.next().await.unwrap() {
            users.push((
                row.get::<String>(0).unwrap(),
                row.get::<String>(1).unwrap(),
                row.get::<i64>(2).unwrap() != 0,
            ));
        }
        users
    }

    /// Waits until the users are `expected`, as `(account, label, is_admin)`.
    async fn are(&self, expected: &[(&str, &str, bool)]) {
        let expected: Vec<_> = expected
            .iter()
            .map(|(account, label, is_admin)| {
                ((*account).to_owned(), (*label).to_owned(), *is_admin)
            })
            .collect();
        eventually(&format!("the sidecar to hold {expected:?}"), || async {
            (self.all().await == expected).then_some(())
        })
        .await;
    }
}

/// Rewrites the row of the user an event belongs to from the user's stream,
/// read through the listener: the way the identity's own read model does.
#[evento::subscription_all]
async fn on_user_event<E: Executor>(
    context: &Context<'_, E>,
    event: RawEvent<User>,
) -> anyhow::Result<()> {
    let users: Users = context.extract();
    if let Some(user) = user_projection()
        .load(&event.aggregate_id)
        .execute(context.executor)
        .await?
    {
        users.upsert(&user).await?;
    }
    users.handled.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

fn users_subscription<E: Executor>(users: Users) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("users")
        .handler(on_user_event())
        .data(users)
}

/// A sidecar following the users: the api's events through `client`, its
/// cursor and its table in the store at `path`.
struct Follower {
    users: Users,
    subscription: Subscription,
}

impl Follower {
    async fn start(client: ClientBuilder, path: &Path) -> Self {
        let client = client.connect().await.unwrap();
        let users = Users::open(path).await;
        let executor = Sidecar::new(client, users.store.clone());
        let subscription = users_subscription(users.clone())
            .supervised()
            .start(&executor)
            .await
            .unwrap();
        Self {
            users,
            subscription,
        }
    }

    fn handled(&self) -> usize {
        self.users.handled.load(Ordering::SeqCst)
    }

    /// Waits until the cursor is at the newest event followed. A cursor is
    /// recorded once the listener said how far behind it is: an event whose
    /// handler ran when the listener went away is delivered again.
    async fn caught_up(&self) {
        eventually("the sidecar to record its cursor", || async {
            let lag = self.subscription.lag().await.ok()?;
            lag.is_caught_up().then_some(())
        })
        .await;
    }

    /// Stops following; the connection goes with the subscription.
    async fn stop(self) {
        self.subscription.shutdown().await.unwrap();
    }
}

fn admin(addr: SocketAddr) -> ClientBuilder {
    RemoteClient::builder(addr).token(ADMIN_TOKEN)
}

async fn register<E: Executor>(executor: &E, email: &str, label: &str) -> String {
    evento::create()
        .event(&EmailRegistered {
            email: email.to_owned(),
            label: label.to_owned(),
            password_hash: "$argon2id$not-a-hash".to_owned(),
        })
        .commit(executor)
        .await
        .unwrap()
}

async fn version<E: Executor>(executor: &E, id: &str) -> u16 {
    evento::read::<User>(id)
        .execute(executor)
        .await
        .unwrap()
        .len() as u16
}

async fn rename<E: Executor>(executor: &E, id: &str, email: &str, label: &str) {
    evento::append(id)
        .original_version(version(executor, id).await)
        .event(&ProfileUpdated {
            account: email.to_owned(),
            label: label.to_owned(),
        })
        .commit(executor)
        .await
        .unwrap();
}

async fn grant_admin<E: Executor>(executor: &E, id: &str) {
    evento::append(id)
        .original_version(version(executor, id).await)
        .event(&AdminGranted)
        .commit(executor)
        .await
        .unwrap();
}

/// Every event `executor` lets a read that names no aggregate see.
async fn everything<E: Executor>(executor: &E) -> Vec<evento::Event> {
    executor
        .read(
            None,
            None,
            TenantFilter::All,
            Args::forward(100, None),
            None,
        )
        .await
        .unwrap()
        .edges
        .into_iter()
        .map(|edge| edge.node)
        .collect()
}

#[tokio::test]
async fn a_sidecar_builds_its_read_model_from_scratch_then_as_events_come() {
    let (node, dir) = TestNode::alone().await;
    // Written before anything listens or follows.
    let alice = register(&node.executor, "alice@example.com", "Alice").await;
    let bob = register(&node.executor, "bob@example.com", "Bob").await;
    rename(&node.executor, &alice, "alice@example.com", "Alice A.").await;
    let (server, addr) = listen(&node).await;

    let follower = Follower::start(admin(addr), &dir.path().join("admin.db")).await;
    follower
        .users
        .are(&[
            ("alice@example.com", "Alice A.", false),
            ("bob@example.com", "Bob", false),
        ])
        .await;

    register(&node.executor, "carol@example.com", "Carol").await;
    grant_admin(&node.executor, &bob).await;
    follower
        .users
        .are(&[
            ("alice@example.com", "Alice A.", false),
            ("bob@example.com", "Bob", true),
            ("carol@example.com", "Carol", false),
        ])
        .await;
    assert_eq!(follower.handled(), 5, "one pass over each event");

    follower.stop().await;
    server.shutdown().await;
    node.cluster.shutdown().await;
}

#[tokio::test]
async fn a_sidecar_that_was_away_catches_up_from_where_it_stopped() {
    let (node, dir) = TestNode::alone().await;
    let store = dir.path().join("admin.db");
    let (server, addr) = listen(&node).await;
    let alice = register(&node.executor, "alice@example.com", "Alice").await;

    let follower = Follower::start(admin(addr), &store).await;
    follower
        .users
        .are(&[("alice@example.com", "Alice", false)])
        .await;
    follower.stop().await;

    // Nobody follows.
    register(&node.executor, "bob@example.com", "Bob").await;
    rename(&node.executor, &alice, "alice@example.com", "Alice A.").await;

    // The same store: its cursor is where the sidecar stopped.
    let follower = Follower::start(admin(addr), &store).await;
    follower
        .users
        .are(&[
            ("alice@example.com", "Alice A.", false),
            ("bob@example.com", "Bob", false),
        ])
        .await;
    assert_eq!(follower.handled(), 2, "only what it missed");

    follower.stop().await;
    server.shutdown().await;
    node.cluster.shutdown().await;
}

#[tokio::test]
async fn a_sidecar_catches_up_when_the_api_comes_back() {
    let (node, dir) = TestNode::alone().await;
    let (server, addr) = listen(&node).await;
    let follower = Follower::start(admin(addr), &dir.path().join("admin.db")).await;
    let alice = register(&node.executor, "alice@example.com", "Alice").await;
    follower
        .users
        .are(&[("alice@example.com", "Alice", false)])
        .await;
    follower.caught_up().await;

    // The listener goes away under the sidecar, which keeps running.
    server.shutdown().await;
    register(&node.executor, "bob@example.com", "Bob").await;
    grant_admin(&node.executor, &alice).await;
    assert_eq!(follower.handled(), 1);

    let server = listen_at(&node, addr, &config()).await;
    follower
        .users
        .are(&[
            ("alice@example.com", "Alice", true),
            ("bob@example.com", "Bob", false),
        ])
        .await;
    assert_eq!(follower.handled(), 3, "only what it missed");

    follower.stop().await;
    server.shutdown().await;
    node.cluster.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_sidecar_follows_what_any_node_writes() {
    let cluster = TestCluster::start(3).await;
    let dir = tempfile::tempdir().unwrap();
    let (server, addr) = listen(&cluster.nodes[0]).await;
    let follower = Follower::start(admin(addr), &dir.path().join("admin.db")).await;

    let alice = register(&cluster.nodes[1].executor, "alice@example.com", "Alice").await;
    register(&cluster.nodes[2].executor, "bob@example.com", "Bob").await;
    grant_admin(&cluster.nodes[2].executor, &alice).await;

    follower
        .users
        .are(&[
            ("alice@example.com", "Alice", true),
            ("bob@example.com", "Bob", false),
        ])
        .await;

    follower.stop().await;
    server.shutdown().await;
    cluster.shutdown().await;
}

#[tokio::test]
async fn a_connection_without_the_token_of_a_sidecar_is_refused() {
    let (node, _dir) = TestNode::alone().await;
    // Only admin is given a token.
    let config = SidecarsConfig {
        tokens: SidecarTokens {
            www: None,
            admin: Some(Secret::new(ADMIN_TOKEN)),
            app: None,
        },
        ..config()
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = sidecars::serve(listener, &config, &node.executor).unwrap();

    for client in [
        RemoteClient::builder(addr),
        RemoteClient::builder(addr).token("not the token of a sidecar, whatever its length"),
        RemoteClient::builder(addr).token(WWW_TOKEN),
    ] {
        let err = client.connect().await.err().expect("connected");
        assert_eq!(
            err.downcast_ref::<Rejection>(),
            Some(&Rejection::Unauthorized),
            "{err:#}"
        );
    }
    admin(addr).connect().await.unwrap();

    server.shutdown().await;
    node.cluster.shutdown().await;
}

#[tokio::test]
async fn a_sidecar_changes_nothing_in_the_api() {
    let (node, dir) = TestNode::alone().await;
    let alice = register(&node.executor, "alice@example.com", "Alice").await;
    let (server, addr) = listen(&node).await;
    let client = admin(addr).connect().await.unwrap();

    // No event, on a new stream or on one it reads.
    let err = evento::create()
        .event(&EmailRegistered {
            email: "mallory@example.com".to_owned(),
            label: "Mallory".to_owned(),
            password_hash: "$argon2id$not-a-hash".to_owned(),
        })
        .commit(&client)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("may not write"), "{err}");
    let err = evento::append(&alice)
        .original_version(1)
        .event(&AdminGranted)
        .commit(&client)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("may not write"), "{err}");
    assert_eq!(everything(&node.store).await.len(), 1);

    // No cursor either: a subscription without a store of its own does not
    // run.
    let users = Users::open(&dir.path().join("admin.db")).await;
    let err = users_subscription(users.clone())
        .no_retry()
        .run_once(&client)
        .await
        .unwrap_err();
    assert!(
        format!("{err:#}").contains("keeps no subscription"),
        "{err:#}"
    );
    assert_eq!(users.handled.load(Ordering::SeqCst), 0);
    let subscribers = node
        .store
        .connect()
        .unwrap()
        .query("SELECT 1 FROM subscriber", ())
        .await
        .unwrap()
        .next()
        .await
        .unwrap();
    assert!(
        subscribers.is_none(),
        "the api keeps a cursor for a sidecar"
    );

    server.shutdown().await;
    node.cluster.shutdown().await;
}

#[tokio::test]
async fn a_sidecar_reads_its_aggregates_and_no_other() {
    let (node, _dir) = TestNode::alone().await;
    let alice = register(&node.executor, "alice@example.com", "Alice").await;
    // What the password sign-in is checked against: no sidecar's to read.
    let index = evento::create()
        .event(&Claimed {
            user_id: alice.clone(),
            email: "alice@example.com".to_owned(),
        })
        .commit(&node.executor)
        .await
        .unwrap();
    // A recipe: its owner's alone, www's to count and admin's to look
    // after.
    let soup = evento::create()
        .event(&recipe::Created {
            owner_id: alice.clone(),
            title: "Soup".to_owned(),
            description: String::new(),
            ingredients: Vec::new(),
            instructions: Vec::new(),
            prep_minutes: 0,
            cook_minutes: 0,
            servings: 1,
        })
        .commit(&node.executor)
        .await
        .unwrap();
    assert_eq!(everything(&node.store).await.len(), 3);
    let (server, addr) = listen(&node).await;

    let admin = admin(addr).connect().await.unwrap();
    assert_eq!(version(&admin, &alice).await, 1);
    let err = evento::read::<EmailAccount>(&index)
        .execute(&admin)
        .await
        .unwrap_err();
    assert!(format!("{err:#}").contains("may not read"), "{err:#}");
    // A read that names nothing is given what admin reads: the user and
    // the recipe, not the index.
    let read = everything(&admin).await;
    let mut read: Vec<&str> = read
        .iter()
        .map(|event| event.aggregate_id.as_str())
        .collect();
    read.sort_unstable();
    let mut expected = [alice.as_str(), soup.as_str()];
    expected.sort_unstable();
    assert_eq!(read, expected);

    // www reads the recipes, and nothing of the identity.
    let www = RemoteClient::builder(addr)
        .token(WWW_TOKEN)
        .connect()
        .await
        .unwrap();
    let err = evento::read::<User>(&alice)
        .execute(&www)
        .await
        .unwrap_err();
    assert!(format!("{err:#}").contains("may not read"), "{err:#}");
    let read = everything(&www).await;
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].aggregate_id, soup);

    server.shutdown().await;
    node.cluster.shutdown().await;
}

/// A certificate for `localhost` and its key, as the PEM files of a
/// configuration, and a connector that trusts it.
fn tls(dir: &TempDir) -> (SidecarsConfig, TlsConnector) {
    let rcgen::CertifiedKey { cert, signing_key } =
        rcgen::generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
    let cert_path = dir.path().join("tls.crt");
    let key_path = dir.path().join("tls.key");
    std::fs::write(&cert_path, cert.pem()).unwrap();
    std::fs::write(&key_path, signing_key.serialize_pem()).unwrap();

    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from_pem_file(&cert_path).unwrap())
        .unwrap();
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let client = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let config = SidecarsConfig {
        tls_cert: Some(cert_path),
        tls_key: Some(key_path),
        ..config()
    };
    (config, TlsConnector::from(Arc::new(client)))
}

#[tokio::test]
async fn a_sidecar_follows_over_tls() {
    let (node, dir) = TestNode::alone().await;
    let (config, connector) = tls(&dir);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = sidecars::serve(listener, &config, &node.executor).unwrap();
    register(&node.executor, "alice@example.com", "Alice").await;

    let client = admin(addr).tls(connector, ServerName::try_from("localhost").unwrap());
    let follower = Follower::start(client, &dir.path().join("admin.db")).await;
    follower
        .users
        .are(&[("alice@example.com", "Alice", false)])
        .await;

    // In clear text nothing is said.
    assert!(admin(addr).connect().await.is_err());

    follower.stop().await;
    server.shutdown().await;
    node.cluster.shutdown().await;
}
