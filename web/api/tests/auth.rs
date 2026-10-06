mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use imkitchen_identity::RegisterInput;
use serde_json::json;
use tower::ServiceExt;

use common::{TestApp, json};

const UA: &str = "imkitchen-android (Android; Pixel 8; test-install)";
const EMAIL: &str = "chef@imkitchen.test";
const PASSWORD: &str = "correct-horse";

async fn register(app: &TestApp) -> anyhow::Result<String> {
    Ok(app
        .state
        .identity
        .register(RegisterInput {
            email: EMAIL.to_owned(),
            password: PASSWORD.to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
        })
        .await?)
}

fn login_request(email: &str, password: &str, ua: &str) -> anyhow::Result<Request<Body>> {
    Ok(Request::post("/api/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::USER_AGENT, ua)
        .header(header::ACCEPT_LANGUAGE, "fr-FR,fr;q=0.9")
        .header("x-timezone", "Europe/Paris")
        .body(Body::from(
            json!({ "email": email, "password": password }).to_string(),
        ))?)
}

fn bearer(path: &str, token: &str, ua: &str) -> anyhow::Result<Request<Body>> {
    Ok(Request::get(path)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::USER_AGENT, ua)
        .body(Body::empty())?)
}

async fn login(app: &TestApp) -> anyhow::Result<(String, serde_json::Value)> {
    let response = app
        .router()
        .oneshot(login_request(EMAIL, PASSWORD, UA)?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let body = json(response).await;
    let token = body["token"].as_str().expect("token").to_owned();
    Ok((token, body))
}

#[tokio::test]
async fn login_returns_token_and_user() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let user_id = register(&app).await?;

    let (token, body) = login(&app).await?;
    assert!(!token.is_empty());
    assert!(body["expires_at"].as_str().unwrap().ends_with('Z'));
    assert_eq!(body["user"]["id"], user_id);
    assert_eq!(body["user"]["email"], EMAIL);
    assert_eq!(body["user"]["role"], "User");
    assert_eq!(body["user"]["is_chef"], false);
    // `[premium]` is absent in the test config: everyone is premium, nothing is for sale.
    assert_eq!(body["user"]["is_premium"], true);
    assert_eq!(body["user"]["premium_enabled"], false);
    assert_eq!(body["user"]["tz"], "Europe/Paris");

    Ok(())
}

#[tokio::test]
async fn login_with_bad_password_is_a_user_error() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    register(&app).await?;

    let response = app
        .router()
        .oneshot(login_request(EMAIL, "nope-nope", UA)?)
        .await?;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json(response).await;
    assert_eq!(body["error"]["code"], "user");

    Ok(())
}

#[tokio::test]
async fn me_requires_bearer_and_the_same_client_identity() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    register(&app).await?;
    let (token, _) = login(&app).await?;

    // No token: 401 JSON with a challenge, never a redirect.
    let response = app
        .router()
        .oneshot(
            Request::get("/api/v1/me")
                .header(header::USER_AGENT, UA)
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(response.headers()[header::WWW_AUTHENTICATE], "Bearer");
    assert_eq!(json(response).await["error"]["code"], "unauthorized");

    // Right token, different device identity: the session is bound to it.
    let response = app
        .router()
        .oneshot(bearer("/api/v1/me", &token, "someone-else/1")?)
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Happy path.
    let response = app
        .router()
        .oneshot(bearer("/api/v1/me", &token, UA)?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json(response).await["email"], EMAIL);

    Ok(())
}

#[tokio::test]
async fn logout_ends_only_this_session() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    register(&app).await?;
    let (token, _) = login(&app).await?;

    // A second device stays signed in.
    let other_ua = "imkitchen-android (Android; Pixel 6; other-install)";
    let response = app
        .router()
        .oneshot(login_request(EMAIL, PASSWORD, other_ua)?)
        .await?;
    let other_token = json(response).await["token"].as_str().unwrap().to_owned();

    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/logout")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::USER_AGENT, UA)
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = app
        .router()
        .oneshot(bearer("/api/v1/me", &token, UA)?)
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let response = app
        .router()
        .oneshot(bearer("/api/v1/me", &other_token, other_ua)?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn refresh_issues_a_token_for_the_same_session() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    register(&app).await?;
    let (token, _) = login(&app).await?;

    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/refresh")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::USER_AGENT, UA)
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let fresh = json(response).await["token"].as_str().unwrap().to_owned();

    let response = app
        .router()
        .oneshot(bearer("/api/v1/me", &fresh, UA)?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
async fn password_reset_is_accepted_for_unknown_addresses_too() -> anyhow::Result<()> {
    let app = TestApp::new().await?;

    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/password-reset")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "email": "nobody@imkitchen.test" }).to_string(),
                ))?,
        )
        .await?;

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    Ok(())
}

#[tokio::test]
async fn register_creates_the_account_and_signs_it_in() -> anyhow::Result<()> {
    let app = TestApp::new().await?;

    let register = |email: &str, password: &str| -> anyhow::Result<Request<Body>> {
        Ok(Request::post("/api/v1/auth/register")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::USER_AGENT, UA)
            .header(header::ACCEPT_LANGUAGE, "fr-FR,fr;q=0.9")
            .header("x-timezone", "Europe/Paris")
            .body(Body::from(
                json!({ "email": email, "password": password }).to_string(),
            ))?)
    };

    let response = app.router().oneshot(register(EMAIL, PASSWORD)?).await?;
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = json(response).await;
    let token = body["token"].as_str().expect("token").to_owned();
    assert_eq!(body["user"]["email"], EMAIL);
    assert_eq!(body["user"]["role"], "User");
    assert_eq!(body["user"]["tz"], "Europe/Paris");

    // The token works on this device right away.
    let response = app
        .router()
        .oneshot(bearer("/api/v1/me", &token, UA)?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    // Taken address: a user error. Short password: a validation error.
    let response = app.router().oneshot(register(EMAIL, PASSWORD)?).await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json(response).await["error"]["code"], "user");

    let response = app
        .router()
        .oneshot(register("other@imkitchen.test", "short")?)
        .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json(response).await["error"]["code"], "validation");

    Ok(())
}
