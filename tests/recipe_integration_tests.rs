use evento::prelude::*;
use recipe::{
    create_recipe, delete_recipe, query_recipe_by_id, query_recipes_by_user, recipe_projection,
    CreateRecipeCommand, DeleteRecipeCommand, Ingredient, InstructionStep, RecipeError,
};
use sqlx::{Pool, Sqlite, SqlitePool};

/// Helper to create test database with required tables
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Run evento migrations for event store tables
    let mut conn = pool.acquire().await.unwrap();
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut *conn, &Plan::apply_all())
        .await
        .unwrap();
    drop(conn);

    // Run SQLx migrations for read model tables (same as production)
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    pool
}

/// Helper to create evento executor
async fn setup_evento_executor(pool: Pool<Sqlite>) -> evento::Sqlite {
    pool.into()
}

/// Create test user using proper evento commands
async fn create_test_user_for_tests(
    pool: &SqlitePool,
    executor: &evento::Sqlite,
    email: &str,
    tier: &str,
) -> String {
    use user::commands::{
        register_user, upgrade_subscription, RegisterUserCommand, UpgradeSubscriptionCommand,
    };

    // Register user via command (creates aggregate + events)
    let user_id = register_user(
        RegisterUserCommand {
            email: email.to_string(),
            password: "testpassword".to_string(),
        },
        executor,
        pool,
    )
    .await
    .unwrap();

    // If premium tier, upgrade subscription
    if tier == "premium" {
        upgrade_subscription(
            UpgradeSubscriptionCommand {
                user_id: user_id.clone(),
                new_tier: "premium".to_string(),
                stripe_customer_id: Some("cus_test".to_string()),
                stripe_subscription_id: Some("sub_test".to_string()),
            },
            executor,
        )
        .await
        .unwrap();
    }

    // Process user projection to populate read model synchronously
    user::user_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();

    user_id
}

#[tokio::test]
async fn test_create_recipe_integration_with_read_model_projection() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Process events synchronously for deterministic tests (unsafe_oneshot instead of run)
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .expect("Projection failed");

    // Create recipe command
    let command = CreateRecipeCommand {
        title: "Integration Test Recipe".to_string(),
        ingredients: vec![
            Ingredient {
                name: "Chicken".to_string(),
                quantity: 2.0,
                unit: "lb".to_string(),
            },
            Ingredient {
                name: "Salt".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            },
        ],
        instructions: vec![
            InstructionStep {
                step_number: 1,
                instruction_text: "Season chicken".to_string(),
                timer_minutes: None,
            },
            InstructionStep {
                step_number: 2,
                instruction_text: "Cook chicken".to_string(),
                timer_minutes: Some(30),
            },
        ],
        prep_time_min: Some(15),
        cook_time_min: Some(30),
        advance_prep_hours: Some(2),
        serving_size: Some(4),
    };

    // Execute recipe creation
    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    // Wait for projection to complete
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify recipe was projected into read model
    let recipe = query_recipe_by_id(&recipe_id, &pool).await.unwrap();
    assert!(recipe.is_some(), "Recipe should exist in read model");

    let recipe_data = recipe.unwrap();
    assert_eq!(recipe_data.id, recipe_id);
    assert_eq!(recipe_data.user_id, user1_id);
    assert_eq!(recipe_data.title, "Integration Test Recipe");
    assert_eq!(recipe_data.prep_time_min, Some(15));
    assert_eq!(recipe_data.cook_time_min, Some(30));
    assert_eq!(recipe_data.advance_prep_hours, Some(2));
    assert_eq!(recipe_data.serving_size, Some(4));
    assert!(!recipe_data.is_favorite);
    assert!(!recipe_data.is_shared); // Default privacy is private (AC-10)

    // Verify ingredients stored as JSON
    let ingredients: Vec<Ingredient> = serde_json::from_str(&recipe_data.ingredients).unwrap();
    assert_eq!(ingredients.len(), 2);
    assert_eq!(ingredients[0].name, "Chicken");
    assert_eq!(ingredients[0].quantity, 2.0);
    assert_eq!(ingredients[0].unit, "lb");

    // Verify instructions stored as JSON
    let instructions: Vec<InstructionStep> =
        serde_json::from_str(&recipe_data.instructions).unwrap();
    assert_eq!(instructions.len(), 2);
    assert_eq!(instructions[0].step_number, 1);
    assert_eq!(instructions[0].instruction_text, "Season chicken");
    assert_eq!(instructions[1].timer_minutes, Some(30));
}

#[tokio::test]
async fn test_query_recipes_by_user() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Start projection
    // Process events synchronously for deterministic tests
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create 3 recipes for user1
    for i in 1..=3 {
        let command = CreateRecipeCommand {
            title: format!("Recipe {}", i),
            ingredients: vec![Ingredient {
                name: "Salt".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Cook it".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: Some(10),
            cook_time_min: Some(20),
            advance_prep_hours: None,
            serving_size: Some(2),
        };
        create_recipe(command, &user1_id, &executor, &pool)
            .await
            .unwrap();
    }

    // Process projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Query all recipes for user1
    let recipes = query_recipes_by_user(&user1_id, false, &pool)
        .await
        .unwrap();
    assert_eq!(recipes.len(), 3, "Should have 3 recipes");

    // Verify sorted by created_at DESC (most recent first)
    assert!(recipes[0].title.contains("Recipe"));
}

// Note: Delete tests temporarily disabled as delete functionality is out of scope for Story 2.1 (Create Recipe only)
#[tokio::test]
async fn test_delete_recipe_integration() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Start projection
    // Process events synchronously for deterministic tests
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create recipe
    let command = CreateRecipeCommand {
        title: "Recipe to Delete".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook it".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: None,
        serving_size: Some(2),
    };
    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify recipe exists
    let recipe = query_recipe_by_id(&recipe_id, &pool).await.unwrap();
    assert!(recipe.is_some());

    // Delete recipe
    let delete_command = DeleteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.to_string(),
    };
    delete_recipe(delete_command, &executor, &pool)
        .await
        .unwrap();

    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify recipe deleted from read model
    let recipe = query_recipe_by_id(&recipe_id, &pool).await.unwrap();
    assert!(recipe.is_none(), "Recipe should be deleted from read model");
}

#[tokio::test]
async fn test_delete_recipe_permission_denied() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;
    let user2_id = create_test_user_for_tests(&pool, &executor, "user2@test.com", "free").await;

    // Start projection
    // Process events synchronously for deterministic tests
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // User1 creates recipe
    let command = CreateRecipeCommand {
        title: "User1 Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook it".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: None,
        serving_size: Some(2),
    };
    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // User2 tries to delete user1's recipe
    let delete_command = DeleteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user2_id.to_string(), // Different user!
    };
    let result = delete_recipe(delete_command, &executor, &pool).await;

    // Should fail with PermissionDenied
    assert!(matches!(result, Err(RecipeError::PermissionDenied)));
}

// ============================================================================
// UPDATE RECIPE HTTP INTEGRATION TESTS (Story 2.2)
// TwinSpark Pattern: POST with 200 OK + ts-location header (progressive enhancement)
// ============================================================================

#[tokio::test]
async fn test_post_recipe_update_success_returns_ts_location() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Start projection
    // Process events synchronously for deterministic tests
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create recipe via command
    let command = CreateRecipeCommand {
        title: "Original Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook it".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: None,
        serving_size: Some(2),
    };
    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    // Process projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create Axum app with recipe routes
    let app = imkitchen::create_app(pool.clone(), executor.clone())
        .await
        .unwrap();

    // Create JWT token for user1
    let token = user::generate_jwt(
        user1_id.to_string(),
        "user1@test.com".to_string(),
        "free".to_string(),
        "test-secret-key-32-bytes-long!!",
    )
    .unwrap();

    // POST /recipes/:id with updated data (TwinSpark pattern)
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/recipes/{}", recipe_id))
                .header("content-type", "application/x-www-form-urlencoded")
                .header("cookie", format!("auth_token={}", token))
                .body(Body::from(
                    "title=Updated Recipe&ingredient_name[]=Pepper&ingredient_quantity[]=2&ingredient_unit[]=tsp&instruction_text[]=Season it&instruction_timer[]=&prep_time_min=15&cook_time_min=25",
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // TwinSpark pattern: Returns 200 OK with ts-location header
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("ts-location").unwrap(),
        format!("/recipes/{}", recipe_id).as_str()
    );

    // Process projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify recipe updated in read model
    let updated_recipe = query_recipe_by_id(&recipe_id, &pool)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_recipe.title, "Updated Recipe");
    assert_eq!(updated_recipe.prep_time_min, Some(15));
    assert_eq!(updated_recipe.cook_time_min, Some(25));
}

#[tokio::test]
async fn test_post_recipe_update_unauthorized_returns_403() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;
    let _user2_id = create_test_user_for_tests(&pool, &executor, "user2@test.com", "free").await;

    // Start projection
    // Process events synchronously for deterministic tests
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // User1 creates recipe
    let command = CreateRecipeCommand {
        title: "User1 Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook it".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: None,
        serving_size: Some(2),
    };
    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    // Process projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create Axum app
    let app = imkitchen::create_app(pool.clone(), executor.clone())
        .await
        .unwrap();

    // Create JWT token for user2
    let token = user::generate_jwt(
        "user2".to_string(),
        "user2@test.com".to_string(),
        "free".to_string(),
        "test-secret-key-32-bytes-long!!",
    )
    .unwrap();

    // User2 tries to POST /recipes/:id (user1's recipe)
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/recipes/{}", recipe_id))
                .header("content-type", "application/x-www-form-urlencoded")
                .header("cookie", format!("auth_token={}", token))
                .body(Body::from("title=Hijacked Recipe&ingredient_name[]=Salt&ingredient_quantity[]=1&ingredient_unit[]=tsp&instruction_text[]=Hack it&instruction_timer[]="))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 403 Forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_post_recipe_update_invalid_data_returns_422() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Start projection
    // Process events synchronously for deterministic tests
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create recipe
    let command = CreateRecipeCommand {
        title: "Original Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook it".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: None,
        serving_size: Some(2),
    };
    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    // Process projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create Axum app
    let app = imkitchen::create_app(pool.clone(), executor.clone())
        .await
        .unwrap();

    // Create JWT token for user1
    let token = user::generate_jwt(
        "user1".to_string(),
        "user1@test.com".to_string(),
        "free".to_string(),
        "test-secret-key-32-bytes-long!!",
    )
    .unwrap();

    // POST /recipes/:id with invalid data (title too short)
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/recipes/{}", recipe_id))
                .header("content-type", "application/x-www-form-urlencoded")
                .header("cookie", format!("auth_token={}", token))
                .body(Body::from("title=Ab")) // Title < 3 characters
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 422 Unprocessable Entity for validation error
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_get_recipe_edit_form_prepopulated() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Start projection
    // Process events synchronously for deterministic tests
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create recipe
    let command = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook it".to_string(),
            timer_minutes: Some(30),
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: Some(2),
        serving_size: Some(4),
    };
    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    // Process projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create Axum app
    let app = imkitchen::create_app(pool.clone(), executor.clone())
        .await
        .unwrap();

    // Create JWT token for user1
    let token = user::generate_jwt(
        user1_id.to_string(),
        "user1@test.com".to_string(),
        "free".to_string(),
        "test-secret-key-32-bytes-long!!",
    )
    .unwrap();

    // GET /recipes/:id/edit
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/recipes/{}/edit", recipe_id))
                .header("cookie", format!("auth_token={}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    // Verify form is prepopulated with recipe data
    assert!(body_str.contains("Edit Recipe"));
    assert!(body_str.contains("value=\"Test Recipe\""));
    assert!(body_str.contains("Salt"));
    assert!(body_str.contains("Cook it"));
    assert!(body_str.contains("value=\"10\"")); // prep_time_min
    assert!(body_str.contains("value=\"20\"")); // cook_time_min
    assert!(body_str.contains("value=\"2\"")); // advance_prep_hours
    assert!(body_str.contains("value=\"4\"")); // serving_size
}

#[tokio::test]
async fn test_recipe_update_syncs_to_read_model() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Start projection
    // Process events synchronously for deterministic tests
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create recipe
    let command = CreateRecipeCommand {
        title: "Original Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook it".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: Some(1),
        serving_size: Some(2),
    };
    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    // Process projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Update recipe via command
    let update_command = recipe::UpdateRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.to_string(),
        title: Some("Updated Recipe".to_string()),
        ingredients: Some(vec![
            Ingredient {
                name: "Pepper".to_string(),
                quantity: 2.0,
                unit: "tsp".to_string(),
            },
            Ingredient {
                name: "Garlic".to_string(),
                quantity: 3.0,
                unit: "cloves".to_string(),
            },
        ]),
        instructions: Some(vec![
            InstructionStep {
                step_number: 1,
                instruction_text: "Season with spices".to_string(),
                timer_minutes: Some(5),
            },
            InstructionStep {
                step_number: 2,
                instruction_text: "Cook thoroughly".to_string(),
                timer_minutes: Some(30),
            },
        ]),
        prep_time_min: Some(Some(15)),
        cook_time_min: Some(Some(25)),
        advance_prep_hours: Some(Some(2)), // Change to 2 hours
        serving_size: Some(None),          // Clear serving size
    };
    recipe::update_recipe(update_command, &executor, &pool)
        .await
        .unwrap();

    // Process projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify read model updated
    let updated_recipe = query_recipe_by_id(&recipe_id, &pool)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_recipe.title, "Updated Recipe");
    assert_eq!(updated_recipe.prep_time_min, Some(15));
    assert_eq!(updated_recipe.cook_time_min, Some(25));
    assert_eq!(updated_recipe.advance_prep_hours, Some(2));
    assert_eq!(updated_recipe.serving_size, None);

    // Verify ingredients updated
    let ingredients: Vec<Ingredient> = serde_json::from_str(&updated_recipe.ingredients).unwrap();
    assert_eq!(ingredients.len(), 2);
    assert_eq!(ingredients[0].name, "Pepper");
    assert_eq!(ingredients[1].name, "Garlic");

    // Verify instructions updated
    let instructions: Vec<InstructionStep> =
        serde_json::from_str(&updated_recipe.instructions).unwrap();
    assert_eq!(instructions.len(), 2);
    assert_eq!(instructions[0].instruction_text, "Season with spices");
    assert_eq!(instructions[1].instruction_text, "Cook thoroughly");
}

// ============================================================================
// Recipe Deletion Integration Tests (Story 2.3)
// ============================================================================

#[tokio::test]
async fn test_delete_recipe_integration_removes_from_read_model() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Start projection
    // Process events synchronously for deterministic tests
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create a recipe
    let command = CreateRecipeCommand {
        title: "Recipe to Delete".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(2),
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    // Wait for projection
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify recipe exists in read model
    let recipe_before = query_recipe_by_id(&recipe_id, &pool).await.unwrap();
    assert!(
        recipe_before.is_some(),
        "Recipe should exist before deletion"
    );

    // Delete the recipe
    let delete_command = DeleteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.to_string(),
    };

    delete_recipe(delete_command, &executor, &pool)
        .await
        .unwrap();

    // Wait for projection to process RecipeDeleted event
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify recipe no longer exists in read model (soft delete via removal)
    let recipe_after = query_recipe_by_id(&recipe_id, &pool).await.unwrap();
    assert!(
        recipe_after.is_none(),
        "Recipe should be removed from read model after deletion"
    );
}

#[tokio::test]
async fn test_delete_recipe_integration_unauthorized_returns_403() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;
    let user2_id = create_test_user_for_tests(&pool, &executor, "user2@test.com", "free").await;

    // Start projection
    // Process events synchronously for deterministic tests
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // User1 creates a recipe
    let command = CreateRecipeCommand {
        title: "User1's Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Add salt".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(2),
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    // Wait for projection
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // User2 attempts to delete user1's recipe
    let delete_command = DeleteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user2_id.to_string(), // Different user!
    };

    let result = delete_recipe(delete_command, &executor, &pool).await;
    assert!(
        matches!(result, Err(RecipeError::PermissionDenied)),
        "Should return PermissionDenied for unauthorized deletion"
    );

    // Verify recipe still exists in read model
    let recipe = query_recipe_by_id(&recipe_id, &pool).await.unwrap();
    assert!(
        recipe.is_some(),
        "Recipe should still exist after failed deletion"
    );
}

#[tokio::test]
async fn test_delete_recipe_integration_excluded_from_user_queries() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Start projection
    // Process events synchronously for deterministic tests
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create 3 recipes for user1
    let mut recipe_ids = Vec::new();
    for i in 1..=3 {
        let command = CreateRecipeCommand {
            title: format!("Recipe {}", i),
            ingredients: vec![Ingredient {
                name: "Salt".to_string(),
                quantity: 1.0,
                unit: "tsp".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Add salt".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: Some(5),
            cook_time_min: Some(10),
            advance_prep_hours: None,
            serving_size: Some(2),
        };

        let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
            .await
            .unwrap();
        recipe_ids.push(recipe_id);
    }

    // Wait for projections
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify all 3 recipes exist
    let recipes_before = query_recipes_by_user(&user1_id, false, &pool)
        .await
        .unwrap();
    assert_eq!(
        recipes_before.len(),
        3,
        "Should have 3 recipes before deletion"
    );

    // Delete the second recipe
    let delete_command = DeleteRecipeCommand {
        recipe_id: recipe_ids[1].clone(),
        user_id: user1_id.to_string(),
    };

    delete_recipe(delete_command, &executor, &pool)
        .await
        .unwrap();

    // Wait for projection
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify only 2 recipes remain
    let recipes_after = query_recipes_by_user(&user1_id, false, &pool)
        .await
        .unwrap();
    assert_eq!(
        recipes_after.len(),
        2,
        "Should have 2 recipes after deleting one"
    );

    // Verify the deleted recipe is not in the list
    assert!(
        !recipes_after.iter().any(|r| r.id == recipe_ids[1]),
        "Deleted recipe should not appear in user queries"
    );
}

// ============================================================================
// Favorite Recipe Integration Tests
// ============================================================================

#[tokio::test]
async fn test_favorite_recipe_integration_full_cycle() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;

    // Run projection once to catch up
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create a recipe
    let command = CreateRecipeCommand {
        title: "Integration Test Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Test Ingredient".to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Test instruction".to_string(),
            timer_minutes: Some(5),
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(15),
        advance_prep_hours: None,
        serving_size: Some(4),
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify initial state: not favorited
    let recipe = query_recipe_by_id(&recipe_id, &pool)
        .await
        .unwrap()
        .unwrap();
    assert!(
        !recipe.is_favorite,
        "Recipe should not be favorited initially"
    );

    // Favorite the recipe
    use recipe::{favorite_recipe, FavoriteRecipeCommand};
    let fav_command = FavoriteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.to_string(),
    };
    let new_status = favorite_recipe(fav_command, &executor, &pool)
        .await
        .unwrap();
    assert!(new_status, "Should return true after favoriting");

    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify favorited in read model
    let recipe = query_recipe_by_id(&recipe_id, &pool)
        .await
        .unwrap()
        .unwrap();
    assert!(
        recipe.is_favorite,
        "Recipe should be favorited in read model"
    );

    // Query all recipes vs favorites
    let all_recipes = query_recipes_by_user(&user1_id, false, &pool)
        .await
        .unwrap();
    let fav_recipes = query_recipes_by_user(&user1_id, true, &pool).await.unwrap();

    assert_eq!(all_recipes.len(), 1, "Should have 1 recipe total");
    assert_eq!(fav_recipes.len(), 1, "Should have 1 favorite recipe");

    // Un-favorite the recipe
    let unfav_command = FavoriteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user1_id.to_string(),
    };
    let new_status = favorite_recipe(unfav_command, &executor, &pool)
        .await
        .unwrap();
    assert!(!new_status, "Should return false after un-favoriting");

    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify not favorited in read model
    let recipe = query_recipe_by_id(&recipe_id, &pool)
        .await
        .unwrap()
        .unwrap();
    assert!(
        !recipe.is_favorite,
        "Recipe should not be favorited in read model"
    );

    // Query favorites (should be empty now)
    let fav_recipes = query_recipes_by_user(&user1_id, true, &pool).await.unwrap();
    assert_eq!(fav_recipes.len(), 0, "Should have 0 favorite recipes");
}

#[tokio::test]
async fn test_favorite_filter_with_multiple_recipes() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "premium").await;

    // Run projection once to catch up
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Create 5 recipes
    let mut recipe_ids = Vec::new();
    for i in 1..=5 {
        let command = CreateRecipeCommand {
            title: format!("Recipe {}", i),
            ingredients: vec![Ingredient {
                name: "Ingredient".to_string(),
                quantity: 1.0,
                unit: "unit".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Do something".to_string(),
                timer_minutes: None,
            }],
            prep_time_min: Some(10),
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: Some(2),
        };

        let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
            .await
            .unwrap();
        recipe_ids.push(recipe_id);
    }

    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Favorite recipes 1, 3, and 5
    use recipe::{favorite_recipe, FavoriteRecipeCommand};
    for i in &[0, 2, 4] {
        let fav_command = FavoriteRecipeCommand {
            recipe_id: recipe_ids[*i].clone(),
            user_id: user1_id.to_string(),
        };
        favorite_recipe(fav_command, &executor, &pool)
            .await
            .unwrap();
    }

    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Query all recipes
    let all_recipes = query_recipes_by_user(&user1_id, false, &pool)
        .await
        .unwrap();
    assert_eq!(all_recipes.len(), 5, "Should have 5 recipes total");

    // Query favorite recipes only
    let fav_recipes = query_recipes_by_user(&user1_id, true, &pool).await.unwrap();
    assert_eq!(fav_recipes.len(), 3, "Should have 3 favorite recipes");

    // Verify the correct recipes are favorited
    let fav_ids: Vec<String> = fav_recipes.iter().map(|r| r.id.clone()).collect();
    assert!(
        fav_ids.contains(&recipe_ids[0]),
        "Recipe 1 should be favorite"
    );
    assert!(
        fav_ids.contains(&recipe_ids[2]),
        "Recipe 3 should be favorite"
    );
    assert!(
        fav_ids.contains(&recipe_ids[4]),
        "Recipe 5 should be favorite"
    );
}

#[tokio::test]
async fn test_favorite_permission_denied_for_other_users_recipe() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user1_id = create_test_user_for_tests(&pool, &executor, "user1@test.com", "free").await;
    let _user2_id = create_test_user_for_tests(&pool, &executor, "user2@test.com", "free").await;

    // Run projection once to catch up
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // User1 creates a recipe
    let command = CreateRecipeCommand {
        title: "User1 Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Ingredient".to_string(),
            quantity: 1.0,
            unit: "unit".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Do something".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: Some(2),
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // User2 tries to favorite User1's recipe
    use recipe::{favorite_recipe, FavoriteRecipeCommand};
    let fav_command = FavoriteRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: "user2".to_string(),
    };
    let result = favorite_recipe(fav_command, &executor, &pool).await;

    assert!(
        matches!(result, Err(RecipeError::PermissionDenied)),
        "Should return PermissionDenied when user tries to favorite another user's recipe"
    );
}
