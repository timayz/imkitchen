mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use imkitchen_identity::RegisterInput;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::{TestApp, json};

const UA: &str = "ua-phone";

async fn sign_in(app: &TestApp) -> anyhow::Result<String> {
    app.state
        .identity
        .register(RegisterInput {
            email: "chef@imkitchen.test".to_owned(),
            password: "correct-horse".to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
        })
        .await?;
    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::USER_AGENT, UA)
                .body(Body::from(
                    json!({ "email": "chef@imkitchen.test", "password": "correct-horse" })
                        .to_string(),
                ))?,
        )
        .await?;
    Ok(json(response).await["token"].as_str().unwrap().to_owned())
}

async fn call(
    app: &TestApp,
    token: &str,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> anyhow::Result<(StatusCode, Value)> {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::USER_AGENT, UA);
    let body = match body {
        Some(value) => {
            request = request.header(header::CONTENT_TYPE, "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };
    let response = app.router().oneshot(request.body(body)?).await?;
    let status = response.status();
    let value = if status == StatusCode::NO_CONTENT {
        Value::Null
    } else {
        json(response).await
    };
    Ok((status, value))
}

fn tour<'a>(body: &'a Value, id: &str) -> &'a Value {
    body["tours"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == id)
        .expect("tour in catalog")
}

#[tokio::test]
async fn a_new_user_has_every_tour_pending() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let token = sign_in(&app).await?;

    let (status, body) = call(&app, &token, "GET", "/api/v1/tours", None).await?;
    assert_eq!(status, StatusCode::OK);
    let tours = body["tours"].as_array().unwrap();
    assert_eq!(tours.len(), 5);
    assert!(
        tours
            .iter()
            .all(|t| t["status"] == "pending" && t["step"] == 0)
    );

    let kitchen = tour(&body, "kitchen");
    assert_eq!(kitchen["page"], "kitchen");
    assert_eq!(kitchen["version"], 1);
    assert_eq!(kitchen["steps"], json!(["welcome", "nav", "list", "cta"]));
    Ok(())
}

#[tokio::test]
async fn progress_round_trip() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let token = sign_in(&app).await?;

    let (status, _) = call(
        &app,
        &token,
        "POST",
        "/api/v1/tours/kitchen/advance",
        Some(json!({ "step": 2 })),
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, body) = call(&app, &token, "GET", "/api/v1/tours", None).await?;
    assert_eq!(tour(&body, "kitchen")["status"], "in_progress");
    assert_eq!(tour(&body, "kitchen")["step"], 2);
    assert_eq!(tour(&body, "recipes")["status"], "pending");

    // Advancing to the same step again writes nothing and still succeeds.
    let (status, _) = call(
        &app,
        &token,
        "POST",
        "/api/v1/tours/kitchen/advance",
        Some(json!({ "step": 2 })),
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = call(&app, &token, "POST", "/api/v1/tours/kitchen/complete", None).await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, body) = call(&app, &token, "GET", "/api/v1/tours", None).await?;
    assert_eq!(tour(&body, "kitchen")["status"], "completed");

    let (status, _) = call(
        &app,
        &token,
        "POST",
        "/api/v1/tours/recipes/skip",
        Some(json!({ "step": 1 })),
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, body) = call(&app, &token, "GET", "/api/v1/tours", None).await?;
    assert_eq!(tour(&body, "recipes")["status"], "skipped");
    assert_eq!(tour(&body, "recipes")["step"], 1);

    let (status, _) = call(&app, &token, "POST", "/api/v1/tours/reset", None).await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, body) = call(&app, &token, "GET", "/api/v1/tours", None).await?;
    assert!(
        body["tours"]
            .as_array()
            .unwrap()
            .iter()
            .all(|t| t["status"] == "pending" && t["step"] == 0)
    );
    Ok(())
}

#[tokio::test]
async fn bad_requests_are_rejected() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let token = sign_in(&app).await?;

    let (status, body) = call(
        &app,
        &token,
        "POST",
        "/api/v1/tours/nope/advance",
        Some(json!({ "step": 0 })),
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");

    let (status, body) = call(
        &app,
        &token,
        "POST",
        "/api/v1/tours/kitchen/advance",
        Some(json!({ "step": 99 })),
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "user");

    let (status, _) = call(&app, &token, "POST", "/api/v1/tours/nope/complete", None).await?;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let response = app
        .router()
        .oneshot(
            Request::get("/api/v1/tours")
                .header(header::USER_AGENT, UA)
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}
