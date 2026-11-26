//! www against an api: its sidecar following the api's listener into a
//! store of its own, from scratch, as events come and after either was away,
//! and the pages read from what it followed.

use std::future::Future;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use evento::remote::ServerHandle;
use evento::subscription::Subscription;
use imkitchen_aggregates::recipe::{Created, Deleted, Ingredient};
use imkitchen_api::cluster::{self, Cluster, Executor};
use imkitchen_api::config::{Secret, SidecarTokens, SidecarsConfig};
use imkitchen_api::sidecars;
use imkitchen_www::config::ApiConfig;
use imkitchen_www::db::{self, Db};
use imkitchen_www::events::Events;
use imkitchen_www::{app, config, recipes, subscriptions};
use tempfile::TempDir;
use tokio::net::TcpListener;
use topcoat::asset::{
    AssetConfig, MANIFEST_VERSION, Manifest, ManifestEntry, RouterBuilderAssetExt,
};
use topcoat::router::header::{ACCEPT_LANGUAGE, LOCATION};
use topcoat::router::request::Request;
use topcoat::router::{Body, Router, StatusCode, to_bytes};

const WWW_TOKEN: &str = "the token of www in the tests, 32 characters";

/// How long a test waits for what www does on its own time.
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
                www: Some(Secret::new(WWW_TOKEN)),
                admin: None,
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

    /// Writes a recipe as the api does; its id.
    async fn write_recipe(&self, title: &str) -> String {
        evento::create()
            .event(&Created {
                owner_id: "alice".to_owned(),
                title: title.to_owned(),
                description: String::new(),
                ingredients: vec![Ingredient {
                    name: "flour".to_owned(),
                    quantity: Some(250.0),
                    unit: Some("g".to_owned()),
                }],
                instructions: vec!["Mix everything.".to_owned()],
                prep_minutes: 10,
                cook_minutes: 20,
                servings: 4,
            })
            .commit(&self.executor)
            .await
            .unwrap()
    }

    async fn delete_recipe(&self, id: &str) {
        evento::append(id)
            .original_version(1)
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

/// www over the store at `path`, following the api at `addr`.
struct Www {
    db: Db,
    events: Events,
    subscriptions: Vec<Subscription>,
}

impl Www {
    async fn start(addr: SocketAddr, path: &Path) -> Self {
        Self::start_with(addr, path, WWW_TOKEN).await
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

    /// Waits until www counts `expected` recipes.
    async fn counts(&self, expected: u64) {
        eventually(&format!("www to count {expected} recipes"), || async {
            (recipes::count(&self.db).await.unwrap() == expected).then_some(())
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

/// `GET path` as a browser that reads `language` asks for it: the status,
/// where it is sent on and the body.
async fn get(router: &Router, path: &str, language: &str) -> (StatusCode, String, String) {
    let request = Request::builder()
        .uri(path)
        .header(ACCEPT_LANGUAGE, language)
        .body(Body::empty())
        .unwrap();
    let response = router.handle(request).await;
    let status = response.status();
    let location = response
        .headers()
        .get(LOCATION)
        .map(|value| value.to_str().unwrap().to_owned())
        .unwrap_or_default();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, location, String::from_utf8(body.to_vec()).unwrap())
}

fn dir() -> TempDir {
    tempfile::tempdir().unwrap()
}

#[tokio::test]
async fn www_counts_the_recipes_from_scratch_then_as_events_come() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    // Written before www ever ran.
    let soup = api.write_recipe("Soup").await;
    api.write_recipe("Pancakes").await;

    let www = Www::start(api.addr, &dir.path().join("www.db")).await;
    www.counts(2).await;

    api.write_recipe("Tart").await;
    www.counts(3).await;
    api.delete_recipe(&soup).await;
    www.counts(2).await;
    assert_eq!(www.events.state(), "connected");

    www.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn www_that_was_away_catches_up_from_where_it_stopped() {
    let dir = dir();
    let store = dir.path().join("www.db");
    let api = Api::start(dir.path()).await;
    let soup = api.write_recipe("Soup").await;

    let www = Www::start(api.addr, &store).await;
    www.counts(1).await;
    www.stop().await;

    // Nobody follows.
    api.write_recipe("Pancakes").await;
    api.write_recipe("Tart").await;
    api.delete_recipe(&soup).await;

    // The same store: what it counted is there, and its cursor.
    let www = Www::start(api.addr, &store).await;
    assert_eq!(recipes::count(&www.db).await.unwrap(), 1);
    www.counts(2).await;

    www.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn www_starts_without_the_api_and_catches_up_when_it_is_back() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    api.write_recipe("Soup").await;
    api.stop_listening().await;

    // Nothing listens where www dials.
    let www = Www::start(api.addr, &dir.path().join("www.db")).await;
    assert!(
        !www.events
            .reached(Duration::from_millis(200))
            .await
            .unwrap()
    );
    assert_eq!(www.events.state(), "connecting");
    assert_eq!(recipes::count(&www.db).await.unwrap(), 0);
    let (status, _, _) = get(&www.router(), "/en", "en").await;
    assert_eq!(status, StatusCode::OK, "the pages are served all the same");

    api.listen_again().await;
    www.counts(1).await;

    // The listener goes away under www, which keeps running.
    api.stop_listening().await;
    api.write_recipe("Pancakes").await;
    api.listen_again().await;
    www.counts(2).await;

    www.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn www_is_told_when_the_api_does_not_know_its_token() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    api.write_recipe("Soup").await;

    let www = Www::start_with(
        api.addr,
        &dir.path().join("www.db"),
        "not the token of www, whatever its length is",
    )
    .await;
    let err = www.events.reached(SETTLE).await.unwrap_err();
    assert!(format!("{err:#}").contains("api.token"), "{err:#}");
    assert_eq!(www.events.state(), "refused");
    assert_eq!(recipes::count(&www.db).await.unwrap(), 0);

    www.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn the_pages_are_read_in_the_language_of_their_address() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    api.write_recipe("Soup").await;
    api.write_recipe("Pancakes").await;
    let www = Www::start(api.addr, &dir.path().join("www.db")).await;
    www.counts(2).await;
    let router = www.router();

    // `/` is the home of the language the browser prefers, English when www
    // speaks none of them.
    for (language, home) in [
        ("fr-FR,fr;q=0.9,en;q=0.8", "/fr"),
        ("en-GB", "/en"),
        ("de", "/en"),
    ] {
        let (status, location, _) = get(&router, "/", language).await;
        assert_eq!(
            (status, location.as_str()),
            (StatusCode::TEMPORARY_REDIRECT, home)
        );
    }

    // The address says the language, whatever the browser prefers.
    let (status, _, page) = get(&router, "/en", "fr").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.starts_with("<!DOCTYPE html><html lang=\"en\""),
        "{page}"
    );
    assert!(page.contains("Your own cookbook"), "{page}");
    assert!(page.contains("data-recipes=\"2\""), "{page}");
    assert!(page.contains("recipes are kept in imkitchen."), "{page}");
    assert!(
        page.contains("href=\"https://assets.test/tailwind.css\""),
        "{page}"
    );

    let (status, _, page) = get(&router, "/fr", "en").await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("<html lang=\"fr\""), "{page}");
    assert!(page.contains("Votre livre de cuisine"), "{page}");
    assert!(
        page.contains("recettes sont gardées dans imkitchen."),
        "{page}"
    );
    // Each language links to the other.
    assert!(page.contains("href=\"/en\" hreflang=\"en\""), "{page}");

    // Nothing else under a language: its own "not found".
    let (status, _, page) = get(&router, "/fr/nowhere", "en").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(page.contains("Cette page n’existe pas"), "{page}");

    // A language www has no sentences for is no address.
    let (status, _, _) = get(&router, "/de", "de").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    www.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn health_says_how_www_stands_with_the_api() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    let www = Www::start(api.addr, &dir.path().join("www.db")).await;
    assert!(www.events.reached(SETTLE).await.unwrap());

    let (status, _, body) = get(&www.router(), "/health", "en").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"status":"ok","api":"connected"}"#);

    www.stop().await;
    api.shutdown().await;
}
