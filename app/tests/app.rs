//! app against an api: its sidecar following the api's listener into a
//! store of its own, from scratch, as events come and after either was away,
//! the pages read from what it followed, who the api lets read them, how a
//! browser keeps its session, and what a user does, handed to the api.

use std::future::Future;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use evento::remote::ServerHandle;
use evento::subscription::Subscription;
use imkitchen_aggregates::recipe::{Created, Deleted, Ingredient, Instruction};
use imkitchen_api::auth::Auth;
use imkitchen_api::cluster::{self, Cluster, Executor};
use imkitchen_api::config::{AuthConfig, RootConfig, Secret, SidecarTokens, SidecarsConfig};
use imkitchen_api::images::Images;
use imkitchen_api::mail::{MailError, Mailer, code_mails_subscription};
use imkitchen_api::recipes::Web;
use imkitchen_api::server::{self, AppState};
use imkitchen_api::sidecars;
use imkitchen_app::api::{Api as Commands, Asking, CommandError};
use imkitchen_app::config::ApiConfig;
use imkitchen_app::db::{self, Db};
use imkitchen_app::events::Events;
use imkitchen_app::session::{MARKER, Sessions};
use imkitchen_app::{app, assets, config, recipes, subscriptions};
use serde_json::{Value, json};
use tempfile::TempDir;
use timada_identity::CodeMail;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use topcoat::asset::{
    AssetBundle, AssetConfig, MANIFEST_VERSION, Manifest, ManifestEntry, RouterBuilderAssetExt,
};
use topcoat::context::BaseUrl;
use topcoat::cookie::Key;
use topcoat::router::header::{
    ACCEPT_LANGUAGE, CACHE_CONTROL, CONTENT_TYPE, COOKIE, LOCATION, SET_COOKIE,
};
use topcoat::router::request::Request;
use topcoat::router::{Body, Router, RouterBuilder, StatusCode, to_bytes};

const APP_TOKEN: &str = "the token of app in the tests, 32 characters";

/// What the cookies of the tests are encrypted under.
const SESSION_KEY: &[u8] = b"the key of the sessions in the tests, 32 bytes or more";

/// An address of the api nothing answers at.
const NOWHERE: &str = "http://127.0.0.1:1";

/// The root account of the api of the tests: the one that signs in without
/// a mailed code.
const ROOT_EMAIL: &str = "root@example.com";
const ROOT_PASSWORD: &str = "root test password";
/// The argon2 hash of [`ROOT_PASSWORD`].
const ROOT_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$diXOV+PTSdOCfHj2wDAlMA$iVEeQUJsww/h81mHmrcmcJRWuZs2mY3bopNrchI/yog";

/// A password that passes the policy.
const PASSWORD: &str = "correct horse battery";

/// Where www always answers, in the tests: where the public pages are
/// linked.
const ABOUT: &str = "https://about.test";

/// What tells whether app is up.
const HEALTH: &str = "/_app/health";

/// Where a visitor signs in.
const SIGN_IN: &str = "/auth/sign-in";
const REGISTER: &str = "/auth/register";
const SIGN_OUT: &str = "/auth/sign-out";

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

/// Where the api's mails land instead of a server.
#[derive(Default)]
struct Outbox(Mutex<Vec<CodeMail>>);

impl Outbox {
    /// The code of the `nth` mail sent to `email`, once it is there.
    async fn code(&self, email: &str, nth: usize) -> String {
        eventually(&format!("mail {nth} to {email}"), || async {
            let mails = self.0.lock().unwrap();
            let mut to = mails.iter().filter(|mail| mail.email == email);
            to.nth(nth).map(|mail| mail.code.clone())
        })
        .await
    }
}

#[async_trait::async_trait]
impl Mailer for Outbox {
    async fn send_code(&self, mail: &CodeMail) -> Result<(), MailError> {
        self.0.lock().unwrap().push(mail.clone());
        Ok(())
    }
}

/// The api's routes, served over HTTP as the api serves them, its codes
/// mailed into an outbox.
struct Http {
    url: String,
    client: reqwest::Client,
    outbox: Arc<Outbox>,
    /// What mails the codes.
    mailing: Subscription,
    stop: oneshot::Sender<()>,
    served: JoinHandle<()>,
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
    /// Its routes, once [`Api::serve_http`] opened them.
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

    /// Serves the routes over HTTP, root among the accounts, and answers
    /// where.
    async fn serve_http(&mut self) -> String {
        let auth = Arc::new(
            Auth::new(&AuthConfig {
                secret: None,
                session_lifetime_secs: 3600,
                root: Some(RootConfig {
                    email: ROOT_EMAIL.to_owned(),
                    password_hash: Secret::new(ROOT_PASSWORD_HASH),
                }),
            })
            .unwrap(),
        );
        let outbox = Arc::new(Outbox::default());
        let mailing =
            code_mails_subscription(&self.executor, outbox.clone(), auth.codes.clone(), "tests")
                .start(&self.executor)
                .await
                .unwrap();
        let state = AppState {
            cluster: self.cluster.clone(),
            executor: self.executor.clone(),
            db: imkitchen_api::db::Db::from(self.store.clone()),
            auth,
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
        self.http = Some(Http {
            url: url.clone(),
            client: reqwest::Client::new(),
            outbox,
            mailing,
            stop,
            served,
        });
        url
    }

    fn http(&self) -> &Http {
        self.http.as_ref().unwrap()
    }

    /// Asks the api over HTTP, as whoever `token` signs in: the status and
    /// what it answers.
    async fn call(
        &self,
        method: reqwest::Method,
        path: &str,
        token: Option<&str>,
        body: Value,
    ) -> (u16, Value) {
        let http = self.http();
        let mut request = http
            .client
            .request(method, format!("{}{path}", http.url))
            .json(&body);
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        let response = request.send().await.unwrap();
        let status = response.status().as_u16();
        (status, response.json().await.unwrap_or(Value::Null))
    }

    /// The code of the `nth` mail the api sent to `email`.
    async fn code(&self, email: &str, nth: usize) -> String {
        self.http().outbox.code(email, nth).await
    }

    /// Signs root in at the api itself; the token of its session.
    async fn root_token(&self) -> String {
        let (status, body) = self
            .call(
                reqwest::Method::POST,
                "/v1/auth/sign-in",
                None,
                json!({ "email": ROOT_EMAIL, "password": ROOT_PASSWORD }),
            )
            .await;
        assert_eq!(status, 201, "{body}");
        body["token"].as_str().unwrap().to_owned()
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
        if let Some(http) = self.http {
            http.mailing.shutdown().await.unwrap();
            let _ = http.stop.send(());
            http.served.await.unwrap();
        }
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
    /// Where it asks the api over HTTP.
    api: String,
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
            api: NOWHERE.to_owned(),
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
        self.router_with(false)
    }

    /// The pages, as app serves them when it is reached over HTTPS
    /// (`secure`) or not.
    fn router_with(&self, secure: bool) -> Router {
        self.pages(secure)
            .assets(AssetConfig::hosted_at(
                "https://assets.test",
                Self::stylesheet(),
            ))
            .build()
    }

    /// The pages, without their assets.
    fn pages(&self, secure: bool) -> RouterBuilder {
        // The same key each time: a browser of the tests keeps its session
        // from a router to the next.
        let sessions = Sessions {
            key: Key::derive_from(SESSION_KEY),
            secure,
        };
        app::router(
            self.db.clone(),
            self.events.clone(),
            self.commands(),
            sessions,
            app::Www(BaseUrl::new(ABOUT).unwrap()),
        )
    }

    /// What a bundle says of the stylesheet.
    fn stylesheet() -> Manifest {
        Manifest {
            version: MANIFEST_VERSION,
            assets: vec![ManifestEntry {
                id: app::STYLESHEET.id(),
                file: "tailwind.css".to_owned(),
                hash: String::new(),
                content_type: "text/css".to_owned(),
            }],
        }
    }

    /// The client app asks the api with.
    fn commands(&self) -> Commands {
        Commands::new(&ApiConfig {
            url: self.api.clone(),
            events: "127.0.0.1:1".to_owned(),
            token: None,
            tls_ca: None,
            tls_server_name: None,
        })
        .unwrap()
    }

    async fn stop(self) {
        subscriptions::shutdown(self.subscriptions).await;
    }
}

/// What a page answered.
struct Answer {
    status: StatusCode,
    /// Where it sends the browser on, if anywhere.
    location: String,
    /// The cookies it set, as it wrote them.
    set_cookies: Vec<String>,
    body: String,
}

impl Answer {
    /// Where it sends the browser on, when it does.
    fn sent_to(&self) -> (StatusCode, &str) {
        (self.status, self.location.as_str())
    }

    /// The cookie `name` as it was written, when it was.
    fn set_cookie(&self, name: &str) -> Option<&str> {
        self.set_cookies
            .iter()
            .map(String::as_str)
            .find(|cookie| cookie.starts_with(&format!("{name}=")))
    }
}

/// A browser: it reads a language, keeps the cookies it is given and sends
/// them back.
#[derive(Clone)]
struct Browser {
    /// What it says it reads (`Accept-Language`).
    language: String,
    /// By name, as `name=value`.
    cookies: Vec<(String, String)>,
}

impl Browser {
    fn reading(language: &str) -> Self {
        Self {
            language: language.to_owned(),
            cookies: Vec::new(),
        }
    }

    /// One that keeps `name=value`, wherever it has it from.
    fn keeping(mut self, name: &str, value: &str) -> Self {
        self.cookies
            .push((name.to_owned(), format!("{name}={value}")));
        self
    }

    fn keeps(&self, name: &str) -> bool {
        self.cookies.iter().any(|(kept, _)| kept == name)
    }

    async fn get(&mut self, router: &Router, path: &str) -> Answer {
        self.send(router, "GET", path, None).await
    }

    /// Posts `fields` to `path` as a form.
    async fn post(&mut self, router: &Router, path: &str, fields: &[(&str, &str)]) -> Answer {
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
        self.send(router, "POST", path, Some(form.join("&"))).await
    }

    /// Asks for `path`, with `form` as what a form posts when there is one.
    async fn send(
        &mut self,
        router: &Router,
        method: &str,
        path: &str,
        form: Option<String>,
    ) -> Answer {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header(ACCEPT_LANGUAGE, &self.language);
        if !self.cookies.is_empty() {
            let pairs: Vec<&str> = self.cookies.iter().map(|(_, pair)| pair.as_str()).collect();
            request = request.header(COOKIE, pairs.join("; "));
        }
        let body = match form {
            Some(form) => {
                request = request.header(CONTENT_TYPE, "application/x-www-form-urlencoded");
                Body::from(form)
            }
            None => Body::empty(),
        };
        let response = router.handle(request.body(body).unwrap()).await;
        let status = response.status();
        let header =
            |value: &topcoat::router::header::HeaderValue| value.to_str().unwrap().to_owned();
        let location = response
            .headers()
            .get(LOCATION)
            .map(header)
            .unwrap_or_default();
        let set_cookies: Vec<String> = response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .map(header)
            .collect();
        for cookie in &set_cookies {
            let pair = cookie.split(';').next().unwrap().trim();
            let (name, value) = pair.split_once('=').unwrap();
            self.cookies.retain(|(kept, _)| kept != name);
            if !value.is_empty() && !cookie.contains("Max-Age=0") {
                self.cookies.push((name.to_owned(), pair.to_owned()));
            }
        }
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        Answer {
            status,
            location,
            set_cookies,
            body: String::from_utf8(body.to_vec()).unwrap(),
        }
    }

    /// Fills the form at `path` (the one that signs in, or the one that
    /// registers), the code the api mails typed at the second step: `nth`
    /// is how many it mailed to `email` before.
    async fn fill(
        &mut self,
        router: &Router,
        api: &Api,
        path: &str,
        email: &str,
        password: &str,
        nth: usize,
    ) -> Answer {
        let credentials = [("email", email), ("password", password)];
        let answer = self.post(router, path, &credentials).await;
        if !answer.body.contains("data-auth=\"code-sent\"") {
            return answer;
        }
        assert_eq!(answer.status, StatusCode::OK);
        assert!(answer.body.contains("name=\"code\""), "{}", answer.body);
        let code = api.code(email, nth).await;
        let credentials = [("email", email), ("password", password), ("code", &code)];
        self.post(router, path, &credentials).await
    }

    /// A browser that reads `language`, signed in as root: the account
    /// that needs no code.
    async fn signed_in(router: &Router, language: &str) -> Self {
        let mut browser = Self::reading(language);
        let credentials = [("email", ROOT_EMAIL), ("password", ROOT_PASSWORD)];
        let answer = browser.post(router, SIGN_IN, &credentials).await;
        assert_eq!(
            answer.sent_to(),
            (StatusCode::SEE_OTHER, "/"),
            "{}",
            answer.body
        );
        browser
    }
}

/// `GET path` as a browser that reads `language` and keeps nothing asks
/// for it: the status, where it is sent on and the body.
async fn get(router: &Router, path: &str, language: &str) -> (StatusCode, String, String) {
    let answer = Browser::reading(language).get(router, path).await;
    (answer.status, answer.location, answer.body)
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
    let (status, _, _) = get(&app.router(), SIGN_IN, "en").await;
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

    let (status, _, body) = get(&app.router(), HEALTH, "en").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, r#"{"status":"ok","api":"connected"}"#);
    // `/health` of the main domain is www's: app has nothing there.
    let (status, location, _) = get(&app.router(), "/health", "en").await;
    assert_eq!(
        (status, location.as_str()),
        (StatusCode::SEE_OTHER, SIGN_IN)
    );

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn the_assets_are_at_addresses_of_app_alone() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    let app = App::start(api.addr, &dir.path().join("app.db")).await;
    // A bundle, as `topcoat asset bundle` leaves one next to the binary.
    let bundle = dir.path().join("assets");
    std::fs::create_dir(&bundle).unwrap();
    App::stylesheet()
        .save(bundle.join("manifest.toml"))
        .unwrap();
    std::fs::write(bundle.join("tailwind.css"), "body{}").unwrap();
    let router = assets::serve(app.pages(false), AssetBundle::load_dir(&bundle).unwrap()).build();

    // The pages of a visitor link to the stylesheet under `/_app`, which
    // the gateway routes to app whoever asks: `/_topcoat` of the main
    // domain is www's for a browser that is not signed in.
    let (status, _, page) = get(&router, SIGN_IN, "en").await;
    assert_eq!(status, StatusCode::OK);
    let stylesheet = "/_app/assets/tailwind.css";
    assert_eq!(
        stylesheet,
        format!("{}/tailwind.css", assets::PREFIX),
        "the gateways route /_app"
    );
    assert!(
        page.contains(&format!("<link rel=\"stylesheet\" href=\"{stylesheet}\">")),
        "{page}"
    );

    // It is served there, to anyone, and for good: its name changes with
    // what it holds.
    let request = Request::builder()
        .uri(stylesheet)
        .body(Body::empty())
        .unwrap();
    let response = router.handle(request).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[CONTENT_TYPE], "text/css");
    assert!(
        response.headers()[CACHE_CONTROL]
            .to_str()
            .unwrap()
            .contains("immutable")
    );
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"body{}");

    // And nowhere else.
    let (status, location, _) = get(&router, "/_topcoat/assets/tailwind.css", "en").await;
    assert_eq!(
        (status, location.as_str()),
        (StatusCode::SEE_OTHER, SIGN_IN)
    );

    app.stop().await;
    api.shutdown().await;
}

/// What the gateways in front of imkitchen route to app is said in their
/// own files: the marker by its name, and the two prefixes of what app
/// serves whoever asks.
#[test]
fn the_gateways_route_what_app_serves() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    for gateway in [
        ".docker/traefik/dynamic.yaml",
        "app/helm/templates/httproute.yaml",
    ] {
        let routes = std::fs::read_to_string(root.join(gateway)).unwrap();
        assert!(routes.contains(&format!("{MARKER}=")), "{gateway}");
        for prefix in ["/auth", "/_app"] {
            assert!(routes.contains(prefix), "{gateway}: {prefix}");
            assert!(
                [SIGN_IN, REGISTER, SIGN_OUT, HEALTH, assets::PREFIX]
                    .iter()
                    .any(|served| served.starts_with(&format!("{prefix}/"))),
                "{prefix}"
            );
        }
    }
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
    let mut api = Api::start(dir.path()).await;
    api.write_recipe("Soup").await;
    api.write_recipe("Pancakes").await;
    let mut app = App::start(api.addr, &dir.path().join("app.db")).await;
    app.api = api.serve_http().await;
    app.counts(2).await;
    let router = app.router();
    let mut reader = Browser::signed_in(&router, "en").await;

    let Answer {
        status, body: page, ..
    } = reader.get(&router, "/").await;
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
    // The public pages are www's, where it answers a signed-in user: the
    // main domain is app for them.
    assert!(page.contains(&format!("href=\"{ABOUT}/en\"")), "{page}");

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn every_screen_is_reached_from_every_other() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    let mut app = App::start(api.addr, &dir.path().join("app.db")).await;
    app.api = api.serve_http().await;
    let router = app.router();
    let mut reader = Browser::signed_in(&router, "en").await;

    // The home is no screen of the navigation: its name leads to it.
    let page = reader.get(&router, "/").await.body;
    assert!(!page.contains("aria-current"), "{page}");

    for (address, name, _) in SCREENS {
        let Answer {
            status, body: page, ..
        } = reader.get(&router, address).await;
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
    let mut api = Api::start(dir.path()).await;
    let mut app = App::start(api.addr, &dir.path().join("app.db")).await;
    app.api = api.serve_http().await;
    let router = app.router();
    let mut reader = Browser::signed_in(&router, "en").await;

    reader.language = "fr-FR,fr;q=0.9,en;q=0.8".to_owned();
    let page = reader.get(&router, "/").await.body;
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
        let Answer {
            status, body: page, ..
        } = reader.get(&router, address).await;
        assert_eq!(status, StatusCode::OK, "{address}");
        assert!(page.contains(&format!(">{name}</h1>")), "{page}");
    }

    // English when app speaks none of the languages the browser reads.
    reader.language = "de".to_owned();
    let page = reader.get(&router, "/").await.body;
    assert!(page.contains("<html lang=\"en\""), "{page}");
    assert!(page.contains("Your kitchen"), "{page}");

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn an_address_of_nothing_is_not_found_within_the_layout() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    let mut app = App::start(api.addr, &dir.path().join("app.db")).await;
    app.api = api.serve_http().await;
    let router = app.router();
    let mut reader = Browser::signed_in(&router, "en").await;

    reader.language = "fr".to_owned();
    for path in ["/nowhere", "/recipes/nowhere/at/all", "/auth/nowhere"] {
        let Answer {
            status, body: page, ..
        } = reader.get(&router, path).await;
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

#[tokio::test]
async fn nobody_reads_a_page_without_signing_in() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    let mut app = App::start(api.addr, &dir.path().join("app.db")).await;
    app.api = api.serve_http().await;
    let router = app.router();
    let mut visitor = Browser::reading("en");

    // A page, and what is the address of none: nothing tells them apart.
    for path in ["/", "/recipes", "/nowhere", "/auth/nowhere"] {
        let answer = visitor.get(&router, path).await;
        assert_eq!(answer.sent_to(), (StatusCode::SEE_OTHER, SIGN_IN), "{path}");
        assert!(answer.body.is_empty(), "{path}: {}", answer.body);
        assert!(answer.set_cookies.is_empty(), "{path}");
    }

    // What a visitor is shown is all under `/auth`, and each form leads to
    // the other.
    let page = visitor.get(&router, SIGN_IN).await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(
        page.body.contains("<title>Sign in — imkitchen</title>"),
        "{}",
        page.body
    );
    assert!(
        page.body.contains("action=\"/auth/sign-in\""),
        "{}",
        page.body
    );
    assert!(page.body.contains("name=\"password\""), "{}", page.body);
    assert!(!page.body.contains("name=\"code\""), "{}", page.body);
    assert!(
        page.body.contains("href=\"/auth/register\""),
        "{}",
        page.body
    );
    assert!(!page.body.contains("data-recipes"), "{}", page.body);
    // And leads back to the public pages, in the language it is read in.
    for public in ["/en", "/en/terms", "/en/privacy"] {
        assert!(
            page.body.contains(&format!("href=\"{ABOUT}{public}\"")),
            "{public}: {}",
            page.body
        );
    }
    assert!(
        page.body.contains("content=\"noindex, nofollow\""),
        "{}",
        page.body
    );
    visitor.language = "fr".to_owned();
    let page = visitor.get(&router, REGISTER).await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.body.contains("<html lang=\"fr\""), "{}", page.body);
    assert!(page.body.contains(">Créer un compte</h1>"), "{}", page.body);
    assert!(
        page.body.contains("action=\"/auth/register\""),
        "{}",
        page.body
    );
    assert!(
        page.body.contains("href=\"/auth/sign-in\""),
        "{}",
        page.body
    );
    assert!(
        page.body.contains(&format!("href=\"{ABOUT}/fr/terms\"")),
        "{}",
        page.body
    );

    // A cookie anyone can write is the session of nobody, and the marker
    // alone opens nothing: both are taken back, and the browser is sent to
    // the main domain, where the gateway then serves it www.
    let forged = Browser::reading("en").keeping("session", "anything");
    let marked = Browser::reading("en").keeping(MARKER, "1");
    let both = forged.clone().keeping(MARKER, "1");
    for mut browser in [forged, marked, both] {
        let kept: Vec<String> = browser
            .cookies
            .iter()
            .map(|(name, _)| name.clone())
            .collect();
        let answer = browser.get(&router, "/recipes").await;
        assert_eq!(answer.sent_to(), (StatusCode::SEE_OTHER, "/"), "{kept:?}");
        for name in &kept {
            let cookie = answer.set_cookie(name).unwrap_or_else(|| panic!("{name}"));
            assert!(cookie.contains("Max-Age=0"), "{cookie}");
            assert!(cookie.contains("Path=/"), "{cookie}");
        }
        assert!(browser.cookies.is_empty(), "{:?}", browser.cookies);
        // Which, without a gateway, is where it signs in.
        let answer = browser.get(&router, &answer.location).await;
        assert_eq!(
            answer.sent_to(),
            (StatusCode::SEE_OTHER, SIGN_IN),
            "{kept:?}"
        );
    }

    // Signing out who is not signed in sends them there too.
    let answer = Browser::reading("en").post(&router, SIGN_OUT, &[]).await;
    assert_eq!(answer.sent_to(), (StatusCode::SEE_OTHER, "/"));

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn a_visitor_registers_and_stays_signed_in() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    let mut app = App::start(api.addr, &dir.path().join("app.db")).await;
    app.api = api.serve_http().await;
    let router = app.router();
    let mut alice = Browser::reading("fr-FR,fr;q=0.9,en;q=0.8");

    // What the api refuses of a field is said under the field, as the api
    // words it in the language the page is read in. The address stays
    // typed, the password does not.
    let answer = alice
        .post(
            &router,
            REGISTER,
            &[("email", "not an address"), ("password", "short")],
        )
        .await;
    assert_eq!(
        answer.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        answer.body
    );
    let page = &answer.body;
    assert!(page.contains("data-code=\"validation_failed\""), "{page}");
    assert!(
        page.contains("data-field=\"email\">Cette adresse e-mail n&#39;est pas valide.</p>")
            || page.contains("data-field=\"email\">Cette adresse e-mail n'est pas valide.</p>"),
        "{page}"
    );
    assert!(
        page.contains(
            "data-field=\"password\">Le mot de passe doit contenir entre 8 et 128 caractères.</p>"
        ),
        "{page}"
    );
    assert!(page.contains("value=\"not an address\""), "{page}");
    assert!(!page.contains("short"), "{page}");
    assert!(answer.set_cookies.is_empty(), "{:?}", answer.set_cookies);

    // The address and a password, then the code the api mailed.
    let answer = alice
        .post(
            &router,
            REGISTER,
            &[("email", "alice@example.com"), ("password", PASSWORD)],
        )
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert!(
        answer.body.contains("data-auth=\"code-sent\""),
        "{}",
        answer.body
    );
    assert!(
        answer.body.contains("Un code a été envoyé"),
        "{}",
        answer.body
    );
    assert!(answer.body.contains("name=\"code\""), "{}", answer.body);
    assert!(!alice.keeps("session"), "nothing is kept before the code");

    // A wrong code is asked for again, in French.
    let answer = alice
        .post(
            &router,
            REGISTER,
            &[
                ("email", "alice@example.com"),
                ("password", PASSWORD),
                ("code", "000000"),
            ],
        )
        .await;
    assert!(answer.status.is_client_error(), "{}", answer.status);
    assert!(
        answer.body.contains("data-code=\"invalid_code\""),
        "{}",
        answer.body
    );
    assert!(answer.body.contains("Ce code n"), "{}", answer.body);
    assert!(answer.body.contains("name=\"code\""), "{}", answer.body);

    let code = api.code("alice@example.com", 0).await;
    let answer = alice
        .post(
            &router,
            REGISTER,
            &[
                ("email", "alice@example.com"),
                ("password", PASSWORD),
                ("code", &code),
            ],
        )
        .await;
    assert_eq!(
        answer.sent_to(),
        (StatusCode::SEE_OTHER, "/"),
        "{}",
        answer.body
    );

    // Two cookies, kept past the browser being closed, for as long as the
    // api says the session lasts. The session is the browser's alone: no
    // script reads it, no other site posts with it, and what it holds is
    // not the token. The marker says a session is there and nothing else.
    let session = answer.set_cookie("session").expect("the session");
    let marker = answer.set_cookie(MARKER).expect("the marker");
    assert_eq!(answer.set_cookies.len(), 2, "{:?}", answer.set_cookies);
    for cookie in [session, marker] {
        for attribute in ["HttpOnly", "SameSite=Lax", "Path=/"] {
            assert!(cookie.contains(attribute), "{attribute}: {cookie}");
        }
        assert!(!cookie.contains("Secure"), "{cookie}");
        let lasts: u64 = cookie
            .split("Max-Age=")
            .nth(1)
            .and_then(|rest| rest.split(';').next())
            .and_then(|secs| secs.trim().parse().ok())
            .unwrap_or_else(|| panic!("no Max-Age: {cookie}"));
        // The lifetime of a session of the api of the tests.
        assert!((3500..=3600).contains(&lasts), "{cookie}");
    }
    assert!(!session.contains("token"), "{session}");
    assert!(marker.starts_with("signed_in=1;"), "{marker}");

    // The pages are hers to read.
    let page = alice.get(&router, "/").await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.body.contains("Votre cuisine"), "{}", page.body);
    assert!(
        page.body.contains("data-account=\"alice@example.com\""),
        "{}",
        page.body
    );
    assert!(
        page.body.contains("action=\"/auth/sign-out\""),
        "{}",
        page.body
    );
    assert!(page.set_cookies.is_empty(), "{:?}", page.set_cookies);

    // Signed in, the two forms have nothing for her.
    for path in [SIGN_IN, REGISTER] {
        let answer = alice.get(&router, path).await;
        assert_eq!(answer.sent_to(), (StatusCode::SEE_OTHER, "/"), "{path}");
    }

    // A session never goes without its marker: lost, it is written again.
    let mut unmarked = alice.clone();
    unmarked.cookies.retain(|(name, _)| name != MARKER);
    let page = unmarked.get(&router, "/recipes").await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(unmarked.keeps(MARKER), "{:?}", page.set_cookies);

    // The address is taken: the api says so.
    let answer = Browser::reading("en")
        .post(
            &router,
            REGISTER,
            &[("email", "alice@example.com"), ("password", PASSWORD)],
        )
        .await;
    assert!(answer.status.is_client_error(), "{}", answer.status);
    assert!(
        answer.body.contains("data-auth=\"refused\""),
        "{}",
        answer.body
    );
    assert!(answer.set_cookies.is_empty(), "{:?}", answer.set_cookies);

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn a_user_signs_in_and_out() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    let mut app = App::start(api.addr, &dir.path().join("app.db")).await;
    app.api = api.serve_http().await;
    let router = app.router();
    let answer = Browser::reading("en")
        .fill(&router, &api, REGISTER, "bob@example.com", PASSWORD, 0)
        .await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);
    let mut bob = Browser::reading("fr");

    // A wrong password is refused as the api words it, in the language of
    // the page, and keeps nothing.
    let answer = bob
        .post(
            &router,
            SIGN_IN,
            &[
                ("email", "bob@example.com"),
                ("password", "not the password"),
            ],
        )
        .await;
    assert_eq!(
        answer.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        answer.body
    );
    assert!(
        answer.body.contains("data-code=\"invalid_credentials\""),
        "{}",
        answer.body
    );
    assert!(
        answer
            .body
            .contains("e-mail ou le mot de passe est incorrect."),
        "{}",
        answer.body
    );
    assert!(answer.body.contains("value=\"bob@example.com\""));
    assert!(!answer.body.contains("not the password"), "{}", answer.body);
    assert!(answer.set_cookies.is_empty(), "{:?}", answer.set_cookies);

    // His password, then the code the api mailed: the second to his
    // address, after the one he registered with.
    let answer = bob
        .fill(&router, &api, SIGN_IN, "bob@example.com", PASSWORD, 1)
        .await;
    assert_eq!(
        answer.sent_to(),
        (StatusCode::SEE_OTHER, "/"),
        "{}",
        answer.body
    );
    assert!(
        bob.keeps("session") && bob.keeps(MARKER),
        "{:?}",
        bob.cookies
    );
    let page = bob.get(&router, "/settings").await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(
        page.body.contains("data-account=\"bob@example.com\""),
        "{}",
        page.body
    );
    assert!(
        page.body.contains(">Se déconnecter</button>"),
        "{}",
        page.body
    );

    // Signed out, the browser keeps neither cookie and is sent to the main
    // domain; what it kept opens nothing: the session ended at the api,
    // and a browser that still has it gets both taken back.
    let signed_in = bob.clone();
    let answer = bob.post(&router, SIGN_OUT, &[]).await;
    assert_eq!(answer.sent_to(), (StatusCode::SEE_OTHER, "/"));
    assert!(bob.cookies.is_empty(), "{:?}", answer.set_cookies);
    let answer = bob.get(&router, "/").await;
    assert_eq!(answer.sent_to(), (StatusCode::SEE_OTHER, SIGN_IN));
    let mut replayed = signed_in;
    let answer = replayed.get(&router, "/recipes").await;
    assert_eq!(answer.sent_to(), (StatusCode::SEE_OTHER, "/"));
    assert!(replayed.cookies.is_empty(), "{:?}", answer.set_cookies);

    // Reached over HTTPS, app only ever has its cookies sent encrypted.
    let secure = app.router_with(true);
    let mut root = Browser::reading("en");
    let answer = root
        .post(
            &secure,
            SIGN_IN,
            &[("email", ROOT_EMAIL), ("password", ROOT_PASSWORD)],
        )
        .await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);
    for name in ["session", MARKER] {
        let cookie = answer.set_cookie(name).unwrap();
        assert!(cookie.contains("Secure"), "{cookie}");
    }

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn nobody_is_let_through_while_the_api_is_away() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    let mut app = App::start(api.addr, &dir.path().join("app.db")).await;
    app.api = api.serve_http().await;
    let mut root = Browser::signed_in(&app.router(), "fr").await;

    // Nothing answers where app asks who reads it.
    app.api = NOWHERE.to_owned();
    let router = app.router();
    let answer = root.get(&router, "/").await;
    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(!answer.body.contains("data-recipes"), "{}", answer.body);
    assert!(
        root.keeps("session") && root.keeps(MARKER),
        "the session is kept for later"
    );

    // Nor where it hands a form: the page says so in its own words.
    for path in [SIGN_IN, REGISTER] {
        let answer = Browser::reading("fr")
            .post(
                &router,
                path,
                &[("email", ROOT_EMAIL), ("password", ROOT_PASSWORD)],
            )
            .await;
        assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE, "{path}");
        assert!(
            answer.body.contains("data-auth=\"failed\""),
            "{}",
            answer.body
        );
        assert!(answer.body.contains("Réessayez dans un instant."), "{path}");
    }

    // Signing out forgets the session all the same.
    let answer = root.post(&router, SIGN_OUT, &[]).await;
    assert_eq!(answer.sent_to(), (StatusCode::SEE_OTHER, "/"));
    assert!(root.cookies.is_empty(), "{:?}", answer.set_cookies);

    app.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn what_a_user_does_is_handed_to_the_api() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    let mut app = App::start(api.addr, &dir.path().join("app.db")).await;
    app.api = api.serve_http().await;
    let commands = app.commands();
    let token = api.root_token().await;
    let name = async || {
        let (status, body) = api
            .call(
                reqwest::Method::GET,
                "/v1/account",
                Some(&token),
                Value::Null,
            )
            .await;
        assert_eq!(status, 200, "{body}");
        body["user"]["name"].as_str().unwrap().to_owned()
    };
    let profile = ["account", "profile"];
    let root = Asking {
        language: "fr",
        token: Some(&token),
    };

    // A command is sent as who asks, and the api does it.
    commands
        .command(
            root,
            reqwest::Method::PUT,
            &profile,
            Some(&json!({ "name": "Root of the tests" })),
        )
        .await
        .unwrap();
    assert_eq!(name().await, "Root of the tests");

    // What it refuses of a field comes back under the field, worded in
    // the language that was asked for, and nothing is written.
    let err = commands
        .command(
            root,
            reqwest::Method::PUT,
            &profile,
            Some(&json!({ "name": "n".repeat(256) })),
        )
        .await
        .unwrap_err();
    let CommandError::Refused(refusal) = err else {
        panic!("{err:?}");
    };
    assert_eq!(
        (refusal.status, refusal.code.as_str()),
        (422, "validation_failed")
    );
    assert_eq!(refusal.message, "Certains champs ne sont pas valides.");
    let field = refusal.field("name").expect("the name is refused");
    assert!(field.contains("255"), "{field}");
    assert_eq!(refusal.field("email"), None);
    assert!(!refusal.signed_out());
    assert_eq!(name().await, "Root of the tests");
    let english = Asking {
        language: "en",
        ..root
    };
    let err = commands
        .command(
            english,
            reqwest::Method::PUT,
            &profile,
            Some(&json!({ "name": "" })),
        )
        .await
        .unwrap_err();
    let CommandError::Refused(refusal) = err else {
        panic!("{err:?}");
    };
    assert_eq!(refusal.message, "Some fields are not valid.");

    // A command that takes no body, and one of nobody: the session ended.
    commands
        .command(
            root,
            reqwest::Method::POST,
            &["auth", "sign-out"],
            None::<&()>,
        )
        .await
        .unwrap();
    let err = commands
        .command(
            root,
            reqwest::Method::PUT,
            &profile,
            Some(&json!({ "name": "Nobody" })),
        )
        .await
        .unwrap_err();
    let CommandError::Refused(refusal) = err else {
        panic!("{err:?}");
    };
    assert!(refusal.signed_out(), "{refusal:?}");

    // An api that is away is not one that refused.
    app.api = NOWHERE.to_owned();
    let err = app
        .commands()
        .command(
            root,
            reqwest::Method::PUT,
            &profile,
            Some(&json!({ "name": "Nobody" })),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, CommandError::Unreachable(_)), "{err:?}");

    app.stop().await;
    api.shutdown().await;
}
