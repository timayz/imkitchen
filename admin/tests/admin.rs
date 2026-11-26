//! admin against an api: its sidecar following the api's listener into a
//! store of its own, from scratch, as events come and after either was away,
//! the pages read from what it followed, and who the api lets read them.

use std::future::Future;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use evento::remote::ServerHandle;
use evento::subscription::Subscription;
use imkitchen_admin::api::Api as Accounts;
use imkitchen_admin::config::ApiConfig;
use imkitchen_admin::db::{self, Db};
use imkitchen_admin::events::Events;
use imkitchen_admin::session::Sessions;
use imkitchen_admin::{app, config, subscriptions, users};
use imkitchen_aggregates::identity::{Deleted, EmailRegistered, ProfileUpdated, User};
use imkitchen_api::auth::Auth;
use imkitchen_api::cluster::{self, Cluster, Executor};
use imkitchen_api::config::{AuthConfig, RootConfig, Secret, SidecarTokens, SidecarsConfig};
use imkitchen_api::images::Images;
use imkitchen_api::mail::{MailError, Mailer, code_mails_subscription};
use imkitchen_api::recipes::Web;
use imkitchen_api::server::{self, AppState};
use imkitchen_api::sidecars;
use serde_json::{Value, json};
use tempfile::TempDir;
use timada_identity::CodeMail;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use topcoat::asset::{
    AssetConfig, MANIFEST_VERSION, Manifest, ManifestEntry, RouterBuilderAssetExt,
};
use topcoat::cookie::Key;
use topcoat::router::header::{CONTENT_TYPE, COOKIE, LOCATION, SET_COOKIE};
use topcoat::router::request::Request;
use topcoat::router::{Body, Router, StatusCode, to_bytes};

const ADMIN_TOKEN: &str = "the token of admin in the tests, 32 characters";

/// What the cookies of the tests are encrypted under.
const SESSION_KEY: &[u8] = b"the key of the sessions in the tests, 32 bytes or more";

/// An address of the api nothing answers at.
const NOWHERE: &str = "http://127.0.0.1:1";

/// The root account of the api of the tests.
const ROOT_EMAIL: &str = "root@example.com";
const ROOT_PASSWORD: &str = "root test password";
/// The argon2 hash of [`ROOT_PASSWORD`].
const ROOT_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$diXOV+PTSdOCfHj2wDAlMA$iVEeQUJsww/h81mHmrcmcJRWuZs2mY3bopNrchI/yog";

/// A password that passes the policy.
const PASSWORD: &str = "correct horse battery";

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

    /// Creates the account of `email` with [`PASSWORD`], through the api's
    /// routes; its id.
    async fn sign_up(&self, email: &str) -> String {
        let credentials = json!({ "email": email, "password": PASSWORD });
        let (status, body) = self
            .call(
                reqwest::Method::POST,
                "/v1/auth/register",
                None,
                credentials.clone(),
            )
            .await;
        assert_eq!(status, 202, "{body}");
        let mut credentials = credentials;
        credentials["code"] = json!(self.code(email, 0).await);
        let (status, body) = self
            .call(
                reqwest::Method::POST,
                "/v1/auth/register",
                None,
                credentials,
            )
            .await;
        assert_eq!(status, 201, "{body}");
        body["user"]["id"].as_str().unwrap().to_owned()
    }

    /// Root makes the user `id` an administrator, or one no more.
    async fn set_role(&self, id: &str, role: &str) {
        let (status, body) = self
            .call(
                reqwest::Method::POST,
                "/v1/auth/sign-in",
                None,
                json!({ "email": ROOT_EMAIL, "password": ROOT_PASSWORD }),
            )
            .await;
        assert_eq!(status, 201, "{body}");
        let (status, body) = self
            .call(
                reqwest::Method::PUT,
                &format!("/v1/admin/users/{id}/role"),
                body["token"].as_str(),
                json!({ "role": role }),
            )
            .await;
        assert_eq!(status, 204, "{body}");
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

/// admin over the store at `path`, following the api at `addr`.
struct Admin {
    db: Db,
    events: Events,
    subscriptions: Vec<Subscription>,
    /// Where it asks the api who reads its pages: nowhere, until a test
    /// says.
    api: String,
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

    /// Waits until admin counts `expected` users.
    async fn counts(&self, expected: u64) {
        eventually(&format!("admin to count {expected} users"), || async {
            (users::count(&self.db).await.unwrap() == expected).then_some(())
        })
        .await;
    }

    /// The pages, their stylesheet at an address of the tests.
    fn router(&self) -> Router {
        self.router_with(false)
    }

    /// The pages, as admin serves them when it is reached over HTTPS
    /// (`secure`) or not.
    fn router_with(&self, secure: bool) -> Router {
        let api = Accounts::new(&ApiConfig {
            url: self.api.clone(),
            events: "127.0.0.1:1".to_owned(),
            token: None,
            tls_ca: None,
            tls_server_name: None,
        })
        .unwrap();
        // The same key each time: a browser of the tests keeps its session
        // from a router to the next.
        let sessions = Sessions {
            key: Key::derive_from(SESSION_KEY),
            secure,
        };
        let manifest = Manifest {
            version: MANIFEST_VERSION,
            assets: vec![ManifestEntry {
                id: app::STYLESHEET.id(),
                file: "tailwind.css".to_owned(),
                hash: String::new(),
                content_type: "text/css".to_owned(),
            }],
        };
        app::router(self.db.clone(), self.events.clone(), api, sessions)
            .assets(AssetConfig::hosted_at("https://assets.test", manifest))
            .build()
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

/// A browser: it keeps the cookies it is given, and sends them back.
#[derive(Default, Clone)]
struct Browser {
    /// By name, as `name=value`.
    cookies: Vec<(String, String)>,
}

impl Browser {
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
        let mut request = Request::builder().method(method).uri(path);
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

    /// Signs in through the form, the code the api mails typed at the
    /// second step when there is one: `nth` is how many it mailed to
    /// `email` before.
    async fn sign_in(
        &mut self,
        router: &Router,
        api: &Api,
        email: &str,
        password: &str,
        nth: usize,
    ) -> Answer {
        let credentials = [("email", email), ("password", password)];
        let answer = self.post(router, "/sign-in", &credentials).await;
        if !answer.body.contains("data-sign-in=\"code-sent\"") {
            return answer;
        }
        assert_eq!(answer.status, StatusCode::OK);
        assert!(answer.body.contains("name=\"code\""), "{}", answer.body);
        let code = api.code(email, nth).await;
        let credentials = [("email", email), ("password", password), ("code", &code)];
        self.post(router, "/sign-in", &credentials).await
    }
}

/// `GET path` as a browser that keeps nothing: the status and the body.
async fn get(router: &Router, path: &str) -> (StatusCode, String) {
    let answer = Browser::default().get(router, path).await;
    (answer.status, answer.body)
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
    let (status, _) = get(&admin.router(), "/sign-in").await;
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
    let mut api = Api::start(dir.path()).await;
    api.register("alice@example.com").await;
    api.register("bob@example.com").await;
    let mut admin = Admin::start(api.addr, &dir.path().join("admin.db")).await;
    admin.api = api.serve_http().await;
    let router = admin.router();
    let mut root = Browser::default();
    let answer = root
        .sign_in(&router, &api, ROOT_EMAIL, ROOT_PASSWORD, 0)
        .await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);
    // Root is a user too, once it signed in.
    admin.counts(3).await;

    let page = root.get(&router, "/").await;
    assert_eq!(page.status, StatusCode::OK);
    let page = page.body;
    assert!(
        page.starts_with("<!DOCTYPE html><html lang=\"en\""),
        "{page}"
    );
    assert!(page.contains("<title>imkitchen admin</title>"), "{page}");
    assert!(page.contains("data-users=\"3\""), "{page}");
    assert!(
        page.contains("href=\"https://assets.test/tailwind.css\""),
        "{page}"
    );
    // No search engine is to list a page of admin.
    assert!(page.contains("content=\"noindex, nofollow\""), "{page}");

    // Nothing else is a page.
    let page = root.get(&router, "/nowhere").await;
    assert_eq!(page.status, StatusCode::NOT_FOUND);
    assert!(
        page.body.contains("This page does not exist"),
        "{}",
        page.body
    );
    assert!(page.body.contains("data-account"), "{}", page.body);

    admin.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn nobody_reads_a_page_without_signing_in() {
    let dir = dir();
    let api = Api::start(dir.path()).await;
    let admin = Admin::start(api.addr, &dir.path().join("admin.db")).await;
    let router = admin.router();
    let mut visitor = Browser::default();

    // A page, and what is the address of none: nothing tells them apart.
    for path in ["/", "/nowhere", "/sign-in/nowhere"] {
        let answer = visitor.get(&router, path).await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER, "{path}");
        assert_eq!(answer.location, "/sign-in", "{path}");
        assert!(answer.body.is_empty(), "{path}: {}", answer.body);
    }
    // A cookie anyone can write is the session of nobody.
    visitor.cookies = vec![("session".to_owned(), "session=anything".to_owned())];
    let answer = visitor.get(&router, "/").await;
    assert_eq!(
        (answer.status, answer.location.as_str()),
        (StatusCode::SEE_OTHER, "/sign-in")
    );

    let page = visitor.get(&router, "/sign-in").await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.body.contains("action=\"/sign-in\""), "{}", page.body);
    assert!(page.body.contains("name=\"password\""), "{}", page.body);
    assert!(!page.body.contains("name=\"code\""), "{}", page.body);
    assert!(!page.body.contains("data-users"), "{}", page.body);
    assert!(
        page.body.contains("content=\"noindex, nofollow\""),
        "{}",
        page.body
    );

    // Signing out who is not signed in sends them to sign in.
    let answer = Browser::default().post(&router, "/sign-out", &[]).await;
    assert_eq!(
        (answer.status, answer.location.as_str()),
        (StatusCode::SEE_OTHER, "/sign-in")
    );

    admin.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn an_administrator_signs_in_and_out() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    let mut admin = Admin::start(api.addr, &dir.path().join("admin.db")).await;
    admin.api = api.serve_http().await;
    let router = admin.router();
    let mut root = Browser::default();

    // A wrong password is refused as the api words it, and keeps nothing.
    let answer = root
        .post(
            &router,
            "/sign-in",
            &[("email", ROOT_EMAIL), ("password", "not the password")],
        )
        .await;
    assert_eq!(
        answer.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        answer.body
    );
    assert!(
        answer.body.contains("data-sign-in=\"refused\""),
        "{}",
        answer.body
    );
    assert!(answer.body.contains("value=\"root@example.com\""));
    assert!(!answer.body.contains("not the password"), "{}", answer.body);
    assert!(answer.set_cookies.is_empty(), "{:?}", answer.set_cookies);
    assert_eq!(root.get(&router, "/").await.status, StatusCode::SEE_OTHER);

    // Root needs no code.
    let answer = root
        .post(
            &router,
            "/sign-in",
            &[("email", ROOT_EMAIL), ("password", ROOT_PASSWORD)],
        )
        .await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);
    assert_eq!(answer.location, "/");
    // The cookie is the browser's alone: no script reads it, no other site
    // posts with it, and what it holds is not the token.
    let [cookie] = &answer.set_cookies[..] else {
        panic!("{:?}", answer.set_cookies);
    };
    assert!(cookie.starts_with("session="), "{cookie}");
    for attribute in ["HttpOnly", "SameSite=Lax", "Path=/", "Max-Age="] {
        assert!(cookie.contains(attribute), "{attribute}: {cookie}");
    }
    assert!(!cookie.contains("Secure"), "{cookie}");
    assert!(!cookie.contains("token"), "{cookie}");

    let page = root.get(&router, "/").await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(
        page.body.contains("data-account=\"root@example.com\""),
        "{}",
        page.body
    );
    assert!(page.body.contains("action=\"/sign-out\""), "{}", page.body);

    // Signed out, the browser keeps nothing, and what it kept opens
    // nothing: the session ended at the api.
    let signed_in = root.clone();
    let answer = root.post(&router, "/sign-out", &[]).await;
    assert_eq!(
        (answer.status, answer.location.as_str()),
        (StatusCode::SEE_OTHER, "/sign-in")
    );
    assert!(root.cookies.is_empty(), "{:?}", answer.set_cookies);
    assert_eq!(root.get(&router, "/").await.status, StatusCode::SEE_OTHER);
    let mut replayed = signed_in;
    let answer = replayed.get(&router, "/").await;
    assert_eq!(
        (answer.status, answer.location.as_str()),
        (StatusCode::SEE_OTHER, "/sign-in")
    );
    assert!(replayed.cookies.is_empty(), "the cookie is taken back");

    // Reached over HTTPS, admin only ever has the cookie sent encrypted.
    let answer = Browser::default()
        .sign_in(&admin.router_with(true), &api, ROOT_EMAIL, ROOT_PASSWORD, 0)
        .await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);
    assert!(
        answer.set_cookies[0].contains("Secure"),
        "{:?}",
        answer.set_cookies
    );

    admin.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn a_user_who_is_not_an_administrator_reads_no_page() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    let mut admin = Admin::start(api.addr, &dir.path().join("admin.db")).await;
    admin.api = api.serve_http().await;
    let router = admin.router();
    let alice = api.sign_up("alice@example.com").await;
    let mut browser = Browser::default();

    // The password and the code are hers, and the api signs her in: admin
    // does not.
    let answer = browser
        .sign_in(&router, &api, "alice@example.com", PASSWORD, 1)
        .await;
    assert_eq!(answer.status, StatusCode::FORBIDDEN, "{}", answer.body);
    assert!(
        answer.body.contains("data-sign-in=\"not-admin\""),
        "{}",
        answer.body
    );
    assert!(answer.set_cookies.is_empty(), "{:?}", answer.set_cookies);
    for path in ["/", "/nowhere"] {
        let answer = browser.get(&router, path).await;
        assert_eq!(answer.status, StatusCode::SEE_OTHER, "{path}");
        assert_eq!(answer.location, "/sign-in", "{path}");
    }

    // A wrong code is asked for again.
    let answer = browser
        .post(
            &router,
            "/sign-in",
            &[
                ("email", "alice@example.com"),
                ("password", PASSWORD),
                ("code", "000000"),
            ],
        )
        .await;
    assert!(answer.status.is_client_error(), "{}", answer.status);
    assert!(
        answer.body.contains("data-sign-in=\"refused\""),
        "{}",
        answer.body
    );
    assert!(answer.body.contains("name=\"code\""), "{}", answer.body);

    // Made an administrator, she reads the pages.
    api.set_role(&alice, "admin").await;
    let answer = browser
        .sign_in(&router, &api, "alice@example.com", PASSWORD, 2)
        .await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);
    let page = browser.get(&router, "/").await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(
        page.body.contains("data-account=\"alice@example.com\""),
        "{}",
        page.body
    );

    // One no more, the session she has reads nothing further, and is
    // taken from her browser.
    api.set_role(&alice, "user").await;
    let answer = browser.get(&router, "/").await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER);
    assert_eq!(answer.location, "/sign-in?refused=1");
    assert!(browser.cookies.is_empty(), "{:?}", answer.set_cookies);
    let page = browser.get(&router, &answer.location).await;
    assert_eq!(page.status, StatusCode::FORBIDDEN);
    assert!(
        page.body.contains("data-sign-in=\"not-admin\""),
        "{}",
        page.body
    );

    admin.stop().await;
    api.shutdown().await;
}

#[tokio::test]
async fn nobody_is_let_through_while_the_api_is_away() {
    let dir = dir();
    let mut api = Api::start(dir.path()).await;
    let mut admin = Admin::start(api.addr, &dir.path().join("admin.db")).await;
    admin.api = api.serve_http().await;
    let mut root = Browser::default();
    let answer = root
        .sign_in(&admin.router(), &api, ROOT_EMAIL, ROOT_PASSWORD, 0)
        .await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);

    // Nothing answers where admin asks who reads it.
    admin.api = NOWHERE.to_owned();
    let router = admin.router();
    let answer = root.get(&router, "/").await;
    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(!answer.body.contains("data-users"), "{}", answer.body);
    assert!(!root.cookies.is_empty(), "the session is kept for later");

    let answer = Browser::default()
        .post(
            &router,
            "/sign-in",
            &[("email", ROOT_EMAIL), ("password", ROOT_PASSWORD)],
        )
        .await;
    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(
        answer.body.contains("data-sign-in=\"failed\""),
        "{}",
        answer.body
    );

    // Signing out forgets the session all the same.
    let answer = root.post(&router, "/sign-out", &[]).await;
    assert_eq!(
        (answer.status, answer.location.as_str()),
        (StatusCode::SEE_OTHER, "/sign-in")
    );
    assert!(root.cookies.is_empty(), "{:?}", answer.set_cookies);

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
