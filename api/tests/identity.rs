//! Registration, sign-in, sign-out and the accounts, through the api's
//! routes, on nodes that mail their codes into an outbox.

mod helpers;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Method, Request, StatusCode};
use evento::subscription::Subscription;
use helpers::{ROOT_EMAIL, ROOT_PASSWORD, TestCluster, TestNode, auth, eventually};
use http_body_util::BodyExt;
use imkitchen_api::locale::Locale;
use imkitchen_api::mail::{MailError, Mailer};
use imkitchen_api::server::{self, AppState};
use imkitchen_api::subscriptions;
use serde_json::{Value, json};
use tempfile::TempDir;
use timada_identity::CodeMail;
use tower::ServiceExt;

/// A password that passes the policy.
const PASSWORD: &str = "correct horse battery";

/// Where the nodes' mails land instead of a server.
#[derive(Default)]
struct Outbox {
    mails: Mutex<Vec<CodeMail>>,
    /// What the server answers instead of taking the next mails.
    refusal: Mutex<Option<MailError>>,
}

impl Outbox {
    fn refuse_with(&self, refusal: Option<MailError>) {
        *self.refusal.lock().unwrap() = refusal;
    }

    fn mails_to(&self, email: &str) -> Vec<CodeMail> {
        self.mails
            .lock()
            .unwrap()
            .iter()
            .filter(|mail| mail.email == email)
            .cloned()
            .collect()
    }

    /// The code of the `nth` mail sent to `email`, once it is there.
    async fn code(&self, email: &str, nth: usize) -> String {
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
struct Api {
    router: Router,
    state: AppState,
    subscriptions: Vec<Subscription>,
}

impl Api {
    async fn over(node: &TestNode, name: &str, outbox: Arc<Outbox>, lifetime: u64) -> Self {
        let state = AppState {
            auth: auth(lifetime),
            ..node.state()
        };
        let subscriptions = subscriptions::start(&node.executor, &state.auth, outbox, name)
            .await
            .unwrap();
        Self {
            router: server::router(state.clone()),
            state,
            subscriptions,
        }
    }

    async fn call(
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

    async fn post(&self, path: &str, token: Option<&str>, body: Value) -> (StatusCode, Value) {
        self.call(Method::POST, path, token, Some(body)).await
    }

    async fn get(&self, path: &str, token: Option<&str>) -> (StatusCode, Value) {
        self.call(Method::GET, path, token, None).await
    }

    async fn shutdown(self) {
        subscriptions::shutdown(self.subscriptions).await;
    }
}

/// The api over a node alone, and the outbox its mails land in.
async fn start() -> (Api, Arc<Outbox>, TestNode, TempDir) {
    let (node, dir) = TestNode::alone().await;
    let outbox = Arc::new(Outbox::default());
    let api = Api::over(&node, "api-0", outbox.clone(), 3600).await;
    (api, outbox, node, dir)
}

fn credentials(email: &str, password: &str, code: Option<&str>) -> Value {
    match code {
        Some(code) => json!({ "email": email, "password": password, "code": code }),
        None => json!({ "email": email, "password": password }),
    }
}

/// Registers `email` with [`PASSWORD`] and returns the session: the token
/// and the user's id.
async fn register(api: &Api, outbox: &Outbox, email: &str) -> (String, String) {
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
async fn sign_in_root(api: &Api) -> String {
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
fn refusal(code: &str) -> Value {
    let message = Locale::En.sentence(&format!("errors.{code}")).unwrap();
    json!({ "error": { "code": code, "message": message } })
}

/// The refusal of a body whose fields do not hold what the command asks.
fn invalid(fields: Value) -> Value {
    json!({ "error": {
        "code": "validation_failed",
        "message": "Some fields are not valid.",
        "fields": fields,
    } })
}

#[tokio::test]
async fn a_user_registers_signs_out_and_signs_in() {
    let (api, outbox, _node, _dir) = start().await;

    // Nothing is created before the address is proven.
    let (status, body) = api
        .post(
            "/v1/auth/register",
            None,
            credentials(" Alice@Example.com ", PASSWORD, None),
        )
        .await;
    assert_eq!(
        (status, &body),
        (StatusCode::ACCEPTED, &json!({ "status": "code_sent" }))
    );
    let code = outbox.code("alice@example.com", 0).await;

    let (status, body) = api
        .post(
            "/v1/auth/register",
            None,
            credentials("alice@example.com", PASSWORD, Some(&code)),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let token = body["token"].as_str().unwrap().to_owned();
    assert_eq!(body["user"]["email"], "alice@example.com");
    assert_eq!(body["user"]["role"], "user");
    assert!(body["expires_at"].as_u64().unwrap() > timada_core::unix_now());

    let (status, me) = api.get("/v1/account", Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{me}");
    assert_eq!(me["user"], body["user"]);

    let (status, _) = api
        .call(Method::POST, "/v1/auth/sign-out", Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, body) = api.get("/v1/account", Some(&token)).await;
    assert_eq!(
        (status, body),
        (StatusCode::UNAUTHORIZED, refusal("signed_out"))
    );

    // Signing in: the password, then the code it had mailed.
    let (status, body) = api
        .post(
            "/v1/auth/sign-in",
            None,
            credentials("alice@example.com", PASSWORD, None),
        )
        .await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    let code = outbox.code("alice@example.com", 1).await;
    let (status, body) = api
        .post(
            "/v1/auth/sign-in",
            None,
            credentials("alice@example.com", PASSWORD, Some(&code)),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let again = body["token"].as_str().unwrap();
    assert_ne!(again, token);
    let (status, _) = api.get("/v1/account", Some(again)).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = api.get("/v1/account", Some(&token)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "the old token stays out");

    api.shutdown().await;
}

#[tokio::test]
async fn what_a_person_can_correct_is_refused_with_its_code() {
    let (api, outbox, _node, _dir) = start().await;

    for (body, refusal) in [
        (
            credentials("not an address", "short", None),
            invalid(json!([
                {
                    "field": "email",
                    "code": "invalid_email",
                    "message": "This is not a valid e-mail address.",
                },
                {
                    "field": "password",
                    "code": "weak_password",
                    "message": "The password must be between 8 and 128 characters.",
                    "params": { "min": 8, "max": 128 },
                },
            ])),
        ),
        (
            credentials(ROOT_EMAIL, PASSWORD, None),
            refusal("already_registered"),
        ),
    ] {
        let (status, body) = api.post("/v1/auth/register", None, body).await;
        assert_eq!((status, body), (StatusCode::UNPROCESSABLE_ENTITY, refusal));
    }
    assert!(
        outbox.mails.lock().unwrap().is_empty(),
        "no code for nothing"
    );

    // A body that is not one.
    let (status, body) = api
        .post("/v1/auth/register", None, json!({ "email": 1 }))
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, refusal("invalid_request"))
    );
    let (status, body) = api
        .call(Method::POST, "/v1/auth/register", None, None)
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::BAD_REQUEST, refusal("invalid_request"))
    );

    let (status, _) = api
        .post(
            "/v1/auth/register",
            None,
            credentials("bob@example.com", PASSWORD, None),
        )
        .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let code = outbox.code("bob@example.com", 0).await;
    // Asked again within the minute: the code on its way is the one.
    let (status, body) = api
        .post(
            "/v1/auth/register",
            None,
            credentials("bob@example.com", PASSWORD, None),
        )
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::TOO_MANY_REQUESTS, refusal("code_recently_sent"))
    );
    let wrong = format!("{:06}", (code.parse::<u32>().unwrap() + 1) % 1_000_000);
    let (status, body) = api
        .post(
            "/v1/auth/register",
            None,
            credentials("bob@example.com", PASSWORD, Some(&wrong)),
        )
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, refusal("invalid_code"))
    );
    let (status, _) = api
        .post(
            "/v1/auth/register",
            None,
            credentials("bob@example.com", PASSWORD, Some(&code)),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);

    // Taken now; and neither an unknown address nor a wrong password says
    // which of the two it is.
    let (status, body) = api
        .post(
            "/v1/auth/register",
            None,
            credentials("bob@example.com", PASSWORD, None),
        )
        .await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            refusal("already_registered")
        )
    );
    for (email, password) in [
        ("bob@example.com", "not the password"),
        ("nobody@example.com", PASSWORD),
        (ROOT_EMAIL, "not root's password"),
    ] {
        let (status, body) = api
            .post("/v1/auth/sign-in", None, credentials(email, password, None))
            .await;
        assert_eq!(
            (status, body),
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                refusal("invalid_credentials")
            ),
            "{email}"
        );
    }

    api.shutdown().await;
}

#[tokio::test]
async fn unauthenticated_commands_are_refused() {
    let (api, outbox, _node, _dir) = start().await;
    let (token, user_id) = register(&api, &outbox, "alice@example.com").await;

    let routes = [
        (Method::POST, "/v1/auth/sign-out".to_owned(), None),
        (Method::GET, "/v1/account".to_owned(), None),
        (
            Method::PUT,
            "/v1/account/profile".to_owned(),
            Some(json!({ "name": "Mallory" })),
        ),
        (
            Method::PUT,
            "/v1/account/password".to_owned(),
            Some(json!({ "current_password": PASSWORD, "password": "another password" })),
        ),
        (
            Method::POST,
            "/v1/account/sign-out-everywhere".to_owned(),
            None,
        ),
        (Method::GET, format!("/v1/admin/users/{user_id}"), None),
        (Method::DELETE, format!("/v1/admin/users/{user_id}"), None),
        (
            Method::PUT,
            format!("/v1/admin/users/{user_id}/role"),
            Some(json!({ "role": "admin" })),
        ),
        (
            Method::POST,
            format!("/v1/admin/users/{user_id}/sign-out"),
            None,
        ),
    ];
    for (method, path, body) in routes {
        let (status, answer) = api.call(method.clone(), &path, None, body.clone()).await;
        assert_eq!(
            (status, answer),
            (StatusCode::UNAUTHORIZED, refusal("sign_in_required")),
            "{method} {path}"
        );
        let (status, answer) = api
            .call(method.clone(), &path, Some("not-a-token"), body)
            .await;
        assert_eq!(
            (status, answer),
            (StatusCode::UNAUTHORIZED, refusal("signed_out")),
            "{method} {path}"
        );
    }

    // Nothing of it happened.
    let (status, me) = api.get("/v1/account", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["user"]["name"], "alice@example.com");
    assert_eq!(me["user"]["role"], "user");

    api.shutdown().await;
}

#[tokio::test]
async fn admin_endpoints_refuse_who_is_not_an_administrator() {
    let (api, outbox, _node, _dir) = start().await;
    let (alice, alice_id) = register(&api, &outbox, "alice@example.com").await;
    let (bob, bob_id) = register(&api, &outbox, "bob@example.com").await;

    let admin_routes = |id: &str| {
        [
            (Method::GET, format!("/v1/admin/users/{id}"), None),
            (Method::DELETE, format!("/v1/admin/users/{id}"), None),
            (
                Method::PUT,
                format!("/v1/admin/users/{id}/role"),
                Some(json!({ "role": "admin" })),
            ),
            (Method::POST, format!("/v1/admin/users/{id}/sign-out"), None),
        ]
    };
    for (method, path, body) in admin_routes(&bob_id) {
        let (status, answer) = api.call(method.clone(), &path, Some(&alice), body).await;
        assert_eq!(
            (status, answer),
            (StatusCode::FORBIDDEN, refusal("not_allowed")),
            "{method} {path}"
        );
    }
    // Not on herself either.
    let (status, _) = api
        .call(
            Method::PUT,
            &format!("/v1/admin/users/{alice_id}/role"),
            Some(&alice),
            Some(json!({ "role": "admin" })),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = api.get("/v1/account", Some(&bob)).await;
    assert_eq!(status, StatusCode::OK, "bob is still there, signed in");

    // Root makes alice an administrator.
    let root = sign_in_root(&api).await;
    let (status, body) = api
        .get(&format!("/v1/admin/users/{alice_id}"), Some(&root))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["role"], "user");
    let (status, _) = api
        .call(
            Method::PUT,
            &format!("/v1/admin/users/{alice_id}/role"),
            Some(&root),
            Some(json!({ "role": "admin" })),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, me) = api.get("/v1/account", Some(&alice)).await;
    assert_eq!(me["user"]["role"], "admin");

    // An administrator reads and signs out the others, but only root makes
    // administrators, and nobody touches root.
    let (status, body) = api
        .get(&format!("/v1/admin/users/{bob_id}"), Some(&alice))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["email"], "bob@example.com");
    let (status, body) = api
        .call(
            Method::PUT,
            &format!("/v1/admin/users/{bob_id}/role"),
            Some(&alice),
            Some(json!({ "role": "admin" })),
        )
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::FORBIDDEN, refusal("not_allowed"))
    );
    let (status, _) = api
        .call(Method::DELETE, "/v1/admin/users/root", Some(&alice), None)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, body) = api.get("/v1/admin/users/nobody", Some(&alice)).await;
    assert_eq!(
        (status, body),
        (StatusCode::NOT_FOUND, refusal("not_registered"))
    );

    let (status, _) = api
        .call(
            Method::POST,
            &format!("/v1/admin/users/{bob_id}/sign-out"),
            Some(&alice),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = api.get("/v1/account", Some(&bob)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Deleted, bob is gone and his address is free again.
    let (status, _) = api
        .call(
            Method::DELETE,
            &format!("/v1/admin/users/{bob_id}"),
            Some(&alice),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = api
        .get(&format!("/v1/admin/users/{bob_id}"), Some(&alice))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, new_bob_id) = register(&api, &outbox, "bob@example.com").await;
    assert_ne!(new_bob_id, bob_id);

    // No longer an administrator, alice is refused again.
    let (status, _) = api
        .call(
            Method::PUT,
            &format!("/v1/admin/users/{alice_id}/role"),
            Some(&root),
            Some(json!({ "role": "user" })),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = api
        .get(&format!("/v1/admin/users/{new_bob_id}"), Some(&alice))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    api.shutdown().await;
}

#[tokio::test]
async fn a_user_manages_their_account() {
    let (api, outbox, _node, _dir) = start().await;
    let email = "alice@example.com";
    let (first, _) = register(&api, &outbox, email).await;

    // A second session, as from another device.
    let (status, _) = api
        .post("/v1/auth/sign-in", None, credentials(email, PASSWORD, None))
        .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let code = outbox.code(email, 1).await;
    let (_, body) = api
        .post(
            "/v1/auth/sign-in",
            None,
            credentials(email, PASSWORD, Some(&code)),
        )
        .await;
    let second = body["token"].as_str().unwrap().to_owned();

    let (status, _) = api
        .call(
            Method::PUT,
            "/v1/account/profile",
            Some(&first),
            Some(json!({ "name": " Alice " })),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, me) = api.get("/v1/account", Some(&second)).await;
    assert_eq!(me["user"]["name"], "Alice");
    assert_eq!(me["user"]["email"], email);
    let (status, body) = api
        .call(
            Method::PUT,
            "/v1/account/profile",
            Some(&first),
            Some(json!({ "name": "  " })),
        )
        .await;
    assert_eq!(
        (status, body),
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            invalid(json!([{
                "field": "name",
                "code": "length",
                "message": "This must be between 1 and 255 characters.",
                "params": { "min": 1, "max": 255 },
            }]))
        )
    );

    // Signing out everywhere spares the session that asks.
    let (status, _) = api
        .call(
            Method::POST,
            "/v1/account/sign-out-everywhere",
            Some(&first),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = api.get("/v1/account", Some(&first)).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = api.get("/v1/account", Some(&second)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // A new password takes the current one.
    let new_password = "another long password";
    let (status, body) = api
        .call(
            Method::PUT,
            "/v1/account/password",
            Some(&first),
            Some(json!({ "current_password": "not it", "password": new_password })),
        )
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, refusal("wrong_password"))
    );
    let (status, _) = api
        .call(
            Method::PUT,
            "/v1/account/password",
            Some(&first),
            Some(json!({ "current_password": PASSWORD, "password": new_password })),
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = api.get("/v1/account", Some(&first)).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = api
        .post("/v1/auth/sign-in", None, credentials(email, PASSWORD, None))
        .await;
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "the old one is out"
    );

    api.shutdown().await;
}

#[tokio::test]
async fn a_forgotten_password_is_reset_with_a_code() {
    let (api, outbox, _node, _dir) = start().await;
    let email = "alice@example.com";
    let (old, user_id) = register(&api, &outbox, email).await;

    // An address nobody registered is answered the same, and mailed nothing.
    let (status, body) = api
        .post(
            "/v1/auth/password-reset",
            None,
            json!({ "email": "nobody@example.com" }),
        )
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::ACCEPTED, json!({ "status": "code_sent" }))
    );

    let (status, _) = api
        .post("/v1/auth/password-reset", None, json!({ "email": email }))
        .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let code = outbox.code(email, 1).await;
    assert!(outbox.mails_to("nobody@example.com").is_empty());

    let new_password = "another long password";
    let (status, body) = api
        .post(
            "/v1/auth/password-reset",
            None,
            json!({ "email": email, "code": code }),
        )
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::UNPROCESSABLE_ENTITY, refusal("invalid_request"))
    );
    let (status, body) = api
        .post(
            "/v1/auth/password-reset",
            None,
            json!({ "email": email, "code": code, "password": new_password }),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["user"]["id"], user_id.as_str());
    let new = body["token"].as_str().unwrap();

    // Whoever held the old password is out; the session of the reset stays.
    let (status, _) = api.get("/v1/account", Some(&old)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = api.get("/v1/account", Some(new)).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = api
        .post(
            "/v1/auth/sign-in",
            None,
            credentials(email, new_password, None),
        )
        .await;
    assert_eq!(status, StatusCode::ACCEPTED);

    api.shutdown().await;
}

#[tokio::test]
async fn an_address_the_server_refuses_may_ask_again_at_once() {
    let (api, outbox, node, _dir) = start().await;
    let email = "alice@example.com";

    outbox.refuse_with(Some(MailError::Undeliverable("no such mailbox".to_owned())));
    let (status, _) = api
        .post(
            "/v1/auth/register",
            None,
            credentials(email, PASSWORD, None),
        )
        .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let identity = api.state.identity();
    eventually("the refusal to be recorded", || async {
        identity
            .load_email_challenge(email)
            .await
            .unwrap()
            .filter(|challenge| challenge.mail.is_undeliverable())
    })
    .await;

    // No minute to wait: the server took nothing.
    outbox.refuse_with(None);
    let (_token, _id) = register(&api, &outbox, email).await;
    assert_eq!(outbox.mails_to(email).len(), 1);

    api.shutdown().await;
    drop(node);
}

#[tokio::test]
async fn a_session_in_use_is_extended() {
    let (node, _dir) = TestNode::alone().await;
    let outbox = Arc::new(Outbox::default());
    // Six seconds of lifetime: extended once less than three are left.
    let api = Api::over(&node, "api-0", outbox.clone(), 6).await;
    let (token, _) = register(&api, &outbox, "alice@example.com").await;

    let (_, me) = api.get("/v1/account", Some(&token)).await;
    let expires_at = me["expires_at"].as_u64().unwrap();
    tokio::time::sleep(Duration::from_secs(4)).await;
    let (status, me) = api.get("/v1/account", Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{me}");
    assert!(me["expires_at"].as_u64().unwrap() > expires_at, "{me}");

    // Left alone for its whole lifetime, it signs nobody in.
    tokio::time::sleep(Duration::from_secs(7)).await;
    let (status, body) = api.get("/v1/account", Some(&token)).await;
    assert_eq!(
        (status, body),
        (StatusCode::UNAUTHORIZED, refusal("signed_out"))
    );

    api.shutdown().await;
}

#[tokio::test]
async fn an_address_tries_only_so_many_passwords() {
    let (api, _outbox, _node, _dir) = start().await;

    let mut last = StatusCode::OK;
    for _ in 0..11 {
        (last, _) = api
            .post(
                "/v1/auth/sign-in",
                None,
                credentials(ROOT_EMAIL, "not root's password", None),
            )
            .await;
    }
    assert_eq!(last, StatusCode::TOO_MANY_REQUESTS);
    // The right one is not looked at either.
    let (status, body) = api
        .post(
            "/v1/auth/sign-in",
            None,
            credentials(ROOT_EMAIL, ROOT_PASSWORD, None),
        )
        .await;
    assert_eq!(
        (status, body),
        (StatusCode::TOO_MANY_REQUESTS, refusal("too_many_sign_ins"))
    );

    api.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cluster_mails_a_code_once_and_every_node_knows_the_session() {
    let cluster = TestCluster::start(3).await;
    let outbox = Arc::new(Outbox::default());
    let mut apis = Vec::new();
    for (id, node) in cluster.nodes.iter().enumerate() {
        apis.push(Api::over(node, &format!("api-{id}"), outbox.clone(), 3600).await);
    }
    let email = "alice@example.com";

    // Asked of one node, finished on another, used on the third.
    let (status, _) = apis[0]
        .post(
            "/v1/auth/register",
            None,
            credentials(email, PASSWORD, None),
        )
        .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let code = outbox.code(email, 0).await;
    let (status, body) = apis[1]
        .post(
            "/v1/auth/register",
            None,
            credentials(email, PASSWORD, Some(&code)),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let token = body["token"].as_str().unwrap();
    let (status, me) = apis[2].get("/v1/account", Some(token)).await;
    assert_eq!(status, StatusCode::OK, "{me}");
    assert_eq!(me["user"]["email"], email);

    // Every node saw the code issued; one of them mailed it.
    let identity = apis[2].state.identity();
    eventually("the mail to be recorded as sent", || async {
        identity
            .load_email_challenge(email)
            .await
            .unwrap()
            .filter(|challenge| challenge.mail.is_settled())
    })
    .await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(outbox.mails_to(email).len(), 1);

    for api in apis {
        api.shutdown().await;
    }
    cluster.shutdown().await;
}
