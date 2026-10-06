mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use imkitchen_identity::RegisterInput;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::{TestApp, json};

async fn sign_in(app: &TestApp, ua: &str, register: bool) -> anyhow::Result<String> {
    if register {
        app.state
            .identity
            .register(RegisterInput {
                email: "chef@imkitchen.test".to_owned(),
                password: "correct-horse".to_owned(),
                lang: "en".to_owned(),
                timezone: "UTC".to_owned(),
            })
            .await?;
    }
    let response = app
        .router()
        .oneshot(
            Request::post("/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::USER_AGENT, ua)
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
    ua: &str,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> anyhow::Result<(StatusCode, Value)> {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::USER_AGENT, ua);
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

#[tokio::test]
async fn general_settings_round_trip() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let ua = "ua-phone";
    let token = sign_in(&app, ua, true).await?;

    let (status, body) = call(&app, &token, ua, "GET", "/api/v1/settings/general", None).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["email"], "chef@imkitchen.test");
    assert_eq!(body["household_size"], 4);
    assert_eq!(body["description"], "");

    let (status, _) = call(
        &app,
        &token,
        ua,
        "PUT",
        "/api/v1/settings/preferences",
        Some(json!({
            "household_size": 2,
            "dietary_restrictions": ["Vegan"],
            "recipe_types": ["Dessert"],
            "cuisine_variety_weight": 0.5
        })),
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(
        &app,
        &token,
        ua,
        "PUT",
        "/api/v1/settings/profile",
        Some(json!({ "description": "Home cook." })),
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, body) = call(&app, &token, ua, "GET", "/api/v1/settings/general", None).await?;
    assert_eq!(body["household_size"], 2);
    assert_eq!(body["dietary_restrictions"], json!(["Vegan"]));
    assert_eq!(body["recipe_types"], json!(["Dessert"]));
    assert_eq!(body["description"], "Home cook.");

    // Out-of-range weight is rejected with the envelope.
    let (status, body) = call(
        &app,
        &token,
        ua,
        "PUT",
        "/api/v1/settings/preferences",
        Some(json!({ "household_size": 2, "cuisine_variety_weight": 3.0 })),
    )
    .await?;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    // Username can be set once.
    let (status, _) = call(
        &app,
        &token,
        ua,
        "POST",
        "/api/v1/settings/username",
        Some(json!({ "username": "homecook" })),
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, me) = call(&app, &token, ua, "GET", "/api/v1/me", None).await?;
    assert_eq!(me["username"], "homecook");
    let (status, body) = call(
        &app,
        &token,
        ua,
        "POST",
        "/api/v1/settings/username",
        Some(json!({ "username": "again" })),
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "user");

    // Password reset is an email, always accepted.
    let (status, _) = call(
        &app,
        &token,
        ua,
        "POST",
        "/api/v1/settings/account/password-reset",
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::ACCEPTED);

    Ok(())
}

#[tokio::test]
async fn sessions_list_and_revoke_other_device() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let phone = sign_in(&app, "ua-phone", true).await?;
    let tablet = sign_in(&app, "ua-tablet", false).await?;

    let (status, body) = call(
        &app,
        &phone,
        "ua-phone",
        "GET",
        "/api/v1/settings/sessions",
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    let sessions = body.as_array().unwrap();
    assert_eq!(sessions.len(), 2);
    let current = sessions.iter().find(|s| s["current"] == true).unwrap();
    assert_eq!(current["user_agent"], "ua-phone");
    let other = sessions.iter().find(|s| s["current"] == false).unwrap();
    assert_eq!(other["user_agent"], "ua-tablet");

    let other_id = other["id"].as_str().unwrap();
    let (status, _) = call(
        &app,
        &phone,
        "ua-phone",
        "DELETE",
        &format!("/api/v1/settings/sessions/{other_id}"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = call(&app, &tablet, "ua-tablet", "GET", "/api/v1/me", None).await?;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (_, body) = call(
        &app,
        &phone,
        "ua-phone",
        "GET",
        "/api/v1/settings/sessions",
        None,
    )
    .await?;
    assert_eq!(body.as_array().unwrap().len(), 1);

    Ok(())
}
