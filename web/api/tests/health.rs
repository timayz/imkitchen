mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use tower::ServiceExt;

use common::{TestApp, json};

#[tokio::test]
async fn health_is_json_and_never_cached() -> anyhow::Result<()> {
    let app = TestApp::new().await?;

    let response = app
        .router()
        .oneshot(Request::get("/api/v1/health").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    let body = json(response).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));

    Ok(())
}

#[tokio::test]
async fn unknown_api_path_returns_json_envelope() -> anyhow::Result<()> {
    let app = TestApp::new().await?;

    let response = app
        .router()
        .oneshot(Request::get("/api/v1/nope").body(Body::empty())?)
        .await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json(response).await;
    assert_eq!(body["error"]["code"], "not_found");
    assert!(body["error"]["message"].is_string());

    Ok(())
}
