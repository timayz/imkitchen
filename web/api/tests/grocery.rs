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

async fn sign_in(app: &TestApp) -> anyhow::Result<(String, String)> {
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
    let token = json(response).await["token"].as_str().unwrap().to_owned();
    Ok((token, user_id))
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

#[tokio::test]
async fn empty_list_has_no_aisles() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let (token, _) = sign_in(&app).await?;

    let (status, body) = call(&app, &token, "GET", "/api/v1/groceries", None).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["recipe_count"], 0);
    assert_eq!(body["total_items"], 0);
    assert!(body["aisles"].as_array().unwrap().is_empty());

    Ok(())
}

#[tokio::test]
async fn groceries_group_by_aisle_and_toggle() -> anyhow::Result<()> {
    let app = TestApp::with_recipes().await?;
    let (token, user_id) = sign_in(&app).await?;

    let id = app.state.core.recipe.create(&user_id, None).await?;
    app.state
        .core
        .recipe
        .update(
            UpdateInput {
                id: id.clone(),
                recipe_type: RecipeType::MainCourse,
                name: "Omelette".to_owned(),
                origin: None,
                description: "Eggs, butter, patience.".to_owned(),
                household_size: 1,
                prep_time: 5,
                cook_time: 5,
                ingredients: vec![
                    Ingredient {
                        name: "Eggs".to_owned(),
                        quantity: 3,
                        unit: None,
                        category: Some(IngredientCategory::DairyAndEggs),
                    },
                    Ingredient {
                        name: "Butter".to_owned(),
                        quantity: 20,
                        unit: Some(IngredientUnit::G),
                        category: Some(IngredientCategory::DairyAndEggs),
                    },
                    Ingredient {
                        name: "Chives".to_owned(),
                        quantity: 10,
                        unit: Some(IngredientUnit::G),
                        category: Some(IngredientCategory::FruitsAndVegetables),
                    },
                ],
                instructions: vec![Instruction {
                    description: "Whisk, pour, fold.".to_owned(),
                    time_next: 0,
                }],
                dietary_restrictions: vec![],
                accepts_accompaniment: false,
                advance_prep: String::new(),
            },
            &user_id,
        )
        .await?;
    let household = app
        .state
        .identity
        .meal_preferences
        .load(&user_id)
        .await?
        .household_size;
    // The recipe projections are async: retry until they have caught up.
    wait_until(|| async {
        app.state
            .core
            .shopping
            .add_recipe(&id, household, &user_id)
            .await
            .ok()
    })
    .await;

    // Ingredients are projected into the shopping aggregate asynchronously.
    let body = wait_until(|| async {
        let (_, body) = call(&app, &token, "GET", "/api/v1/groceries", None)
            .await
            .ok()?;
        (body["total_items"] == 3).then_some(body)
    })
    .await;
    assert_eq!(body["recipe_count"], 1);
    assert_eq!(body["checked_items"], 0);
    let aisles = body["aisles"].as_array().unwrap();
    assert_eq!(aisles.len(), 2);
    assert_eq!(aisles[0]["key"], "shopping_DairyAndEggs");
    assert_eq!(aisles[0]["total"], 2);
    assert_eq!(aisles[1]["key"], "shopping_FruitsAndVegetables");
    let butter = aisles[0]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "Butter")
        .unwrap();
    assert_eq!(butter["checked"], false);
    let key = butter["key"].as_str().unwrap().to_owned();
    assert!(butter["quantity_label"].as_str().unwrap().ends_with(" g"));

    // Check it off. The ingredient table behind the aggregate is projected
    // asynchronously: retry until the toggle is accepted.
    wait_until(|| async {
        let (status, _) = call(
            &app,
            &token,
            "POST",
            "/api/v1/groceries/toggle",
            Some(json!({ "key": key.clone() })),
        )
        .await
        .ok()?;
        (status == StatusCode::NO_CONTENT).then_some(())
    })
    .await;

    let (_, body) = call(&app, &token, "GET", "/api/v1/groceries", None).await?;
    assert_eq!(body["checked_items"], 1);
    assert_eq!(body["aisles"][0]["checked"], 1);
    assert_eq!(body["aisles"][0]["done"], false);
    let butter = body["aisles"][0]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "Butter")
        .unwrap();
    assert_eq!(butter["checked"], true);

    // And back.
    call(
        &app,
        &token,
        "POST",
        "/api/v1/groceries/toggle",
        Some(json!({ "key": key })),
    )
    .await?;
    let (_, body) = call(&app, &token, "GET", "/api/v1/groceries", None).await?;
    assert_eq!(body["checked_items"], 0);

    Ok(())
}
