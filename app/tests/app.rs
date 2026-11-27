//! app against an api: its sidecar following the api's listener into a
//! store of its own, from scratch, as events come and after either was away,
//! and the pages read from what it followed.

use std::future::Future;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use evento::remote::ServerHandle;
use evento::subscription::Subscription;
use imkitchen_aggregates::recipe::{Created, Deleted, Ingredient, Instruction};
use imkitchen_api::cluster::{self, Cluster, Executor};
use imkitchen_api::config::{Secret, SidecarTokens, SidecarsConfig};
use imkitchen_api::sidecars;
use imkitchen_app::config::ApiConfig;
use imkitchen_app::db::{self, Db};
use imkitchen_app::events::Events;
use imkitchen_app::{app, config, recipes, subscriptions};
use tempfile::TempDir;
use tokio::net::TcpListener;
use topcoat::asset::{
    AssetConfig, MANIFEST_VERSION, Manifest, ManifestEntry, RouterBuilderAssetExt,
};
use topcoat::router::header::{ACCEPT_LANGUAGE, LOCATION};
use topcoat::router::request::Request;
use topcoat::router::{Body, Router, StatusCode, to_bytes};

const APP_TOKEN: &str = "the token of app in the tests, 32 characters";

/// The screens, as their addresses, with their names in English and in
/// French.
const SCREENS: [(&str, &str, &str); 5] = [
    ("/recipes", "Recipes", "Recettes"),
    ("/dishes", "Dishes", "Plats"),
    ("/list", "List", "Liste"),
    ("/groceries", "Groceries", "Courses"),
    ("/settings", "Settings", "Réglages"),
];

/// How long a test waits for what app does on its own time.
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
                admin: None,
                app: Some(Secret::new(APP_TOKEN)),
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
                instructions: vec![Instruction {
                    text: "Mix everything.".to_owned(),
                    timer_minutes: None,
                }],
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

/// app over the store at `path`, following the api at `addr`.
struct App {
    db: Db,
    events: Events,
    subscriptions: Vec<Subscription>,
}

impl App {
    async fn start(addr: SocketAddr, path: &Path) -> Self {
        Self::start_with(addr, path, APP_TOKEN).await
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

    /// Waits until app counts `expected` recipes.
    async fn counts(&self, expected: u64) {
        eventually(&format!("app to count {expected} recipes"), || async {
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
async fn app_counts_the_recipes_from_scratch_then_as_events_come() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    // Written before app ever ran.
    let soup = api.write_recipe("Soup").await;
    api.write_recipe("Pancakes").await;

    let app = App::start(api.addr, &dir.path().join("app.db")).await;
    app.counts(2).await;

    api.write_recipe("Tart").await;
    app.counts(3).await;
    api.delete_recipe(&soup).await;
    app.counts(2).await;
    assert_eq!(app.events.state(), "connected");

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn app_that_was_away_catches_up_from_where_it_stopped() {
    let dir = dir();
    let store = dir.path().join("app.db");
    let api = Api::start(dir.path()).await;
    let soup = api.write_recipe("Soup").await;

    let app = App::start(api.addr, &store).await;
    app.counts(1).await;
    app.stop().await;

    // Nobody follows.
    api.write_recipe("Pancakes").await;
    api.write_recipe("Tart").await;
    api.delete_recipe(&soup).await;

    // The same store: what it counted is there, and its cursor.
    let app = App::start(api.addr, &store).await;
    assert_eq!(recipes::count(&app.db).await.unwrap(), 1);
    app.counts(2).await;

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn app_starts_without_the_api_and_catches_up_when_it_is_back() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    api.write_recipe("Soup").await;
    api.stop_listening().await;

    // Nothing listens where app dials.
    let app = App::start(api.addr, &dir.path().join("app.db")).await;
    assert!(
        !app.events
            .reached(Duration::from_millis(200))
            .await
            .unwrap()
    );
    assert_eq!(app.events.state(), "connecting");
    assert_eq!(recipes::count(&app.db).await.unwrap(), 0);
    let (status, _, _) = get(&app.router(), "/", "en").await;
    assert_eq!(status, StatusCode::OK, "the pages are served all the same");

    api.listen_again().await;
    app.counts(1).await;

    // The listener goes away under app, which keeps running.
    api.stop_listening().await;
    api.write_recipe("Pancakes").await;
    api.listen_again().await;
    app.counts(2).await;

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn app_is_told_when_the_api_does_not_know_its_token() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    api.write_recipe("Soup").await;

    let app = App::start_with(
        api.addr,
        &dir.path().join("app.db"),
        "not the token of app, whatever its length is",
    )
    .await;
    let err = app.events.reached(SETTLE).await.unwrap_err();
    assert!(format!("{err:#}").contains("api.token"), "{err:#}");
    assert_eq!(app.events.state(), "refused");
    assert_eq!(recipes::count(&app.db).await.unwrap(), 0);

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn health_says_how_app_stands_with_the_api() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    let app = App::start(api.addr, &dir.path().join("app.db")).await;
    assert!(app.events.reached(SETTLE).await.unwrap());

    let (status, _, body) = get(&app.router(), "/health", "en").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"status":"ok","api":"connected"}"#);

    app.stop().await;
    api.shutdown().await;
}

/// The link of the navigation to `address`, when the page has one.
fn link<'a>(page: &'a str, address: &str) -> Option<&'a str> {
    let at = page.find(&format!("href=\"{address}\""))?;
    let from = page[..at].rfind("<a ")?;
    let to = at + page[at..].find("</a>")?;
    Some(&page[from..to])
}

#[tokio::test]
async fn the_home_is_read_from_what_app_followed() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    api.write_recipe("Soup").await;
    api.write_recipe("Pancakes").await;
    let app = App::start(api.addr, &dir.path().join("app.db")).await;
    app.counts(2).await;

    let (status, _, page) = get(&app.router(), "/", "en").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.starts_with("<!DOCTYPE html><html lang=\"en\""),
        "{page}"
    );
    assert!(page.contains("Your kitchen"), "{page}");
    assert!(page.contains("data-recipes=\"2\""), "{page}");
    assert!(page.contains("recipes are kept in imkitchen."), "{page}");
    assert!(
        page.contains("href=\"https://assets.test/tailwind.css\""),
        "{page}"
    );
    // A phone lays the page out at its own width, and no search engine is
    // to list it.
    assert!(page.contains("name=\"viewport\""), "{page}");
    assert!(page.contains("content=\"noindex, nofollow\""), "{page}");

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn every_screen_is_reached_from_every_other() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    let app = App::start(api.addr, &dir.path().join("app.db")).await;
    let router = app.router();

    // The home is no screen of the navigation: its name leads to it.
    let (_, _, page) = get(&router, "/", "en").await;
    assert!(!page.contains("aria-current"), "{page}");

    for (address, name, _) in SCREENS {
        let (status, _, page) = get(&router, address, "en").await;
        assert_eq!(status, StatusCode::OK, "{address}");
        assert!(
            page.contains(&format!("<title>{name} — imkitchen</title>")),
            "{page}"
        );
        assert!(page.contains(&format!(">{name}</h1>")), "{page}");
        assert!(link(&page, "/").is_some(), "{page}");
        // Every screen is a link away, and the one being read says so.
        for (other, name, _) in SCREENS {
            let link = link(&page, other).unwrap_or_else(|| panic!("no {other} in {page}"));
            assert!(link.ends_with(&format!(">{name}")), "{link}");
            assert_eq!(
                link.contains("aria-current=\"page\""),
                other == address,
                "{other} on {address}: {link}"
            );
        }
    }

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn the_pages_are_read_in_the_language_of_the_browser() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    let app = App::start(api.addr, &dir.path().join("app.db")).await;
    let router = app.router();

    let (_, _, page) = get(&router, "/", "fr-FR,fr;q=0.9,en;q=0.8").await;
    assert!(page.contains("<html lang=\"fr\""), "{page}");
    assert!(page.contains("Votre cuisine"), "{page}");
    assert!(
        page.contains("Les premières recettes sont en train d’être écrites."),
        "{page}"
    );
    for (address, _, name) in SCREENS {
        let link = link(&page, address).unwrap();
        assert!(link.ends_with(&format!(">{name}")), "{link}");
        // The same address in each language.
        let (status, _, page) = get(&router, address, "fr").await;
        assert_eq!(status, StatusCode::OK, "{address}");
        assert!(page.contains(&format!(">{name}</h1>")), "{page}");
    }

    // English when app speaks none of the languages the browser reads.
    let (_, _, page) = get(&router, "/", "de").await;
    assert!(page.contains("<html lang=\"en\""), "{page}");
    assert!(page.contains("Your kitchen"), "{page}");

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn an_address_of_nothing_is_not_found_within_the_layout() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    let app = App::start(api.addr, &dir.path().join("app.db")).await;
    let router = app.router();

    for path in ["/nowhere", "/recipes/nowhere/at/all"] {
        let (status, _, page) = get(&router, path, "fr").await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
        assert!(page.contains("Cette page n’existe pas"), "{page}");
        // The navigation is still there to leave by.
        for (address, _, _) in SCREENS {
            assert!(link(&page, address).is_some(), "{address} in {page}");
        }
    }

    app.stop().await;
    api.shutdown().await;
}
