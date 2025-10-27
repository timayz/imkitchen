//! Tests for Story 9.4: Update Recipe Creation Form with Accompaniment Fields
//!
//! Acceptance Criteria Coverage:
//! - AC 9.4.2: Recipe type includes "accompaniment"
//! - AC 9.4.3: Main course accepts accompaniment checkbox
//! - AC 9.4.4: Preferred accompaniment categories checkboxes
//! - AC 9.4.5: Accompaniment category radio buttons
//! - AC 9.4.6: Cuisine selection with custom option
//! - AC 9.4.7: Dietary tags checkboxes
//! - AC 9.4.9: Validation - accompaniment requires category
//! - AC 9.4.10: Integration tests verify form submission creates recipe with accompaniment data

use recipe::{
    create_recipe, query_recipe_by_id, recipe_projection, AccompanimentCategory,
    CreateRecipeCommand, Cuisine, DietaryTag, Ingredient, InstructionStep, RecipeError,
};
use sqlx::{Pool, Sqlite, SqlitePool};

/// Helper to create in-memory SQLite database for testing
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Run evento migrations for event store tables
    use evento::migrator::{Migrate, Plan};
    let mut conn = pool.acquire().await.unwrap();
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut *conn, &Plan::apply_all())
        .await
        .unwrap();
    drop(conn);

    // Run SQLx migrations for read model tables
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();

    pool
}

/// Helper to create in-memory evento executor for testing
async fn setup_evento_executor(pool: Pool<Sqlite>) -> evento::Sqlite {
    pool.into()
}

/// Create a test user using proper evento commands
async fn create_test_user(pool: &SqlitePool, executor: &evento::Sqlite, email: &str) -> String {
    use user::commands::{register_user, RegisterUserCommand};

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

    user::user_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();

    user_id
}

/// AC 9.4.10: Test creating a main course recipe with accepts_accompaniment and preferred categories
#[tokio::test]
async fn test_create_main_course_with_accompaniment_preferences() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "mainuser@test.com").await;

    // Create main course recipe that accepts accompaniments
    let command = CreateRecipeCommand {
        title: "Grilled Chicken with Herbs".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![
            Ingredient {
                name: "Chicken breast".to_string(),
                quantity: 500.0,
                unit: "grams".to_string(),
            },
            Ingredient {
                name: "Olive oil".to_string(),
                quantity: 2.0,
                unit: "tablespoons".to_string(),
            },
        ],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Grill chicken until cooked through".to_string(),
            timer_minutes: Some(20),
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: None,
        serving_size: Some(4),
        // AC 9.4.3: Accepts accompaniment checkbox
        accepts_accompaniment: true,
        // AC 9.4.4: Preferred accompaniment categories
        preferred_accompaniments: vec!["Rice".to_string(), "Salad".to_string()],
        accompaniment_category: None,
        // AC 9.4.6: Cuisine selection
        cuisine: Some("Mediterranean".to_string()),
        // AC 9.4.7: Dietary tags
        dietary_tags: vec!["Gluten-Free".to_string(), "Dairy-Free".to_string()],
    };

    println!(
        "DEBUG: command.cuisine before create_recipe = {:?}",
        command.cuisine
    );
    println!(
        "DEBUG: command.dietary_tags before create_recipe = {:?}",
        command.dietary_tags
    );

    // Test parse functions directly
    use recipe::{Cuisine, DietaryTag};
    let test_cuisine = command.cuisine.as_ref().map(|s| {
        let parsed = match s.as_str() {
            "Mediterranean" => Some(Cuisine::Mediterranean),
            _ => None,
        };
        println!("DEBUG: Manually parsing '{}' -> {:?}", s, parsed);
        parsed
    });
    println!("DEBUG: test_cuisine = {:?}", test_cuisine);

    let recipe_id = create_recipe(command, &user_id, &executor, &pool, true)
        .await
        .unwrap();

    // Run projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify recipe was created with correct accompaniment settings
    let recipe = query_recipe_by_id(&recipe_id, &pool)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(recipe.title, "Grilled Chicken with Herbs");
    assert_eq!(recipe.recipe_type, "main_course");
    assert!(recipe.accepts_accompaniment);

    // Deserialize preferred_accompaniments from JSON
    let preferred: Vec<AccompanimentCategory> = recipe
        .preferred_accompaniments
        .as_ref()
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default();
    assert_eq!(preferred.len(), 2);
    assert!(preferred.contains(&AccompanimentCategory::Rice));
    assert!(preferred.contains(&AccompanimentCategory::Salad));

    // Deserialize cuisine from JSON
    println!("DEBUG: recipe.cuisine = {:?}", recipe.cuisine);
    let cuisine_enum: Option<Cuisine> = recipe.cuisine.as_ref().and_then(|json| {
        println!("DEBUG: Attempting to deserialize: {}", json);
        serde_json::from_str::<Cuisine>(json).ok()
    });
    println!("DEBUG: cuisine_enum = {:?}", cuisine_enum);
    assert_eq!(cuisine_enum, Some(Cuisine::Mediterranean));

    // Deserialize dietary_tags from JSON
    println!("DEBUG: recipe.dietary_tags = {:?}", recipe.dietary_tags);
    let dietary: Vec<DietaryTag> = recipe
        .dietary_tags
        .as_ref()
        .and_then(|json| {
            println!("DEBUG: Attempting to deserialize dietary_tags: {}", json);
            serde_json::from_str(json).ok()
        })
        .unwrap_or_default();
    println!("DEBUG: dietary = {:?}", dietary);
    assert_eq!(dietary.len(), 2);
    assert!(dietary.contains(&DietaryTag::GlutenFree));
    assert!(dietary.contains(&DietaryTag::DairyFree));
}

/// AC 9.4.10: Test creating an accompaniment recipe with category
#[tokio::test]
async fn test_create_accompaniment_recipe_with_category() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "sideuser@test.com").await;

    // AC 9.4.2: Recipe type includes "accompaniment"
    let command = CreateRecipeCommand {
        title: "Garlic Butter Rice".to_string(),
        recipe_type: "accompaniment".to_string(),
        ingredients: vec![
            Ingredient {
                name: "Basmati rice".to_string(),
                quantity: 200.0,
                unit: "grams".to_string(),
            },
            Ingredient {
                name: "Butter".to_string(),
                quantity: 30.0,
                unit: "grams".to_string(),
            },
            Ingredient {
                name: "Garlic".to_string(),
                quantity: 3.0,
                unit: "cloves".to_string(),
            },
        ],
        instructions: vec![
            InstructionStep {
                step_number: 1,
                instruction_text: "Cook rice according to package directions".to_string(),
                timer_minutes: Some(15),
            },
            InstructionStep {
                step_number: 2,
                instruction_text: "Melt butter and sauté minced garlic".to_string(),
                timer_minutes: Some(2),
            },
            InstructionStep {
                step_number: 3,
                instruction_text: "Mix garlic butter with cooked rice".to_string(),
                timer_minutes: None,
            },
        ],
        prep_time_min: Some(5),
        cook_time_min: Some(17),
        advance_prep_hours: None,
        serving_size: Some(4),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        // AC 9.4.5: Accompaniment category required
        accompaniment_category: Some("Rice".to_string()),
        cuisine: Some("Indian".to_string()),
        dietary_tags: vec!["Vegetarian".to_string()],
    };

    let recipe_id = create_recipe(command, &user_id, &executor, &pool, true)
        .await
        .unwrap();

    // Run projection to sync read model
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify accompaniment recipe was created correctly
    let recipe = query_recipe_by_id(&recipe_id, &pool)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(recipe.title, "Garlic Butter Rice");
    assert_eq!(recipe.recipe_type, "accompaniment");
    assert!(!recipe.accepts_accompaniment);

    // Deserialize accompaniment_category from JSON
    let category: Option<AccompanimentCategory> = recipe
        .accompaniment_category
        .as_ref()
        .and_then(|json| serde_json::from_str(json).ok());
    assert_eq!(category, Some(AccompanimentCategory::Rice));

    // Deserialize cuisine from JSON
    let cuisine_enum: Option<Cuisine> = recipe
        .cuisine
        .as_ref()
        .and_then(|json| serde_json::from_str::<Cuisine>(json).ok());
    assert_eq!(cuisine_enum, Some(Cuisine::Indian));

    // Deserialize dietary_tags from JSON
    let dietary: Vec<DietaryTag> = recipe
        .dietary_tags
        .as_ref()
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default();
    assert_eq!(dietary.len(), 1);
    assert!(dietary.contains(&DietaryTag::Vegetarian));
}

/// AC 9.4.9: Test validation error when accompaniment type but no category selected
#[tokio::test]
async fn test_accompaniment_validation_requires_category() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "validuser@test.com").await;

    // Try to create accompaniment without category - should fail validation
    let command = CreateRecipeCommand {
        title: "Invalid Side Dish".to_string(),
        recipe_type: "accompaniment".to_string(),
        ingredients: vec![Ingredient {
            name: "Test ingredient".to_string(),
            quantity: 100.0,
            unit: "grams".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook it".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: None,
        cook_time_min: None,
        advance_prep_hours: None,
        serving_size: None,
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        // AC 9.4.9: Missing category should trigger validation error
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let result = create_recipe(command, &user_id, &executor, &pool, true).await;

    // Should return validation error
    assert!(result.is_err());
    match result.unwrap_err() {
        RecipeError::ValidationError(msg) => {
            assert!(msg.contains("Accompaniment category is required"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

/// AC 9.4.10: Test recipe with custom cuisine input
#[tokio::test]
async fn test_create_recipe_with_custom_cuisine() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "customuser@test.com").await;

    let command = CreateRecipeCommand {
        title: "Fusion Stir-Fry".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Mixed vegetables".to_string(),
            quantity: 300.0,
            unit: "grams".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Stir-fry vegetables".to_string(),
            timer_minutes: Some(10),
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(10),
        advance_prep_hours: None,
        serving_size: Some(2),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        // AC 9.4.6: Custom cuisine support
        cuisine: Some("Asian-Fusion".to_string()),
        dietary_tags: vec!["Vegan".to_string(), "Gluten-Free".to_string()],
    };

    let recipe_id = create_recipe(command, &user_id, &executor, &pool, true)
        .await
        .unwrap();

    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    let recipe = query_recipe_by_id(&recipe_id, &pool)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(recipe.title, "Fusion Stir-Fry");

    // Deserialize custom cuisine from JSON
    let cuisine_enum: Option<Cuisine> = recipe
        .cuisine
        .as_ref()
        .and_then(|json| serde_json::from_str::<Cuisine>(json).ok());
    assert_eq!(
        cuisine_enum,
        Some(Cuisine::Custom("Asian-Fusion".to_string()))
    );
}

/// AC 9.4.10: Test all dietary tags can be set
#[tokio::test]
async fn test_all_dietary_tags() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "dietuser@test.com").await;

    let command = CreateRecipeCommand {
        title: "Multi-Dietary Recipe".to_string(),
        recipe_type: "dessert".to_string(),
        ingredients: vec![Ingredient {
            name: "Test ingredient".to_string(),
            quantity: 100.0,
            unit: "grams".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Prepare dessert".to_string(),
            timer_minutes: Some(30),
        }],
        prep_time_min: Some(15),
        cook_time_min: Some(30),
        advance_prep_hours: None,
        serving_size: Some(6),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: Some("American".to_string()),
        // AC 9.4.7: All dietary tags
        dietary_tags: vec![
            "Vegetarian".to_string(),
            "Vegan".to_string(),
            "Gluten-Free".to_string(),
            "Dairy-Free".to_string(),
            "Nut-Free".to_string(),
        ],
    };

    let recipe_id = create_recipe(command, &user_id, &executor, &pool, true)
        .await
        .unwrap();

    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    let recipe = query_recipe_by_id(&recipe_id, &pool)
        .await
        .unwrap()
        .unwrap();

    // Deserialize dietary_tags from JSON
    let dietary: Vec<DietaryTag> = recipe
        .dietary_tags
        .as_ref()
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default();
    assert_eq!(dietary.len(), 5);
    assert!(dietary.contains(&DietaryTag::Vegetarian));
    assert!(dietary.contains(&DietaryTag::Vegan));
    assert!(dietary.contains(&DietaryTag::GlutenFree));
    assert!(dietary.contains(&DietaryTag::DairyFree));
    assert!(dietary.contains(&DietaryTag::NutFree));
}

/// AC 9.4.10: Test empty preferred accompaniments for main course
#[tokio::test]
async fn test_main_course_without_preferred_accompaniments() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "simple@test.com").await;

    let command = CreateRecipeCommand {
        title: "Simple Main Course".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Main ingredient".to_string(),
            quantity: 200.0,
            unit: "grams".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook main course".to_string(),
            timer_minutes: Some(25),
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(25),
        advance_prep_hours: None,
        serving_size: Some(2),
        // Main course doesn't accept accompaniment
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
        cuisine: None,
        dietary_tags: vec![],
    };

    let recipe_id = create_recipe(command, &user_id, &executor, &pool, true)
        .await
        .unwrap();

    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    let recipe = query_recipe_by_id(&recipe_id, &pool)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(recipe.recipe_type, "main_course");
    assert!(!recipe.accepts_accompaniment);

    // Deserialize preferred_accompaniments from JSON
    let preferred: Vec<AccompanimentCategory> = recipe
        .preferred_accompaniments
        .as_ref()
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default();
    assert!(preferred.is_empty());
}
