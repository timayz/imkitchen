//! www against an api: its sidecar following the api's listener into a
//! store of its own, from scratch, as events come and after either was away,
//! and the pages read from what it followed.

use std::future::Future;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use evento::metadata::Event;
use evento::remote::ServerHandle;
use evento::subscription::{Context, Subscription, SubscriptionBuilder};
use imkitchen_aggregates::contact::Sent;
use imkitchen_aggregates::recipe::{Created, Deleted, Ingredient};
use imkitchen_api::auth::Auth;
use imkitchen_api::cluster::{self, Cluster, Executor};
use imkitchen_api::config::{AuthConfig, Secret, SidecarTokens, SidecarsConfig};
use imkitchen_api::images::Images;
use imkitchen_api::recipes::Web;
use imkitchen_api::server::{self, AppState};
use imkitchen_api::sidecars;
use imkitchen_www::api::Api as Commands;
use imkitchen_www::config::{ApiConfig, AppConfig, ContactConfig};
use imkitchen_www::db::{self, Db};
use imkitchen_www::events::Events;
use imkitchen_www::{app, config, recipes, subscriptions};
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use topcoat::asset::{
    AssetConfig, MANIFEST_VERSION, Manifest, ManifestEntry, RouterBuilderAssetExt,
};
use topcoat::context::BaseUrl;
use topcoat::router::header::{ACCEPT_LANGUAGE, CONTENT_TYPE, LOCATION};
use topcoat::router::request::Request;
use topcoat::router::{Body, Router, StatusCode, to_bytes};

const WWW_TOKEN: &str = "the token of www in the tests, 32 characters";

/// Where the visitors of the tests reach www, and where they write.
const SITE: &str = "https://www.test";
const CONTACT: &str = "hello@www.test";

/// An address of the api's commands nothing answers at.
const NOWHERE: &str = "http://127.0.0.1:1";

/// The pages of a language, as what follows its code in their address.
const PAGES: [&str; 4] = ["", "/terms", "/privacy", "/contact"];

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

/// What was written to imkitchen, as the api kept it: who, where they are
/// answered, what they wrote and the language they read.
type Messages = Arc<Mutex<Vec<[String; 4]>>>;

#[evento::subscription]
async fn on_message_sent<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Sent>,
) -> anyhow::Result<()> {
    let messages: Messages = context.extract();
    let Sent {
        name,
        email,
        message,
        language,
    } = event.data;
    messages
        .lock()
        .unwrap()
        .push([name, email, message, language]);
    Ok(())
}

/// The api's commands, served over HTTP as the api serves them.
struct Http {
    stop: oneshot::Sender<()>,
    served: JoinHandle<()>,
    /// The messages it took, as they come.
    messages: Messages,
    following: Subscription,
}

/// An api node alone, over a store created the way a deployment creates it,
/// serving the sidecars as the api does.
struct Api {
    cluster: Arc<Cluster>,
    executor: Executor,
    store: imkitchen_api::db::Store,
    /// Where it keeps its files.
    dir: std::path::PathBuf,
    /// The sidecars' listener, while there is one.
    server: Option<ServerHandle>,
    addr: SocketAddr,
    /// Its commands, once [`Api::serve_commands`] opened them.
    http: Option<Http>,
}

impl Api {
    async fn start(dir: &Path) -> Self {
        let path = dir.join("imkitchen.db");
        drop(imkitchen_api::db::migrate(&path).await.unwrap());
        let store = imkitchen_api::db::open(&path).await.unwrap();
        let (cluster, executor) = cluster::open(None, None, &path, store.clone())
            .await
            .unwrap();
        let (server, addr) = Self::listen(&executor, "127.0.0.1:0".parse().unwrap()).await;
        Self {
            cluster,
            executor,
            store,
            dir: dir.to_owned(),
            server: Some(server),
            addr,
            http: None,
        }
    }

    /// Serves the commands over HTTP, and answers where.
    async fn serve_commands(&mut self) -> String {
        let auth = AuthConfig {
            secret: None,
            session_lifetime_secs: 3600,
            root: None,
        };
        let state = AppState {
            cluster: self.cluster.clone(),
            executor: self.executor.clone(),
            db: imkitchen_api::db::Db::from(self.store.clone()),
            auth: Arc::new(Auth::new(&auth).unwrap()),
            sign_ins: Arc::default(),
            pages: Arc::new(Web::new().unwrap()),
            images: Arc::new(Images::new(self.dir.join("images"), None)),
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (stop, stopped) = oneshot::channel();
        let served = tokio::spawn(async move {
            let stopped = async {
                let _ = stopped.await;
            };
            server::serve(listener, state, Duration::from_secs(1), stopped)
                .await
                .unwrap();
        });
        let messages = Messages::default();
        let following = SubscriptionBuilder::new("tests/contact-messages")
            .data(messages.clone())
            .handler(on_message_sent())
            .start(&self.executor)
            .await
            .unwrap();
        self.http = Some(Http {
            stop,
            served,
            messages,
            following,
        });
        url
    }

    /// Waits until the api took `expected` messages, and answers them.
    async fn messages(&self, expected: usize) -> Vec<[String; 4]> {
        let messages = &self.http.as_ref().unwrap().messages;
        eventually(&format!("the api to take {expected} messages"), || async {
            let messages = messages.lock().unwrap();
            (messages.len() >= expected).then(|| messages.clone())
        })
        .await
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
        if let Some(http) = self.http {
            http.following.shutdown().await.unwrap();
            let _ = http.stop.send(());
            http.served.await.unwrap();
        }
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
    /// Where its commands go: nowhere anything answers, until a test says.
    commands: String,
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
            url: NOWHERE.to_owned(),
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
            commands: NOWHERE.to_owned(),
        }
    }

    /// Waits until www counts `expected` recipes.
    async fn counts(&self, expected: u64) {
        eventually(&format!("www to count {expected} recipes"), || async {
            (recipes::count(&self.db).await.unwrap() == expected).then_some(())
        })
        .await;
    }

    /// The pages, their stylesheet at an address of the tests, while no
    /// listing of the application is live.
    fn router(&self) -> Router {
        self.router_with(AppConfig::default())
    }

    /// The pages, the application got at `app`.
    fn router_with(&self, app: AppConfig) -> Router {
        let manifest = Manifest {
            version: MANIFEST_VERSION,
            assets: vec![ManifestEntry {
                id: app::STYLESHEET.id(),
                file: "tailwind.css".to_owned(),
                hash: String::new(),
                content_type: "text/css".to_owned(),
            }],
        };
        let site = app::Site {
            url: BaseUrl::new(SITE).unwrap(),
            app,
            contact: ContactConfig {
                email: CONTACT.to_owned(),
            },
        };
        let commands = Commands::new(&ApiConfig {
            url: self.commands.clone(),
            events: "127.0.0.1:1".to_owned(),
            token: None,
            tls_ca: None,
            tls_server_name: None,
        })
        .unwrap();
        app::router(self.db.clone(), self.events.clone(), commands, site)
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

/// Posts `fields` to `path` as a browser that reads `language` posts a
/// form: the status, where it is sent on and the body.
async fn post(
    router: &Router,
    path: &str,
    language: &str,
    fields: &[(&str, &str)],
) -> (StatusCode, String, String) {
    let encode = |text: &str| -> String {
        text.bytes()
            .map(|byte| match byte {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => char::from(byte).to_string(),
                _ => format!("%{byte:02X}"),
            })
            .collect()
    };
    let form: Vec<String> = fields
        .iter()
        .map(|(name, value)| format!("{name}={}", encode(value)))
        .collect();
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header(ACCEPT_LANGUAGE, language)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(form.join("&")))
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
    // And a page to itself in the other.
    let (_, _, page) = get(&router, "/fr/terms", "en").await;
    assert!(
        page.contains("href=\"/en/terms\" hreflang=\"en\""),
        "{page}"
    );

    // Nothing else under a language: its own "not found".
    let (status, _, page) = get(&router, "/fr/nowhere", "en").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(page.contains("Cette page n’existe pas"), "{page}");

    // A language www has no sentences for is no address, nor is another
    // way to write one it has.
    for path in ["/de", "/EN", "/en-US/terms"] {
        let (status, _, _) = get(&router, path, "de").await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path}");
    }

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

/// The words of `page`, tags and addresses included, in lower case.
fn words(page: &str) -> Vec<String> {
    page.to_lowercase()
        .split(|c: char| !c.is_alphabetic())
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect()
}

#[tokio::test]
async fn every_page_is_there_in_every_language() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    let www = Www::start(api.addr, &dir.path().join("www.db")).await;
    let router = www.router();

    // What the rewrite dropped, in the languages www speaks.
    let dropped = [
        "ad",
        "ads",
        "advert",
        "adverts",
        "advertising",
        "advertisement",
        "advertisements",
        "premium",
        "subscription",
        "subscriptions",
        "subscribe",
        "pay",
        "paid",
        "payment",
        "payments",
        "billing",
        "share",
        "shared",
        "shares",
        "sharing",
        "chef",
        "chefs",
        "pub",
        "pubs",
        "publicité",
        "publicités",
        "abonnement",
        "abonnements",
        "abonner",
        "payer",
        "payant",
        "paiement",
        "paiements",
        "facturation",
        "partage",
        "partages",
        "partager",
        "partagé",
    ];

    for (language, title) in [("en", "Terms of use"), ("fr", "Conditions d’utilisation")] {
        let mut read = Vec::new();
        for page in PAGES {
            let path = format!("/{language}{page}");
            let (status, _, html) = get(&router, &path, "de").await;
            assert_eq!(status, StatusCode::OK, "{path}");
            assert!(
                html.starts_with(&format!("<!DOCTYPE html><html lang=\"{language}\"")),
                "{path}: {html}"
            );
            // No sentence is missing: one that is shows as its key.
            assert!(
                !html.contains("_title") && !html.contains("_text"),
                "{path}: {html}"
            );

            // What a search engine is told: the one address of the page,
            // and the page in each language.
            assert!(
                html.contains(&format!("<link rel=\"canonical\" href=\"{SITE}{path}\">")),
                "{path}: {html}"
            );
            for other in ["en", "fr"] {
                assert!(
                    html.contains(&format!(
                        "<link rel=\"alternate\" hreflang=\"{other}\" href=\"{SITE}/{other}{page}\">"
                    )),
                    "{path}: {html}"
                );
            }
            assert!(
                html.contains("<meta name=\"description\" content=\""),
                "{path}"
            );
            assert!(!html.contains("noindex"), "{path}");

            // Nothing tells anyone the page was read: no script, and
            // nothing fetched but the stylesheet.
            for fetching in [
                "<script",
                "<img",
                "<iframe",
                "<object",
                "<embed",
                "<video",
                "<audio",
                "ping=",
                "src=",
                "http-equiv",
            ] {
                assert!(!html.contains(fetching), "{path} holds {fetching}: {html}");
            }
            assert_eq!(
                html.matches("<link rel=\"stylesheet\"").count(),
                1,
                "{path}"
            );
            assert_eq!(html.matches("https://assets.test").count(), 1, "{path}");

            let words = words(&html);
            for word in dropped {
                assert!(
                    !words.iter().any(|read| read == word),
                    "{path} says \"{word}\": {html}"
                );
            }

            // Every page leads to the legal pages and to the contact.
            for linked in ["/terms", "/privacy", "/contact"] {
                assert!(
                    html.contains(&format!("href=\"/{language}{linked}\"")),
                    "{path} does not link to {linked}"
                );
            }
            read.push(html);
        }

        // The home alone is where a visitor without a language lands.
        let default = format!("<link rel=\"alternate\" hreflang=\"x-default\" href=\"{SITE}/\">");
        assert!(read[0].contains(&default), "{}", read[0]);
        assert!(!read[1].contains(&default), "{}", read[1]);

        assert!(
            read[1].contains(&format!(
                "<h1 class=\"text-4xl font-bold tracking-tight\">{title}</h1>"
            )),
            "{}",
            read[1]
        );
        // Who publishes imkitchen, and where they are written to.
        assert!(read[1].contains("SIRET"), "{}", read[1]);
        assert!(read[1].contains(CONTACT), "{}", read[1]);
        assert!(read[2].contains(CONTACT), "{}", read[2]);
        assert!(
            read[3].contains(&format!("href=\"mailto:{CONTACT}\"")),
            "{}",
            read[3]
        );
    }

    www.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn the_home_links_to_the_listings_that_are_live() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    let www = Www::start(api.addr, &dir.path().join("www.db")).await;

    // None is: the application is coming, and nothing is linked.
    let (_, _, page) = get(&www.router(), "/en", "en").await;
    assert!(page.contains("data-get=\"coming\""), "{page}");
    assert!(page.contains("href=\"/en#get\""), "{page}");
    assert!(!page.contains("Google Play"), "{page}");

    let play = "https://play.google.com/store/apps/details?id=app.imkitchen";
    let router = www.router_with(AppConfig {
        google_play: Some(play.to_owned()),
        app_store: None,
        desktop: Some("https://imkitchen.app/download".to_owned()),
    });
    let (_, _, page) = get(&router, "/fr", "fr").await;
    assert!(!page.contains("data-get=\"coming\""), "{page}");
    assert!(
        page.contains(&format!(
            "href=\"{play}\" rel=\"noopener\">Disponible sur Google Play</a>"
        )),
        "{page}"
    );
    assert!(
        page.contains("href=\"https://imkitchen.app/download\""),
        "{page}"
    );
    assert!(!page.contains("App Store"), "{page}");

    www.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn a_search_engine_is_given_every_page() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    let www = Www::start(api.addr, &dir.path().join("www.db")).await;
    let router = www.router();

    let (status, _, robots) = get(&router, "/robots.txt", "en").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        robots,
        format!("User-agent: *\nAllow: /\n\nSitemap: {SITE}/sitemap.xml\n")
    );

    let (status, _, sitemap) = get(&router, "/sitemap.xml", "en").await;
    assert_eq!(status, StatusCode::OK);
    let listed: Vec<&str> = sitemap
        .split("<loc>")
        .skip(1)
        .map(|rest| rest.split_once("</loc>").unwrap().0)
        .collect();
    assert_eq!(listed.len(), 2 * PAGES.len(), "{sitemap}");
    for language in ["en", "fr"] {
        for page in PAGES {
            let address = format!("{SITE}/{language}{page}");
            assert!(listed.contains(&address.as_str()), "{address}: {sitemap}");
        }
    }
    // What is listed is there.
    for address in listed {
        let (status, _, _) = get(&router, address.strip_prefix(SITE).unwrap(), "en").await;
        assert_eq!(status, StatusCode::OK, "{address}");
    }

    www.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn the_contact_form_is_a_command_of_the_api() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    let mut www = Www::start(api.addr, &dir.path().join("www.db")).await;
    www.commands = api.serve_commands().await;
    let router = www.router();

    let (status, _, page) = get(&router, "/fr/contact", "en").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        page.contains("method=\"post\" action=\"/fr/contact\""),
        "{page}"
    );

    // Taken by the api, as written and in the language of the page: the
    // visitor is sent on to a page that does not send it again.
    let written = [
        ("name", "Alice Dupont"),
        ("email", "Alice@Example.com"),
        (
            "message",
            "Bonjour,\nune question : où trouver l’application ?",
        ),
        ("website", ""),
    ];
    let (status, location, _) = post(&router, "/fr/contact", "en", &written).await;
    assert_eq!(
        (status, location.as_str()),
        (StatusCode::SEE_OTHER, "/fr/contact?sent=1")
    );
    assert_eq!(
        api.messages(1).await,
        [[
            "Alice Dupont".to_owned(),
            "alice@example.com".to_owned(),
            "Bonjour,\nune question : où trouver l’application ?".to_owned(),
            "fr".to_owned(),
        ]]
    );
    let (status, _, page) = get(&router, &location, "en").await;
    assert_eq!(status, StatusCode::OK);
    assert!(page.contains("data-contact=\"sent\""), "{page}");
    assert!(page.contains("Votre message est envoyé."), "{page}");
    assert!(!page.contains("<form"), "{page}");

    // What fills the field no person sees is not a person: it is told the
    // same, and the api is asked nothing.
    let (status, location, _) = post(
        &router,
        "/en/contact",
        "en",
        &[
            ("name", "A robot"),
            ("email", "robot@example.com"),
            ("message", "Buy."),
            ("website", "https://example.com"),
        ],
    )
    .await;
    assert_eq!(
        (status, location.as_str()),
        (StatusCode::SEE_OTHER, "/en/contact?sent=1")
    );

    // A second message of a person: the robot's never reached the api.
    let (status, _, _) = post(
        &router,
        "/en/contact",
        "fr",
        &[
            ("name", "Bob"),
            ("email", "bob@example.com"),
            ("message", "Hello!"),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let messages = api.messages(2).await;
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[1][0], "Bob");
    assert_eq!(messages[1][3], "en");

    www.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn what_the_api_refuses_is_shown_on_the_form() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    let mut www = Www::start(api.addr, &dir.path().join("www.db")).await;
    www.commands = api.serve_commands().await;
    let router = www.router();

    // Each field the api refuses says so in its words, in the language of
    // the page, over what the visitor typed.
    let (status, location, page) = post(
        &router,
        "/fr/contact",
        "en",
        &[
            ("name", "  "),
            ("email", "nobody"),
            ("message", "Une question <b>simple</b>"),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{page}");
    assert_eq!(location, "");
    assert!(page.contains("<html lang=\"fr\""), "{page}");
    assert!(
        page.contains(
            "<p id=\"contact-name-error\" class=\"mt-1 text-sm text-red-700\">\
             Ce champ doit contenir entre 1 et 100 caractères.</p>"
        ),
        "{page}"
    );
    assert!(page.contains("id=\"contact-email-error\""), "{page}");
    assert!(page.contains("adresse e-mail n"), "{page}");
    assert!(!page.contains("id=\"contact-message-error\""), "{page}");
    assert!(page.contains("value=\"nobody\""), "{page}");
    assert!(
        page.contains(">Une question &lt;b&gt;simple&lt;/b&gt;</textarea>"),
        "{page}"
    );
    assert!(
        page.contains("aria-invalid=\"true\" aria-describedby=\"contact-email-error\""),
        "{page}"
    );
    assert!(!page.contains("data-contact="), "{page}");

    // The same in English.
    let (status, _, page) = post(
        &router,
        "/en/contact",
        "fr",
        &[("name", "Alice"), ("email", "alice@example.com")],
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{page}");
    assert!(
        page.contains("This must be between 1 and 4000 characters."),
        "{page}"
    );
    assert!(page.contains("value=\"Alice\""), "{page}");

    // Nothing of it was kept.
    assert!(
        api.http
            .as_ref()
            .unwrap()
            .messages
            .lock()
            .unwrap()
            .is_empty()
    );

    www.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn a_message_the_api_cannot_be_handed_is_kept_on_the_form() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    // Its commands go where nothing answers.
    let www = Www::start(api.addr, &dir.path().join("www.db")).await;
    let router = www.router();

    let (status, _, page) = post(
        &router,
        "/en/contact",
        "en",
        &[
            ("name", "Alice"),
            ("email", "alice@example.com"),
            ("message", "Hello!"),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{page}");
    assert!(page.contains("data-contact=\"failed\""), "{page}");
    assert!(page.contains("Your message could not be sent."), "{page}");
    assert!(page.contains("value=\"Alice\""), "{page}");
    assert!(page.contains(">Hello!</textarea>"), "{page}");
    // The address to write to instead is still there.
    assert!(
        page.contains(&format!("href=\"mailto:{CONTACT}\"")),
        "{page}"
    );

    www.stop().await;
    api.shutdown().await;
}
