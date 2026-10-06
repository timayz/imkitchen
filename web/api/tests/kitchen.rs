mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use imkitchen_core::recipe::UpdateInput;
use imkitchen_identity::RegisterInput;
use imkitchen_types::recipe::{
    Ingredient, IngredientCategory, IngredientUnit, Instruction, RecipeType,
};
use serde_json::{Value, json};
use tower::ServiceExt;

use common::{TestApp, json, wait_until};

const UA: &str = "imkitchen-android (Android; Pixel 8; test-install)";

struct Session {
    token: String,
    user_id: String,
}

async fn sign_in(app: &TestApp) -> anyhow::Result<Session> {
    let user_id = app
        .state
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

/// A three-step main course owned by `user_id`, written through the real
/// commands so every projection sees it.
async fn create_recipe(app: &TestApp, user_id: &str) -> anyhow::Result<String> {
    let id = app.state.core.recipe.create(user_id, None).await?;
    app.state
        .core
        .recipe
        .update(
            UpdateInput {
                id: id.clone(),
                recipe_type: RecipeType::MainCourse,
                name: "Shakshuka".to_owned(),
                origin: None,
                description: "Eggs poached in spiced tomato sauce.".to_owned(),
                household_size: 2,
                prep_time: 10,
                cook_time: 25,
                ingredients: vec![
                    Ingredient {
                        name: "Tomatoes".to_owned(),
                        quantity: 400,
                        unit: Some(IngredientUnit::G),
                        category: Some(IngredientCategory::FruitsAndVegetables),
                    },
                    Ingredient {
                        name: "Eggs".to_owned(),
                        quantity: 4,
                        unit: None,
                        category: Some(IngredientCategory::DairyAndEggs),
                    },
                ],
                instructions: vec![
                    Instruction {
                        description: "Simmer the tomatoes.".to_owned(),
                        time_next: 10,
                    },
                    Instruction {
                        description: "Crack in the eggs.".to_owned(),
                        time_next: 0,
                    },
                    Instruction {
                        description: "Cover and cook until set.".to_owned(),
                        time_next: 5,
                    },
                ],
                dietary_restrictions: vec![],
                accepts_accompaniment: true,
                advance_prep: "Chop the onions the night before.".to_owned(),
            },
            user_id,
        )
        .await?;
    Ok(id)
}

#[tokio::test]
async fn empty_kitchen_is_onboarding_recipe() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let session = sign_in(&app).await?;

    let (status, body) = call(&app, &session, "GET", "/api/v1/kitchen", None).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["kind"], "onboarding_recipe");

    Ok(())
}

#[tokio::test]
async fn kitchen_requires_auth() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let response = app
        .router()
        .oneshot(Request::get("/api/v1/kitchen").body(Body::empty())?)
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test]
async fn cooks_a_recipe_from_the_list_step_by_step() -> anyhow::Result<()> {
    let app = TestApp::with_recipes().await?;
    let session = sign_in(&app).await?;
    let recipe_id = create_recipe(&app, &session.user_id).await?;

    // The household size (preferences default) scales the 2-serving recipe.
    let household = app
        .state
        .identity
        .meal_preferences
        .load(&session.user_id)
        .await?
        .household_size;

    // `add_recipe` validates against the recipe projections, which are async:
    // retry until they have caught up.
    wait_until(|| async {
        app.state
            .core
            .shopping
            .add_recipe(&recipe_id, household, &session.user_id)
            .await
            .ok()
    })
    .await;

    // The list reads the aggregate, but entries come from the recipe projection.
    let body = wait_until(|| async {
        let (_, body) = call(&app, &session, "GET", "/api/v1/kitchen", None)
            .await
            .ok()?;
        (body["kind"] == "list" && body["entries"].as_array()?.len() == 1).then_some(body)
    })
    .await;
    assert_eq!(body["total_count"], 1);
    assert_eq!(body["completed_count"], 0);
    assert_eq!(body["entries"][0]["name"], "Shakshuka");
    assert_eq!(body["entries"][0]["recipe_type"], "MainCourse");
    assert_eq!(body["entries"][0]["status"]["status"], "idle");
    assert_eq!(body["entries"][0]["total_time"], 35);
    assert_eq!(body["focused"]["id"], recipe_id);
    // The dashboard previews the first step while idle.
    assert_eq!(body["steps"]["current"]["index"], 0);
    assert_eq!(body["steps"]["coming"].as_array().unwrap().len(), 2);
    assert_eq!(body["cook_external"], false);
    assert!(
        body["prep_ahead"].as_array().unwrap().is_empty(),
        "focused recipe is not in prep_ahead"
    );

    // Cooking screen: ingredients first, scaled to the household.
    let (status, body) = call(
        &app,
        &session,
        "GET",
        &format!("/api/v1/kitchen/recipes/{recipe_id}/cook"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["show_ingredients"], true);
    assert_eq!(body["status"]["status"], "idle");
    assert!(body["steps"]["current"].is_null());
    let aisles = body["ingredient_aisles"].as_array().unwrap();
    assert_eq!(aisles.len(), 2);
    let tomatoes = aisles
        .iter()
        .flat_map(|a| a["items"].as_array().unwrap())
        .find(|i| i["name"] == "Tomatoes")
        .unwrap();
    assert_eq!(tomatoes["quantity"], 400 * (household.max(2) as u64) / 2);
    assert!(tomatoes["quantity_label"].as_str().unwrap().ends_with('g'));

    // Step forward: first instruction with its timer.
    let step = |direction: &'static str| {
        let app = &app;
        let session = &session;
        let recipe_id = recipe_id.clone();
        async move {
            call(
                app,
                session,
                "POST",
                &format!("/api/v1/kitchen/recipes/{recipe_id}/step"),
                Some(json!({ "direction": direction })),
            )
            .await
        }
    };
    let (status, body) = step("next").await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], json!({ "status": "cooking", "step": 0 }));
    assert_eq!(body["show_ingredients"], false);
    assert_eq!(body["steps"]["current"]["time_next"], 10);
    assert_eq!(body["steps"]["coming"].as_array().unwrap().len(), 2);

    let (_, body) = step("next").await?;
    assert_eq!(body["status"]["step"], 1);
    let (_, body) = step("next").await?;
    assert_eq!(body["status"]["status"], "completed");
    assert_eq!(body["steps"]["current"]["index"], 2);
    assert!(body["steps"]["coming"].as_array().unwrap().is_empty());

    // Back from completed lands on the second-to-last step.
    let (_, body) = step("prev").await?;
    assert_eq!(body["status"], json!({ "status": "cooking", "step": 1 }));

    // Unknown direction is a validation error from serde.
    let (status, _) = step("sideways").await?;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // Remove it: the list is empty again, recipes still exist -> onboarding_menu.
    let (status, _) = call(
        &app,
        &session,
        "DELETE",
        &format!("/api/v1/kitchen/recipes/{recipe_id}"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = call(
        &app,
        &session,
        "GET",
        &format!("/api/v1/kitchen/recipes/{recipe_id}"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let body = wait_until(|| async {
        let (_, body) = call(&app, &session, "GET", "/api/v1/kitchen", None)
            .await
            .ok()?;
        (body["kind"] == "onboarding_menu").then_some(body)
    })
    .await;
    assert_eq!(body["main_count"], 1);
    assert_eq!(body["recipes"][0]["name"], "Shakshuka");

    Ok(())
}

#[tokio::test]
async fn generate_fills_the_list_from_the_pool() -> anyhow::Result<()> {
    let app = TestApp::with_recipes().await?;
    let session = sign_in(&app).await?;
    create_recipe(&app, &session.user_id).await?;

    wait_until(|| async {
        let (_, body) = call(&app, &session, "GET", "/api/v1/kitchen", None)
            .await
            .ok()?;
        (body["kind"] == "onboarding_menu").then_some(())
    })
    .await;

    let (status, body) = call(
        &app,
        &session,
        "POST",
        "/api/v1/kitchen/generate",
        Some(json!({ "count": 2 })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["kind"], "list");

    // The list itself is read from the aggregate, but its entries come from
    // the recipe projection, which may still be catching up.
    wait_until(|| async {
        let (_, body) = call(&app, &session, "GET", "/api/v1/kitchen", None)
            .await
            .ok()?;
        (body["total_count"].as_u64()? >= 1).then_some(())
    })
    .await;

    Ok(())
}
