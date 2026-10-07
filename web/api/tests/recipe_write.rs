mod common;

use std::io::Cursor;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use base64::{Engine, engine::general_purpose::STANDARD};
use image::{DynamicImage, ImageFormat, RgbImage};
use imkitchen_identity::RegisterInput;
use serde_json::{Value, json};
use tower::ServiceExt;

use common::{TestApp, json};

const UA: &str = "imkitchen-android (Android; Pixel 8; test-install)";

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

fn recipe_input(name: &str) -> Value {
    json!({
        "recipe_type": "MainCourse",
        "name": name,
        "description": "A proper description.",
        "household_size": 2,
        "prep_time": 10,
        "cook_time": 20,
        "ingredients": [
            { "name": "Rice", "quantity": 200, "unit": "G", "category": "Grocery" },
            { "name": "Egg", "quantity": 2, "unit": null, "category": "DairyAndEggs" }
        ],
        "instructions": [
            { "description": "Cook the rice.", "time_next": 12 },
            { "description": "Fry the eggs.", "time_next": 0 }
        ],
        "dietary_restrictions": ["Vegetarian"],
        "accepts_accompaniment": true,
        "advance_prep": ""
    })
}

fn png_bytes() -> Vec<u8> {
    let img = RgbImage::new(4, 4);
    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

#[tokio::test]
async fn create_edit_update_round_trip() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let token = sign_in(&app).await?;

    let (status, body) = call(&app, &token, "POST", "/api/v1/recipes", None).await?;
    assert_eq!(status, StatusCode::CREATED);
    let id = body["id"].as_str().unwrap().to_owned();

    // The draft is editable by its owner right away (aggregate read).
    let (status, body) = call(
        &app,
        &token,
        "GET",
        &format!("/api/v1/recipes/{id}/edit"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["ingredients"].as_array().unwrap().is_empty());

    let (status, _) = call(
        &app,
        &token,
        "PUT",
        &format!("/api/v1/recipes/{id}"),
        Some(recipe_input("Egg fried rice")),
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, body) = call(
        &app,
        &token,
        "GET",
        &format!("/api/v1/recipes/{id}/edit"),
        None,
    )
    .await?;
    assert_eq!(body["name"], "Egg fried rice");
    assert_eq!(body["ingredients"][0]["unit"], "G");
    assert_eq!(body["ingredients"][1]["unit"], Value::Null);
    assert_eq!(body["instructions"][0]["time_next"], 12);
    assert_eq!(body["dietary_restrictions"][0], "Vegetarian");

    // Validation errors come back as the envelope, not a 500.
    let mut bad = recipe_input("No");
    bad["household_size"] = json!(0);
    let (status, body) = call(
        &app,
        &token,
        "PUT",
        &format!("/api/v1/recipes/{id}"),
        Some(bad),
    )
    .await?;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "validation");

    // Someone else cannot edit it.
    app.state
        .identity
        .register(RegisterInput {
            email: "other@imkitchen.test".to_owned(),
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
                    json!({ "email": "other@imkitchen.test", "password": "correct-horse" })
                        .to_string(),
                ))?,
        )
        .await?;
    let other = json(response).await["token"].as_str().unwrap().to_owned();
    let (status, _) = call(
        &app,
        &other,
        "GET",
        &format!("/api/v1/recipes/{id}/edit"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::FORBIDDEN);

    Ok(())
}

#[tokio::test]
async fn import_batch_reports_errors_and_last_id() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let token = sign_in(&app).await?;

    let mut broken = recipe_input("x");
    broken["description"] = json!("");
    let (status, body) = call(
        &app,
        &token,
        "POST",
        "/api/v1/recipes/import",
        Some(json!([
            recipe_input("Imported one"),
            broken,
            recipe_input("Imported two")
        ])),
    )
    .await?;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    let last_id = body["last_id"].as_str().unwrap().to_owned();
    assert_eq!(body["errors"].as_array().unwrap().len(), 1);
    assert_eq!(body["errors"][0]["name"], "x");

    app.drain().await?;
    let (_, body) = call(
        &app,
        &token,
        "GET",
        &format!("/api/v1/recipes/{last_id}/exists"),
        None,
    )
    .await?;
    assert_eq!(body["exists"], true);
    let (_, body) = call(&app, &token, "GET", "/api/v1/recipes?mine=true", None).await?;
    assert_eq!(body["page"]["edges"].as_array().unwrap().len(), 2);

    Ok(())
}

#[tokio::test]
async fn thumbnail_accepts_json_and_multipart_and_rejects_other_types() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let token = sign_in(&app).await?;
    let (_, body) = call(&app, &token, "POST", "/api/v1/recipes", None).await?;
    let id = body["id"].as_str().unwrap().to_owned();
    let upload = imkitchen_web_api::upload_routes().with_state(app.state.clone());

    // JSON body with base64 bytes (what the Lynx app sends).
    let response = upload
        .clone()
        .oneshot(
            Request::post(format!("/api/v1/recipes/{id}/thumbnail"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::USER_AGENT, UA)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "content_type": "image/png", "data_base64": STANDARD.encode(png_bytes()) })
                        .to_string(),
                ))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    // Multipart, like the web form.
    let boundary = "xXx";
    let mut multipart = Vec::new();
    multipart.extend_from_slice(
        format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"t.png\"\r\nContent-Type: image/png\r\n\r\n")
            .as_bytes(),
    );
    multipart.extend_from_slice(&png_bytes());
    multipart.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    let response = upload
        .clone()
        .oneshot(
            Request::post(format!("/api/v1/recipes/{id}/thumbnail"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::USER_AGENT, UA)
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(multipart))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    // Wrong type is a user error.
    let response = upload
        .oneshot(
            Request::post(format!("/api/v1/recipes/{id}/thumbnail"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::USER_AGENT, UA)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "content_type": "image/gif", "data_base64": STANDARD.encode(b"GIF89a") })
                        .to_string(),
                ))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(json(response).await["error"]["code"], "user");

    Ok(())
}
