mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use imkitchen_core::recipe::UpdateInput;
use imkitchen_identity::RegisterInput;
use imkitchen_types::recipe::{Ingredient, IngredientCategory, IngredientUnit, RecipeType};
use serde_json::{Value, json};
use tower::ServiceExt;

use common::{TestApp, json};

const UA: &str = "imkitchen-android (Android; Pixel 8; test-install)";

struct Session {
    token: String,
    user_id: String,
}

async fn sign_in(app: &TestApp, email: &str) -> anyhow::Result<Session> {
    let user_id = app
        .state
        .identity
        .register(RegisterInput {
            email: email.to_owned(),
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
                    json!({ "email": email, "password": "correct-horse" }).to_string(),
                ))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let token = json(response).await["token"].as_str().unwrap().to_owned();
    Ok(Session { token, user_id })
}

async fn call(
    app: &TestApp,
    session: &Session,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> anyhow::Result<(StatusCode, Value)> {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {}", session.token))
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

/// A one-ingredient recipe of `recipe_type` owned by `user_id`, written
/// through the real commands so every projection sees it.
async fn create_recipe(
    app: &TestApp,
    user_id: &str,
    name: &str,
    recipe_type: RecipeType,
) -> anyhow::Result<String> {
    let id = app.state.core.recipe.create(user_id, None).await?;
    app.state
        .core
        .recipe
        .update(
            UpdateInput {
                id: id.clone(),
                recipe_type,
                name: name.to_owned(),
                origin: None,
                description: format!("{name} description"),
                household_size: 4,
                prep_time: 10,
                cook_time: 20,
                ingredients: vec![Ingredient {
                    name: format!("{name} ingredient"),
                    quantity: 100,
                    unit: Some(IngredientUnit::G),
                    category: Some(IngredientCategory::Grocery),
                }],
                instructions: vec![],
                dietary_restrictions: vec![],
                accepts_accompaniment: true,
                advance_prep: "".to_owned(),
            },
            user_id,
        )
        .await?;
    Ok(id)
}

struct Pool {
    main: String,
    side: String,
    sauce: String,
    dessert: String,
}

async fn seed(app: &TestApp, user_id: &str) -> anyhow::Result<Pool> {
    let pool = Pool {
        main: create_recipe(app, user_id, "Roast chicken", RecipeType::MainCourse).await?,
        side: create_recipe(app, user_id, "Potatoes", RecipeType::Accompaniment).await?,
        sauce: create_recipe(app, user_id, "Gravy", RecipeType::Condiment).await?,
        dessert: create_recipe(app, user_id, "Apple pie", RecipeType::Dessert).await?,
    };
    app.drain().await?;
    Ok(pool)
}

#[tokio::test]
async fn dishes_require_auth() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let response = app
        .router()
        .oneshot(Request::get("/api/v1/dishes").body(Body::empty())?)
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test]
async fn composes_lists_updates_and_deletes_a_dish() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let session = sign_in(&app, "chef@imkitchen.test").await?;
    let pool = seed(&app, &session.user_id).await?;

    let (status, body) = call(&app, &session, "GET", "/api/v1/dishes", None).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["dishes"].as_array().unwrap().len(), 0);

    // Candidates: a page of the pool, by course then name.
    let (status, body) = call(&app, &session, "GET", "/api/v1/dishes/candidates", None).await?;
    assert_eq!(status, StatusCode::OK);
    let candidates = body["candidates"].as_array().unwrap();
    assert_eq!(candidates.len(), 4);
    assert_eq!(body["total"], 4);
    assert_eq!(candidates[0]["recipe_type"], "Accompaniment");
    assert_eq!(candidates[0]["name"], "Potatoes");
    assert_eq!(candidates[0]["accepts_accompaniment"], true);

    // One course, searched by name, paged.
    let (_, body) = call(
        &app,
        &session,
        "GET",
        "/api/v1/dishes/candidates?recipe_type=MainCourse&search=chick&limit=1",
        None,
    )
    .await?;
    assert_eq!(body["total"], 1);
    assert_eq!(body["candidates"][0]["name"], "Roast chicken");
    let (_, body) = call(
        &app,
        &session,
        "GET",
        "/api/v1/dishes/candidates?search=nothing-like-this",
        None,
    )
    .await?;
    assert_eq!(body["total"], 0);
    assert_eq!(body["candidates"].as_array().unwrap().len(), 0);

    // Compose, in any order: the dish comes back in meal order.
    let (status, body) = call(
        &app,
        &session,
        "POST",
        "/api/v1/dishes",
        Some(json!({
            "name": "Sunday roast",
            "recipe_ids": [pool.sauce, pool.side, pool.main],
        })),
    )
    .await?;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let id = body["id"].as_str().unwrap().to_owned();
    // The service waits for the read model; here nothing feeds it but `drain`.
    app.drain().await?;

    // Listed right away.
    let (status, body) = call(&app, &session, "GET", "/api/v1/dishes", None).await?;
    assert_eq!(status, StatusCode::OK);
    let dishes = body["dishes"].as_array().unwrap();
    assert_eq!(dishes.len(), 1);
    assert_eq!(dishes[0]["id"], id);
    assert_eq!(dishes[0]["name"], "Sunday roast");
    assert_eq!(dishes[0]["in_list"], false);
    assert_eq!(dishes[0]["total_time"], 30);
    let names: Vec<&str> = dishes[0]["recipes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["Roast chicken", "Potatoes", "Gravy"]);
    assert_eq!(dishes[0]["recipes"][0]["recipe_type"], "MainCourse");
    assert_eq!(dishes[0]["recipes"][0]["total_time"], 30);

    let (status, body) = call(&app, &session, "GET", &format!("/api/v1/dishes/{id}"), None).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Sunday roast");

    // Recompose: swap the sauce for the dessert and rename.
    let (status, _) = call(
        &app,
        &session,
        "PUT",
        &format!("/api/v1/dishes/{id}"),
        Some(json!({
            "name": "Sunday roast & pie",
            "recipe_ids": [pool.dessert, pool.main, pool.side],
        })),
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    app.drain().await?;

    let (_, body) = call(&app, &session, "GET", &format!("/api/v1/dishes/{id}"), None).await?;
    assert_eq!(body["name"], "Sunday roast & pie");
    let names: Vec<&str> = body["recipes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["Roast chicken", "Potatoes", "Apple pie"]);

    let (status, _) = call(
        &app,
        &session,
        "DELETE",
        &format!("/api/v1/dishes/{id}"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    app.drain().await?;
    let (status, _) = call(&app, &session, "GET", &format!("/api/v1/dishes/{id}"), None).await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, body) = call(&app, &session, "GET", "/api/v1/dishes", None).await?;
    assert_eq!(body["dishes"].as_array().unwrap().len(), 0);

    Ok(())
}

#[tokio::test]
async fn rejects_invalid_compositions_with_a_message() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let session = sign_in(&app, "chef@imkitchen.test").await?;
    let pool = seed(&app, &session.user_id).await?;

    let (status, body) = call(
        &app,
        &session,
        "POST",
        "/api/v1/dishes",
        Some(json!({ "name": "Sides only", "recipe_ids": [pool.side, pool.sauce] })),
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "user");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("main course")
    );

    let (status, body) = call(
        &app,
        &session,
        "POST",
        "/api/v1/dishes",
        Some(json!({ "name": "", "recipe_ids": [pool.main] })),
    )
    .await?;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    Ok(())
}

#[tokio::test]
async fn a_dish_is_private_to_its_owner() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let chef = sign_in(&app, "chef@imkitchen.test").await?;
    let guest = sign_in(&app, "guest@imkitchen.test").await?;
    let pool = seed(&app, &chef.user_id).await?;

    // The guest cannot use the chef's recipes.
    let (status, _) = call(
        &app,
        &guest,
        "POST",
        "/api/v1/dishes",
        Some(json!({ "name": "Borrowed", "recipe_ids": [pool.main] })),
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (_, body) = call(
        &app,
        &chef,
        "POST",
        "/api/v1/dishes",
        Some(json!({ "name": "Mine", "recipe_ids": [pool.main] })),
    )
    .await?;
    let id = body["id"].as_str().unwrap().to_owned();
    app.drain().await?;

    let (status, _) = call(&app, &guest, "GET", &format!("/api/v1/dishes/{id}"), None).await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        &app,
        &guest,
        "DELETE",
        &format!("/api/v1/dishes/{id}"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(
        &app,
        &guest,
        "POST",
        &format!("/api/v1/dishes/{id}/shopping"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, body) = call(&app, &guest, "GET", "/api/v1/dishes", None).await?;
    assert_eq!(body["dishes"].as_array().unwrap().len(), 0);

    Ok(())
}

#[tokio::test]
async fn adds_a_dish_to_the_kitchen_list() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let session = sign_in(&app, "chef@imkitchen.test").await?;
    let pool = seed(&app, &session.user_id).await?;

    let (_, body) = call(
        &app,
        &session,
        "POST",
        "/api/v1/dishes",
        Some(json!({ "name": "Sunday roast", "recipe_ids": [pool.main, pool.side, pool.sauce] })),
    )
    .await?;
    let id = body["id"].as_str().unwrap().to_owned();
    app.drain().await?;

    let (status, _) = call(
        &app,
        &session,
        "POST",
        &format!("/api/v1/dishes/{id}/shopping"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, body) = call(&app, &session, "GET", "/api/v1/kitchen", None).await?;
    assert_eq!(body["kind"], "list");
    let names: Vec<&str> = body["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["Roast chicken", "Potatoes", "Gravy"]);

    let (_, body) = call(&app, &session, "GET", &format!("/api/v1/dishes/{id}"), None).await?;
    assert_eq!(body["in_list"], true);

    // The groceries hold the three ingredients.
    let (_, body) = call(&app, &session, "GET", "/api/v1/groceries", None).await?;
    assert_eq!(body["total_items"], 3, "{body}");

    Ok(())
}
