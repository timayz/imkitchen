mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use imkitchen_core::recipe::UpdateInput;
use imkitchen_identity::{RegisterInput, types::user::Role};
use imkitchen_types::recipe::{
    Ingredient, IngredientCategory, IngredientUnit, Instruction, RecipeType,
};
use serde_json::{Value, json};
use tower::ServiceExt;

use common::{TestApp, json, wait_until};

struct Session {
    token: String,
    ua: String,
    user_id: String,
}

async fn sign_in(app: &TestApp, email: &str, ua: &str) -> anyhow::Result<Session> {
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
                .header(header::USER_AGENT, ua)
                .body(Body::from(
                    json!({ "email": email, "password": "correct-horse" }).to_string(),
                ))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let token = json(response).await["token"].as_str().unwrap().to_owned();
    Ok(Session {
        token,
        ua: ua.to_owned(),
        user_id,
    })
}

async fn call(
    app: &TestApp,
    s: &Session,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> anyhow::Result<(StatusCode, Value)> {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header(header::AUTHORIZATION, format!("Bearer {}", s.token))
        .header(header::USER_AGENT, &s.ua);
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

async fn create_recipe(app: &TestApp, user_id: &str, name: &str) -> anyhow::Result<String> {
    let id = app.state.core.recipe.create(user_id, None).await?;
    app.state
        .core
        .recipe
        .update(
            UpdateInput {
                id: id.clone(),
                recipe_type: RecipeType::Dessert,
                name: name.to_owned(),
                origin: None,
                description: "Sweet and simple.".to_owned(),
                household_size: 4,
                prep_time: 15,
                cook_time: 30,
                ingredients: vec![Ingredient {
                    name: "Sugar".to_owned(),
                    quantity: 100,
                    unit: Some(IngredientUnit::G),
                    category: Some(IngredientCategory::Grocery),
                }],
                instructions: vec![Instruction {
                    description: "Mix and bake.".to_owned(),
                    time_next: 30,
                }],
                dietary_restrictions: vec![],
                accepts_accompaniment: false,
                advance_prep: String::new(),
            },
            user_id,
        )
        .await?;
    wait_until(|| async {
        app.state
            .core
            .recipe
            .find_user(&id)
            .await
            .ok()
            .flatten()
            .filter(|r| r.name == name)
    })
    .await;
    Ok(id)
}

#[tokio::test]
async fn my_library_lists_own_recipes_and_detail_resolves_slug() -> anyhow::Result<()> {
    let app = TestApp::with_recipes().await?;
    let me = sign_in(&app, "chef@imkitchen.test", "ua-chef").await?;
    let id = create_recipe(&app, &me.user_id, "Lemon Tart").await?;

    let (status, body) = call(&app, &me, "GET", "/api/v1/recipes?mine=true", None).await?;
    assert_eq!(status, StatusCode::OK);
    let edges = body["page"]["edges"].as_array().unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0]["node"]["name"], "Lemon Tart");
    assert_eq!(edges[0]["node"]["recipe_type"], "Dessert");
    assert_eq!(edges[0]["node"]["total_time"], 45);
    assert!(edges[0]["cursor"].is_string());
    assert_eq!(body["page"]["page_info"]["has_next_page"], false);
    assert_eq!(body["has_shared"], false);

    // Community browse hides my own (unshared) recipe.
    let (_, body) = call(&app, &me, "GET", "/api/v1/recipes", None).await?;
    assert!(body["page"]["edges"].as_array().unwrap().is_empty());

    // Detail by slug and by id, with the viewer's relation to it.
    let slug = edges[0]["node"]["slug"].as_str().unwrap().to_owned();
    let (status, body) = call(&app, &me, "GET", &format!("/api/v1/recipes/{slug}"), None).await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["id"], id);
    assert_eq!(body["is_owner"], true);
    assert_eq!(body["saved"], false);
    assert_eq!(body["in_shopping"], false);
    assert_eq!(body["ingredients"][0]["quantity_label"], "100 g");
    assert_eq!(body["instructions"][0]["time_next"], 30);
    assert_eq!(body["owner_stat"]["total"], 1);
    let (status, _) = call(&app, &me, "GET", &format!("/api/v1/recipes/{id}"), None).await?;
    assert_eq!(status, StatusCode::OK);

    // Draft creation returns an id the editor can load.
    let (status, body) = call(&app, &me, "POST", "/api/v1/recipes", None).await?;
    assert_eq!(status, StatusCode::CREATED);
    assert!(body["id"].is_string());

    Ok(())
}

#[tokio::test]
async fn sharing_saving_and_shopping_flow() -> anyhow::Result<()> {
    let app = TestApp::with_recipes().await?;
    let chef = sign_in(&app, "chef@imkitchen.test", "ua-chef").await?;
    let guest = sign_in(&app, "guest@imkitchen.test", "ua-guest").await?;
    let id = create_recipe(&app, &chef.user_id, "Pavlova").await?;

    // Not shared yet: invisible to others, cannot be saved.
    let (status, _) = call(&app, &guest, "GET", &format!("/api/v1/recipes/{id}"), None).await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        &app,
        &guest,
        "POST",
        &format!("/api/v1/recipes/{id}/save"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Sharing is for chefs, and chefs need a public name.
    let (status, _) = call(
        &app,
        &chef,
        "POST",
        &format!("/api/v1/recipes/{id}/share"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::FORBIDDEN);
    app.state
        .identity
        .change_role(&chef.user_id, Role::Chef, &chef.user_id)
        .await?;
    let (status, body) = call(
        &app,
        &chef,
        "POST",
        &format!("/api/v1/recipes/{id}/share"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "username_required");
    app.state
        .identity
        .set_username(&chef.user_id, "pastrychef".to_owned())
        .await?;
    let (status, _) = call(
        &app,
        &chef,
        "POST",
        &format!("/api/v1/recipes/{id}/share"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    wait_until(|| async {
        app.state
            .core
            .recipe
            .find_user(&id)
            .await
            .ok()
            .flatten()
            .filter(|r| r.is_shared)
    })
    .await;

    // Now the guest sees it in the community, can save it and add it to the list.
    let (_, body) = call(&app, &guest, "GET", "/api/v1/recipes", None).await?;
    assert_eq!(body["page"]["edges"][0]["node"]["id"], id);
    let (status, _) = call(
        &app,
        &guest,
        "POST",
        &format!("/api/v1/recipes/{id}/save"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(
        &app,
        &guest,
        "POST",
        &format!("/api/v1/recipes/{id}/shopping"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, body) = call(&app, &guest, "GET", &format!("/api/v1/recipes/{id}"), None).await?;
    assert_eq!(body["is_owner"], false);
    assert_eq!(body["saved"], true);
    assert_eq!(body["in_shopping"], true);
    assert_eq!(body["owner_name"], "pastrychef");
    let (status, _) = call(
        &app,
        &guest,
        "DELETE",
        &format!("/api/v1/recipes/{id}/save"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, body) = call(&app, &guest, "GET", &format!("/api/v1/recipes/{id}"), None).await?;
    assert_eq!(body["saved"], false);

    // Similar recipes: none besides itself.
    let (status, body) = call(
        &app,
        &guest,
        "GET",
        &format!("/api/v1/recipes/{id}/similar"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert!(body["edges"].as_array().unwrap().is_empty());

    // The chef's public profile.
    let (status, body) = call(&app, &guest, "GET", "/api/v1/cooks/pastrychef", None).await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["username"], "pastrychef");
    assert_eq!(body["stat"]["shared"], 1);
    assert_eq!(body["recipes"]["edges"].as_array().unwrap().len(), 1);
    let (status, _) = call(&app, &guest, "GET", "/api/v1/cooks/nobody", None).await?;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Make it private again: the guest loses access.
    let (status, _) = call(
        &app,
        &chef,
        "POST",
        &format!("/api/v1/recipes/{id}/unshare"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);
    wait_until(|| async {
        app.state
            .core
            .recipe
            .find_user(&id)
            .await
            .ok()
            .flatten()
            .filter(|r| !r.is_shared)
    })
    .await;
    let (status, _) = call(&app, &guest, "GET", &format!("/api/v1/recipes/{id}"), None).await?;
    assert_eq!(status, StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
async fn delete_is_async_and_exists_reports_it() -> anyhow::Result<()> {
    let app = TestApp::with_recipes().await?;
    let me = sign_in(&app, "chef@imkitchen.test", "ua-chef").await?;
    let id = create_recipe(&app, &me.user_id, "Gone Soon").await?;

    let (_, body) = call(
        &app,
        &me,
        "GET",
        &format!("/api/v1/recipes/{id}/exists"),
        None,
    )
    .await?;
    assert_eq!(body["exists"], true);

    let (status, _) = call(&app, &me, "DELETE", &format!("/api/v1/recipes/{id}"), None).await?;
    assert_eq!(status, StatusCode::ACCEPTED);

    wait_until(|| async {
        let (_, body) = call(
            &app,
            &me,
            "GET",
            &format!("/api/v1/recipes/{id}/exists"),
            None,
        )
        .await
        .ok()?;
        (body["exists"] == false).then_some(())
    })
    .await;

    Ok(())
}
