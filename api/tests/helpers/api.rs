//! The api's routes over test nodes that mail their codes into an outbox,
//! and the people of the tests signing in through them.

use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Method, Request, StatusCode};
use evento::subscription::Subscription;
use http_body_util::BodyExt;
use imkitchen_api::locale::Locale;
use imkitchen_api::mail::{MailError, Mailer};
use imkitchen_api::server::{self, AppState};
use imkitchen_api::subscriptions;
use serde_json::{Value, json};
use tempfile::TempDir;
use timada_identity::CodeMail;
use tower::ServiceExt;

use super::{ROOT_EMAIL, ROOT_PASSWORD, TestNode, auth, eventually};

/// A password that passes the policy.
pub const PASSWORD: &str = "correct horse battery";

/// Where the nodes' mails land instead of a server.
#[derive(Default)]
pub struct Outbox {
    pub mails: Mutex<Vec<CodeMail>>,
    /// What the server answers instead of taking the next mails.
    pub refusal: Mutex<Option<MailError>>,
}

impl Outbox {
    pub fn refuse_with(&self, refusal: Option<MailError>) {
        *self.refusal.lock().unwrap() = refusal;
    }

    pub fn mails_to(&self, email: &str) -> Vec<CodeMail> {
        self.mails
            .lock()
            .unwrap()
            .iter()
            .filter(|mail| mail.email == email)
            .cloned()
            .collect()
    }

    /// The code of the `nth` mail sent to `email`, once it is there.
    pub async fn code(&self, email: &str, nth: usize) -> String {
        eventually(&format!("mail {nth} to {email}"), || async {
            self.mails_to(email).get(nth).map(|mail| mail.code.clone())
        })
        .await
    }
}

#[async_trait::async_trait]
impl Mailer for Outbox {
    async fn send_code(&self, mail: &CodeMail) -> Result<(), MailError> {
        if let Some(refusal) = self.refusal.lock().unwrap().clone() {
            return Err(refusal);
        }
        self.mails.lock().unwrap().push(mail.clone());
        Ok(())
    }
}

/// The api's routes over a node, its subscriptions running.
pub struct Api {
    pub router: Router,
    pub state: AppState,
    pub subscriptions: Vec<Subscription>,
}

impl Api {
    pub async fn over(node: &TestNode, name: &str, outbox: Arc<Outbox>, lifetime: u64) -> Self {
        let state = AppState {
            auth: auth(lifetime),
            ..node.state()
        };
        let subscriptions =
            subscriptions::start(&node.executor, &state.db, &state.auth, outbox, name)
                .await
                .unwrap();
        Self {
            router: server::router(state.clone()),
            state,
            subscriptions,
        }
    }

    pub async fn call(
        &self,
        method: Method,
        path: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder().method(method).uri(path);
        if let Some(token) = token {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        let request = match body {
            Some(body) => request
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string())),
            None => request.body(Body::empty()),
        }
        .unwrap();
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        };
        (status, body)
    }

    pub async fn post(&self, path: &str, token: Option<&str>, body: Value) -> (StatusCode, Value) {
        self.call(Method::POST, path, token, Some(body)).await
    }

    pub async fn get(&self, path: &str, token: Option<&str>) -> (StatusCode, Value) {
        self.call(Method::GET, path, token, None).await
    }

    pub async fn shutdown(self) {
        subscriptions::shutdown(self.subscriptions).await;
    }
}

/// The api over a node alone, and the outbox its mails land in.
pub async fn start() -> (Api, Arc<Outbox>, TestNode, TempDir) {
    let (node, dir) = TestNode::alone().await;
    let outbox = Arc::new(Outbox::default());
    let api = Api::over(&node, "api-0", outbox.clone(), 3600).await;
    (api, outbox, node, dir)
}

pub fn credentials(email: &str, password: &str, code: Option<&str>) -> Value {
    match code {
        Some(code) => json!({ "email": email, "password": password, "code": code }),
        None => json!({ "email": email, "password": password }),
    }
}

/// Registers `email` with [`PASSWORD`] and returns the session: the token
/// and the user's id.
pub async fn register(api: &Api, outbox: &Outbox, email: &str) -> (String, String) {
    let (status, body) = api
        .post(
            "/v1/auth/register",
            None,
            credentials(email, PASSWORD, None),
        )
        .await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    let code = outbox.code(email, outbox.mails_to(email).len()).await;
    let (status, body) = api
        .post(
            "/v1/auth/register",
            None,
            credentials(email, PASSWORD, Some(&code)),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    (
        body["token"].as_str().unwrap().to_owned(),
        body["user"]["id"].as_str().unwrap().to_owned(),
    )
}

/// Signs root in, which needs no code, and returns its token.
pub async fn sign_in_root(api: &Api) -> String {
    let (status, body) = api
        .post(
            "/v1/auth/sign-in",
            None,
            credentials(ROOT_EMAIL, ROOT_PASSWORD, None),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["user"]["role"], "admin");
    body["token"].as_str().unwrap().to_owned()
}

/// What a refusal answers to a request that names no language.
pub fn refusal(code: &str) -> Value {
    let message = Locale::En.sentence(&format!("errors.{code}")).unwrap();
    json!({ "error": { "code": code, "message": message } })
}

/// The refusal of a body whose fields do not hold what the command asks.
pub fn invalid(fields: Value) -> Value {
    json!({ "error": {
        "code": "validation_failed",
        "message": "Some fields are not valid.",
        "fields": fields,
    } })
}
