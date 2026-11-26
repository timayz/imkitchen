//! admin against an api: its sidecar following the api's listener into a
//! store of its own, from scratch, as events come and after either was away,
//! and the pages read from what it followed.

use std::future::Future;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use evento::remote::ServerHandle;
use evento::subscription::Subscription;
use imkitchen_admin::config::ApiConfig;
use imkitchen_admin::db::{self, Db};
use imkitchen_admin::events::Events;
use imkitchen_admin::{app, config, subscriptions, users};
use imkitchen_aggregates::identity::{Deleted, EmailRegistered, ProfileUpdated, User};
use imkitchen_api::cluster::{self, Cluster, Executor};
use imkitchen_api::config::{Secret, SidecarTokens, SidecarsConfig};
use imkitchen_api::sidecars;
use tempfile::TempDir;
use tokio::net::TcpListener;
use topcoat::asset::{
    AssetConfig, MANIFEST_VERSION, Manifest, ManifestEntry, RouterBuilderAssetExt,
};
use topcoat::router::request::Request;
use topcoat::router::{Body, Router, StatusCode, to_bytes};

const ADMIN_TOKEN: &str = "the token of admin in the tests, 32 characters";

/// How long a test waits for what admin does on its own time.
const SETTLE: Duration = Duration::from_secs(30);

/// Polls `check` until it answers, and panics past [`SETTLE`].
async fn eventually<T, F, Fut>(what: &str, mut check: F) -> T
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Option<T>>,
{
    let deadline = tokio::time::Instant::now() + SETTLE;
    loop {
        if let Some(value) = check().await {
            return value;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for {what}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// An api node alone, over a store created the way a deployment creates it,
/// serving the sidecars as the api does.
struct Api {
    cluster: Arc<Cluster>,
    executor: Executor,
    /// The sidecars' listener, while there is one.
    server: Option<ServerHandle>,
    addr: SocketAddr,
}

impl Api {
    async fn start(dir: &Path) -> Self {
        let path = dir.join("imkitchen.db");
        drop(imkitchen_api::db::migrate(&path).await.unwrap());
        let store = imkitchen_api::db::open(&path).await.unwrap();
        let (cluster, executor) = cluster::open(None, None, &path, store).await.unwrap();
        let (server, addr) = Self::listen(&executor, "127.0.0.1:0".parse().unwrap()).await;
        Self {
            cluster,
            executor,
            server: Some(server),
            addr,
        }
    }

    async fn listen(executor: &Executor, addr: SocketAddr) -> (ServerHandle, SocketAddr) {
        let config = SidecarsConfig {
            listen: addr,
            tls_cert: None,
            tls_key: None,
            tokens: SidecarTokens {
                www: None,
                admin: Some(Secret::new(ADMIN_TOKEN)),
            },
        };
        let listener = TcpListener::bind(addr).await.unwrap();
        let addr = listener.local_addr().unwrap();
        (sidecars::serve(listener, &config, executor).unwrap(), addr)
    }

    /// Stops listening: the node stays, and takes writes.
    async fn stop_listening(&mut self) {
        self.server.take().unwrap().shutdown().await;
    }

    /// Listens again where it did.
    async fn listen_again(&mut self) {
        let (server, _) = Self::listen(&self.executor, self.addr).await;
        self.server = Some(server);
    }

    /// Registers a user as the api does; its id.
    async fn register(&self, email: &str) -> String {
        evento::create()
            .event(&EmailRegistered {
                email: email.to_owned(),
                label: email.to_owned(),
                password_hash: "$argon2id$not-a-hash".to_owned(),
            })
            .commit(&self.executor)
            .await
            .unwrap()
    }

    async fn version(&self, id: &str) -> u16 {
        evento::read::<User>(id)
            .execute(&self.executor)
            .await
            .unwrap()
            .len() as u16
    }

    async fn rename(&self, id: &str, email: &str, label: &str) {
        evento::append(id)
            .original_version(self.version(id).await)
            .event(&ProfileUpdated {
                account: email.to_owned(),
                label: label.to_owned(),
            })
            .commit(&self.executor)
            .await
            .unwrap();
    }

    async fn delete(&self, id: &str) {
        evento::append(id)
            .original_version(self.version(id).await)
            .event(&Deleted)
            .commit(&self.executor)
            .await
            .unwrap();
    }

    async fn shutdown(self) {
        if let Some(server) = self.server {
            server.shutdown().await;
        }
        self.cluster.shutdown().await;
    }
}

/// admin over the store at `path`, following the api at `addr`.
struct Admin {
    db: Db,
    events: Events,
    subscriptions: Vec<Subscription>,
}

impl Admin {
    async fn start(addr: SocketAddr, path: &Path) -> Self {
        Self::start_with(addr, path, ADMIN_TOKEN).await
    }

    async fn start_with(addr: SocketAddr, path: &Path, token: &str) -> Self {
        drop(db::migrate(path).await.unwrap());
        let store = db::open(path).await.unwrap();
        let db = Db::from(store.clone());
        let api = ApiConfig {
            events: addr.to_string(),
            token: Some(config::Secret::new(token)),
            tls_ca: None,
            tls_server_name: None,
        };
        let events = Events::connect(&api, store).unwrap();
        let subscriptions = subscriptions::start(events.executor(), &db).await.unwrap();
        Self {
            db,
            events,
            subscriptions,
        }
    }

    /// Waits until admin counts `expected` users.
    async fn counts(&self, expected: u64) {
        eventually(&format!("admin to count {expected} users"), || async {
            (users::count(&self.db).await.unwrap() == expected).then_some(())
        })
        .await;
    }

    /// The pages, their stylesheet at an address of the tests.
    fn router(&self) -> Router {
        let manifest = Manifest {
            version: MANIFEST_VERSION,
            assets: vec![ManifestEntry {
                id: app::STYLESHEET.id(),
                file: "tailwind.css".to_owned(),
                hash: String::new(),
                content_type: "text/css".to_owned(),
            }],
        };
        app::router(self.db.clone(), self.events.clone())
            .assets(AssetConfig::hosted_at("https://assets.test", manifest))
            .build()
    }

    async fn stop(self) {
        subscriptions::shutdown(self.subscriptions).await;
    }
}

/// `GET path`: the status and the body.
async fn get(router: &Router, path: &str) -> (StatusCode, String) {
    let request = Request::builder().uri(path).body(Body::empty()).unwrap();
    let response = router.handle(request).await;
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

fn dir() -> TempDir {
    tempfile::tempdir().unwrap()
}

#[tokio::test]
async fn admin_counts_the_users_from_scratch_then_as_events_come() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    // Registered before admin ever ran.
    let alice = api.register("alice@example.com").await;
    api.register("bob@example.com").await;

    let admin = Admin::start(api.addr, &dir.path().join("admin.db")).await;
    admin.counts(2).await;

    let carol = api.register("carol@example.com").await;
    admin.counts(3).await;
    // A user who changes is the same user.
    api.rename(&carol, "carol@example.com", "Carol").await;
    api.delete(&alice).await;
    admin.counts(2).await;
    assert_eq!(admin.events.state(), "connected");

    admin.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn admin_that_was_away_catches_up_from_where_it_stopped() {
    let dir = dir();
    let store = dir.path().join("admin.db");
    let api = Api::start(dir.path()).await;
    let alice = api.register("alice@example.com").await;

    let admin = Admin::start(api.addr, &store).await;
    admin.counts(1).await;
    admin.stop().await;

    // Nobody follows.
    api.register("bob@example.com").await;
    api.register("carol@example.com").await;
    api.delete(&alice).await;

    // The same store: what it counted is there, and its cursor.
    let admin = Admin::start(api.addr, &store).await;
    assert_eq!(users::count(&admin.db).await.unwrap(), 1);
    admin.counts(2).await;

    admin.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn admin_starts_without_the_api_and_catches_up_when_it_is_back() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    api.register("alice@example.com").await;
    api.stop_listening().await;

    // Nothing listens where admin dials.
    let admin = Admin::start(api.addr, &dir.path().join("admin.db")).await;
    assert!(
        !admin
            .events
            .reached(Duration::from_millis(200))
            .await
            .unwrap()
    );
    assert_eq!(admin.events.state(), "connecting");
    assert_eq!(users::count(&admin.db).await.unwrap(), 0);
    let (status, _) = get(&admin.router(), "/").await;
    assert_eq!(status, StatusCode::OK, "the pages are served all the same");

    api.listen_again().await;
    admin.counts(1).await;

    // The listener goes away under admin, which keeps running.
    api.stop_listening().await;
    api.register("bob@example.com").await;
    api.listen_again().await;
    admin.counts(2).await;

    admin.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn admin_is_told_when_the_api_does_not_know_its_token() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    api.register("alice@example.com").await;

    let admin = Admin::start_with(
        api.addr,
        &dir.path().join("admin.db"),
        "not the token of admin, whatever its length",
    )
    .await;
    let err = admin.events.reached(SETTLE).await.unwrap_err();
    assert!(format!("{err:#}").contains("api.token"), "{err:#}");
    assert_eq!(admin.events.state(), "refused");
    assert_eq!(users::count(&admin.db).await.unwrap(), 0);

    admin.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn the_dashboard_says_what_admin_followed() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    api.register("alice@example.com").await;
    api.register("bob@example.com").await;
    let admin = Admin::start(api.addr, &dir.path().join("admin.db")).await;
    admin.counts(2).await;
    let router = admin.router();

    let (status, page) = get(&router, "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.starts_with("<!DOCTYPE html><html lang=\"en\""),
        "{page}"
    );
    assert!(page.contains("<title>imkitchen admin</title>"), "{page}");
    assert!(page.contains("data-users=\"2\""), "{page}");
    assert!(
        page.contains("href=\"https://assets.test/tailwind.css\""),
        "{page}"
    );
    // No search engine is to list a page of admin.
    assert!(page.contains("content=\"noindex, nofollow\""), "{page}");

    // Nothing else is a page.
    let (status, page) = get(&router, "/nowhere").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(page.contains("This page does not exist"), "{page}");

    admin.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn health_says_how_admin_stands_with_the_api() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    let admin = Admin::start(api.addr, &dir.path().join("admin.db")).await;
    assert!(admin.events.reached(SETTLE).await.unwrap());

    let (status, body) = get(&admin.router(), "/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"status":"ok","api":"connected"}"#);

    admin.stop().await;
    api.shutdown().await;
}
