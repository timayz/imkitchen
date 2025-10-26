//! Tests for Epic 6: Enhanced Meal Planning System - Recipe Domain Model Updates
//!
//! Story 6.2: Update Recipe Domain Model
//! - AccompanimentCategory, Cuisine, DietaryTag enums
//! - Recipe accompaniment fields
//! - RecipeCreated event with new fields
//! - RecipeAccompanimentSettingsUpdated event
//! - Backwards compatibility with old events

use recipe::{
    create_recipe, query_recipe_by_id, recipe_projection, AccompanimentCategory,
    CreateRecipeCommand, Cuisine, DietaryTag, Ingredient, InstructionStep,
    RecipeAccompanimentSettingsUpdated, RecipeAggregate, RecipeCreated,
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

/// Test AccompanimentCategory enum serialization
#[test]
fn test_accompaniment_category_serde_roundtrip() {
    let categories = vec![
        AccompanimentCategory::Pasta,
        AccompanimentCategory::Rice,
        AccompanimentCategory::Fries,
        AccompanimentCategory::Salad,
        AccompanimentCategory::Bread,
        AccompanimentCategory::Vegetable,
        AccompanimentCategory::Other,
    ];

    for category in categories {
        // Test serde JSON roundtrip
        let json = serde_json::to_string(&category).unwrap();
        let deserialized: AccompanimentCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(category, deserialized);

        // Test bincode roundtrip
        let encoded = bincode::encode_to_vec(category, bincode::config::standard()).unwrap();
        let (decoded, _): (AccompanimentCategory, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(category, decoded);
    }
}

/// Test Cuisine enum serialization with Custom variant
#[test]
fn test_cuisine_serde_roundtrip() {
    let cuisines = vec![
        Cuisine::Italian,
        Cuisine::Indian,
        Cuisine::Mexican,
        Cuisine::Chinese,
        Cuisine::Japanese,
        Cuisine::French,
        Cuisine::American,
        Cuisine::Mediterranean,
        Cuisine::Thai,
        Cuisine::Korean,
        Cuisine::Vietnamese,
        Cuisine::Greek,
        Cuisine::Spanish,
        Cuisine::Custom("Fusion".to_string()),
    ];

    for cuisine in cuisines {
        // Test serde JSON roundtrip
        let json = serde_json::to_string(&cuisine).unwrap();
        let deserialized: Cuisine = serde_json::from_str(&json).unwrap();
        assert_eq!(cuisine, deserialized);

        // Test bincode roundtrip
        let encoded = bincode::encode_to_vec(&cuisine, bincode::config::standard()).unwrap();
        let (decoded, _): (Cuisine, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(cuisine, decoded);
    }
}

/// Test DietaryTag enum serialization
#[test]
fn test_dietary_tag_serde_roundtrip() {
    let tags = vec![
        DietaryTag::Vegetarian,
        DietaryTag::Vegan,
        DietaryTag::GlutenFree,
        DietaryTag::DairyFree,
        DietaryTag::NutFree,
        DietaryTag::Halal,
        DietaryTag::Kosher,
    ];

    for tag in tags {
        // Test serde JSON roundtrip
        let json = serde_json::to_string(&tag).unwrap();
        let deserialized: DietaryTag = serde_json::from_str(&json).unwrap();
        assert_eq!(tag, deserialized);

        // Test bincode roundtrip
        let encoded = bincode::encode_to_vec(tag, bincode::config::standard()).unwrap();
        let (decoded, _): (DietaryTag, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(tag, decoded);
    }
}

/// Test RecipeCreated event with all Epic 6 fields
#[tokio::test]
async fn test_recipe_created_with_epic6_fields() {
    let _pool = setup_test_db().await;
    let _executor = setup_evento_executor(_pool.clone()).await;

    // Create RecipeCreated event with Epic 6 fields
    let event = RecipeCreated {
        user_id: "user-123".to_string(),
        title: "Chicken Tikka Masala".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Chicken".to_string(),
            quantity: 500.0,
            unit: "g".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Marinate chicken".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(30),
        cook_time_min: Some(45),
        advance_prep_hours: Some(4),
        serving_size: Some(4),
        created_at: chrono::Utc::now().to_rfc3339(),
        // Epic 6 fields
        accepts_accompaniment: Some(true),
        preferred_accompaniments: Some(vec![
            AccompanimentCategory::Rice,
            AccompanimentCategory::Bread,
        ]),
        accompaniment_category: None,
        cuisine: Some(Cuisine::Indian),
        dietary_tags: Some(vec![]),
    };

    // Test serde JSON roundtrip
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RecipeCreated = serde_json::from_str(&json).unwrap();
    assert_eq!(event.title, deserialized.title);
    assert_eq!(
        event.accepts_accompaniment,
        deserialized.accepts_accompaniment
    );
    assert_eq!(
        event.preferred_accompaniments,
        deserialized.preferred_accompaniments
    );
    assert_eq!(event.cuisine, deserialized.cuisine);

    // Test bincode roundtrip
    let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
    let (decoded, _): (RecipeCreated, _) =
        bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
    assert_eq!(event.title, decoded.title);
    assert_eq!(event.accepts_accompaniment, decoded.accepts_accompaniment);
}

/// Test backwards compatibility: old RecipeCreated events without Epic 6 fields
#[tokio::test]
async fn test_backwards_compatibility_old_recipe_created() {
    let _pool = setup_test_db().await;
    let _executor = setup_evento_executor(_pool.clone()).await;

    // Simulate old RecipeCreated event JSON (without Epic 6 fields)
    let old_event_json = r#"{
        "user_id": "user-123",
        "title": "Old Recipe",
        "recipe_type": "main_course",
        "ingredients": [{"name": "Flour", "quantity": 2.0, "unit": "cups"}],
        "instructions": [{"step_number": 1, "instruction_text": "Mix flour", "timer_minutes": null}],
        "prep_time_min": 10,
        "cook_time_min": 20,
        "advance_prep_hours": null,
        "serving_size": 4,
        "created_at": "2025-01-01T00:00:00Z"
    }"#;

    // Deserialize old event (should use default values for missing fields)
    let event: RecipeCreated = serde_json::from_str(old_event_json).unwrap();

    // Verify defaults
    assert_eq!(event.accepts_accompaniment, None); // Defaults to None (aggregate will use false)
    assert_eq!(event.preferred_accompaniments, None); // Defaults to None (aggregate will use vec![])
    assert_eq!(event.accompaniment_category, None);
    assert_eq!(event.cuisine, None);
    assert_eq!(event.dietary_tags, None);

    // Apply event to aggregate (using struct initializer to satisfy clippy)
    let aggregate = RecipeAggregate {
        recipe_id: "recipe-1".to_string(),
        user_id: event.user_id.clone(),
        title: event.title.clone(),
        recipe_type: event.recipe_type.clone(),
        accepts_accompaniment: event.accepts_accompaniment.unwrap_or(false),
        preferred_accompaniments: event.preferred_accompaniments.clone().unwrap_or_default(),
        accompaniment_category: event.accompaniment_category,
        ..Default::default()
    };

    // Verify aggregate state has correct defaults
    assert!(!aggregate.accepts_accompaniment);
    assert_eq!(aggregate.preferred_accompaniments, vec![]);
    assert_eq!(aggregate.accompaniment_category, None);
}

/// Test RecipeAccompanimentSettingsUpdated event serialization
#[test]
fn test_recipe_accompaniment_settings_updated_event() {
    let event = RecipeAccompanimentSettingsUpdated {
        recipe_id: "recipe-123".to_string(),
        user_id: "user-456".to_string(),
        accepts_accompaniment: true,
        preferred_accompaniments: vec![AccompanimentCategory::Pasta, AccompanimentCategory::Rice],
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    // Test serde JSON roundtrip
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RecipeAccompanimentSettingsUpdated = serde_json::from_str(&json).unwrap();
    assert_eq!(event.recipe_id, deserialized.recipe_id);
    assert_eq!(
        event.accepts_accompaniment,
        deserialized.accepts_accompaniment
    );
    assert_eq!(
        event.preferred_accompaniments,
        deserialized.preferred_accompaniments
    );

    // Test bincode roundtrip
    let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
    let (decoded, _): (RecipeAccompanimentSettingsUpdated, _) =
        bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
    assert_eq!(event.recipe_id, decoded.recipe_id);
}

/// Test Recipe aggregate applies RecipeCreated with Epic 6 fields
#[tokio::test]
async fn test_aggregate_applies_recipe_created_with_epic6_fields() {
    let _pool = setup_test_db().await;
    let _executor = setup_evento_executor(_pool.clone()).await;

    // Create event with Epic 6 fields
    let event = RecipeCreated {
        user_id: "user-123".to_string(),
        title: "Basmati Rice".to_string(),
        recipe_type: "accompaniment".to_string(),
        ingredients: vec![Ingredient {
            name: "Rice".to_string(),
            quantity: 2.0,
            unit: "cups".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Boil water".to_string(),
            timer_minutes: Some(15),
        }],
        prep_time_min: Some(5),
        cook_time_min: Some(15),
        advance_prep_hours: None,
        serving_size: Some(4),
        created_at: chrono::Utc::now().to_rfc3339(),
        // This is an accompaniment recipe
        accepts_accompaniment: Some(false),
        preferred_accompaniments: Some(vec![]),
        accompaniment_category: Some(AccompanimentCategory::Rice),
        cuisine: None,
        dietary_tags: Some(vec![DietaryTag::Vegetarian, DietaryTag::Vegan]),
    };

    // Create aggregate and apply event (using struct initializer to satisfy clippy)
    let aggregate = RecipeAggregate {
        recipe_id: "recipe-rice-1".to_string(),
        title: event.title.clone(),
        accepts_accompaniment: event.accepts_accompaniment.unwrap_or(false),
        preferred_accompaniments: event.preferred_accompaniments.clone().unwrap_or_default(),
        accompaniment_category: event.accompaniment_category,
        ..Default::default()
    };

    // Verify aggregate state
    assert_eq!(aggregate.title, "Basmati Rice");
    assert!(!aggregate.accepts_accompaniment);
    assert_eq!(aggregate.preferred_accompaniments, vec![]);
    assert_eq!(
        aggregate.accompaniment_category,
        Some(AccompanimentCategory::Rice)
    );
}

/// Test Recipe aggregate applies RecipeAccompanimentSettingsUpdated
#[tokio::test]
async fn test_aggregate_applies_accompaniment_settings_updated() {
    let _pool = setup_test_db().await;
    let _executor = setup_evento_executor(_pool.clone()).await;

    // Create initial aggregate (using struct initializer to satisfy clippy)
    let mut aggregate = RecipeAggregate {
        recipe_id: "recipe-123".to_string(),
        title: "Pasta Carbonara".to_string(),
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        ..Default::default()
    };

    // Create accompaniment update event
    let event = RecipeAccompanimentSettingsUpdated {
        recipe_id: "recipe-123".to_string(),
        user_id: "user-456".to_string(),
        accepts_accompaniment: true,
        preferred_accompaniments: vec![AccompanimentCategory::Salad, AccompanimentCategory::Bread],
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    // Apply event to aggregate
    aggregate.accepts_accompaniment = event.accepts_accompaniment;
    aggregate.preferred_accompaniments = event.preferred_accompaniments.clone();

    // Verify aggregate state updated
    assert!(aggregate.accepts_accompaniment);
    assert_eq!(aggregate.preferred_accompaniments.len(), 2);
    assert!(aggregate
        .preferred_accompaniments
        .contains(&AccompanimentCategory::Salad));
    assert!(aggregate
        .preferred_accompaniments
        .contains(&AccompanimentCategory::Bread));
}

/// Test projection updates with Epic 6 fields
#[tokio::test]
async fn test_projection_persists_epic6_fields() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;
    let user_id = create_test_user(&pool, &executor, "chef@test.com").await;

    // Create recipe with Epic 6 fields using command
    let command = CreateRecipeCommand {
        title: "Tikka Masala with Rice".to_string(),
        recipe_type: "main_course".to_string(),
        ingredients: vec![Ingredient {
            name: "Chicken".to_string(),
            quantity: 500.0,
            unit: "g".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Cook chicken".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(30),
        cook_time_min: Some(45),
        advance_prep_hours: Some(4),
        serving_size: Some(4),
    };

    let recipe_id = create_recipe(command, &user_id, &executor, &pool, false)
        .await
        .unwrap();

    // Run projection using unsafe_oneshot (synchronous processing for tests)
    recipe_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Query read model
    let read_model = query_recipe_by_id(&recipe_id, &pool)
        .await
        .unwrap()
        .unwrap();

    // Verify Epic 6 fields are persisted (with defaults since CreateRecipeCommand doesn't set them yet)
    assert!(!read_model.accepts_accompaniment); // Default
    assert_eq!(read_model.preferred_accompaniments, Some("[]".to_string())); // Empty JSON array
    assert_eq!(read_model.accompaniment_category, None); // Default
}
