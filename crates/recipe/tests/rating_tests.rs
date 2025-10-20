use recipe::{
    create_recipe, delete_rating, query_rating_stats, query_recipe_ratings, query_user_rating,
    rate_recipe, share_recipe, CreateRecipeCommand, DeleteRatingCommand, Ingredient,
    InstructionStep, RateRecipeCommand, RecipeError, ShareRecipeCommand,
};
use sqlx::{Pool, Sqlite, SqlitePool};

/// Helper to create in-memory SQLite database for testing
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Run evento migrations for event store tables
    use evento::prelude::*;
    let mut conn = pool.acquire().await.unwrap();
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut *conn, &Plan::apply_all())
        .await
        .unwrap();
    drop(conn);

    // Run SQLx migrations for read model tables (same as production)
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

/// Helper to run projections after events
async fn run_projections(pool: &SqlitePool, executor: &evento::Sqlite) {
    use recipe::recipe_projection;
    recipe_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();
}

/// Helper to create a shared recipe for testing
/// Returns the actual generated recipe_id
async fn create_shared_recipe(
    pool: &SqlitePool,
    executor: &evento::Sqlite,
    user_id: &str,
) -> String {
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
            instruction_text: "Mix".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: None,
        serving_size: Some(4),
    };

    let recipe_id = create_recipe(command, user_id, executor, pool)
        .await
        .unwrap();

    run_projections(pool, executor).await;

    // Share recipe (using the generated recipe_id)
    let share_command = ShareRecipeCommand {
        recipe_id: recipe_id.clone(),
        user_id: user_id.to_string(),
        shared: true,
    };

    share_recipe(share_command, executor, pool).await.unwrap();

    run_projections(pool, executor).await;

    recipe_id
}

#[tokio::test]
async fn test_rate_recipe_success() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    // Setup test data
    insert_test_user(&pool, "user1", "user1@test.com").await;
    let recipe_id = create_shared_recipe(&pool, &executor, "user1").await;

    insert_test_user(&pool, "user2", "user2@test.com").await;

    // Rate recipe
    let command = RateRecipeCommand {
        recipe_id: recipe_id.clone(),
        stars: 5,
        review_text: Some("Great recipe!".to_string()),
    };

    let result = rate_recipe(command, "user2", &executor, &pool).await;
    run_projections(&pool, &executor).await;
    assert!(result.is_ok());

    // Verify rating in database
    let rating = query_user_rating(&recipe_id, "user2", &pool).await.unwrap();
    assert!(rating.is_some());
    let rating = rating.unwrap();
    assert_eq!(rating.stars, 5);
    assert_eq!(rating.review_text, Some("Great recipe!".to_string()));
}

#[tokio::test]
async fn test_rate_recipe_validates_stars_range() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    insert_test_user(&pool, "user1", "user1@test.com").await;
    let recipe_id = create_shared_recipe(&pool, &executor, "user1").await;
    insert_test_user(&pool, "user2", "user2@test.com").await;

    // Test stars < 1
    let command = RateRecipeCommand {
        recipe_id: recipe_id.clone(),
        stars: 0,
        review_text: None,
    };

    let result = rate_recipe(command, "user2", &executor, &pool).await;
    run_projections(&pool, &executor).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RecipeError::ValidationError(_)
    ));

    // Test stars > 5
    let command = RateRecipeCommand {
        recipe_id: recipe_id.clone(),
        stars: 6,
        review_text: None,
    };

    let result = rate_recipe(command, "user2", &executor, &pool).await;
    run_projections(&pool, &executor).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RecipeError::ValidationError(_)
    ));
}

#[tokio::test]
async fn test_rate_recipe_validates_review_length() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    insert_test_user(&pool, "user1", "user1@test.com").await;
    let recipe_id = create_shared_recipe(&pool, &executor, "user1").await;
    insert_test_user(&pool, "user2", "user2@test.com").await;

    // Create review text longer than 500 characters
    let long_review = "a".repeat(501);

    let command = RateRecipeCommand {
        recipe_id: recipe_id.clone(),
        stars: 5,
        review_text: Some(long_review),
    };

    let result = rate_recipe(command, "user2", &executor, &pool).await;
    run_projections(&pool, &executor).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RecipeError::ValidationError(_)
    ));
}

#[tokio::test]
async fn test_rate_recipe_only_shared_recipes() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    insert_test_user(&pool, "user1", "user1@test.com").await;
    insert_test_user(&pool, "user2", "user2@test.com").await;

    // Create recipe but don't share it
    let command = CreateRecipeCommand {
        title: "Private Recipe".to_string(),
        ingredients: vec![Ingredient {
            name: "Salt".to_string(),
            quantity: 1.0,
            unit: "tsp".to_string(),
        }],
        instructions: vec![InstructionStep {
            step_number: 1,
            instruction_text: "Mix".to_string(),
            timer_minutes: None,
        }],
        prep_time_min: Some(10),
        cook_time_min: Some(20),
        advance_prep_hours: None,
        serving_size: Some(4),
    };

    let recipe_id = create_recipe(command, &user1_id, &executor, &pool)
        .await
        .unwrap();

    run_projections(&pool, &executor).await;

    // Try to rate private recipe
    let rate_command = RateRecipeCommand {
        recipe_id: recipe_id.clone(),
        stars: 5,
        review_text: None,
    };

    let result = rate_recipe(rate_command, "user2", &executor, &pool).await;
    run_projections(&pool, &executor).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RecipeError::ValidationError(_)
    ));
}

#[tokio::test]
async fn test_rate_recipe_nonexistent_recipe() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    insert_test_user(&pool, "user1", "user1@test.com").await;

    let command = RateRecipeCommand {
        recipe_id: "nonexistent".to_string(),
        stars: 5,
        review_text: None,
    };

    let result = rate_recipe(command, "user1", &executor, &pool).await;
    run_projections(&pool, &executor).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RecipeError::ValidationError(_)
    ));
}

#[tokio::test]
async fn test_rate_recipe_upsert_behavior() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    insert_test_user(&pool, "user1", "user1@test.com").await;
    let recipe_id = create_shared_recipe(&pool, &executor, "user1").await;
    insert_test_user(&pool, "user2", "user2@test.com").await;

    // First rating
    let command = RateRecipeCommand {
        recipe_id: recipe_id.clone(),
        stars: 3,
        review_text: Some("Okay".to_string()),
    };

    rate_recipe(command, "user2", &executor, &pool)
        .await
        .unwrap();
    run_projections(&pool, &executor).await;

    // Update rating (same user, same recipe)
    let command = RateRecipeCommand {
        recipe_id: recipe_id.clone(),
        stars: 5,
        review_text: Some("Actually great!".to_string()),
    };

    rate_recipe(command, "user2", &executor, &pool)
        .await
        .unwrap();
    run_projections(&pool, &executor).await;

    // Verify only one rating exists
    let rating = query_user_rating(&recipe_id, "user2", &pool)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rating.stars, 5);
    assert_eq!(rating.review_text, Some("Actually great!".to_string()));

    // Verify count is 1, not 2
    let stats = query_rating_stats(&recipe_id, &pool).await.unwrap();
    assert_eq!(stats.review_count, 1);
}

#[tokio::test]
async fn test_delete_rating_success() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    insert_test_user(&pool, "user1", "user1@test.com").await;
    let recipe_id = create_shared_recipe(&pool, &executor, "user1").await;
    insert_test_user(&pool, "user2", "user2@test.com").await;

    // Create rating
    let command = RateRecipeCommand {
        recipe_id: recipe_id.clone(),
        stars: 5,
        review_text: Some("Great!".to_string()),
    };
    rate_recipe(command, "user2", &executor, &pool)
        .await
        .unwrap();
    run_projections(&pool, &executor).await;

    // Delete rating
    let delete_command = DeleteRatingCommand {
        recipe_id: recipe_id.clone(),
    };
    let result = delete_rating(delete_command, "user2", &executor, &pool).await;
    run_projections(&pool, &executor).await;
    assert!(result.is_ok());

    // Verify rating is deleted
    let rating = query_user_rating(&recipe_id, "user2", &pool).await.unwrap();
    assert!(rating.is_none());
}

#[tokio::test]
async fn test_delete_rating_ownership_check() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    insert_test_user(&pool, "user1", "user1@test.com").await;
    let recipe_id = create_shared_recipe(&pool, &executor, "user1").await;
    insert_test_user(&pool, "user2", "user2@test.com").await;
    insert_test_user(&pool, "user3", "user3@test.com").await;

    // User2 creates rating
    let command = RateRecipeCommand {
        recipe_id: recipe_id.clone(),
        stars: 5,
        review_text: Some("Great!".to_string()),
    };
    rate_recipe(command, "user2", &executor, &pool)
        .await
        .unwrap();
    run_projections(&pool, &executor).await;

    // User3 tries to delete user2's rating
    let delete_command = DeleteRatingCommand {
        recipe_id: recipe_id.clone(),
    };
    let result = delete_rating(delete_command, "user3", &executor, &pool).await;
    run_projections(&pool, &executor).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RecipeError::ValidationError(_)
    ));
}

#[tokio::test]
async fn test_query_rating_stats_average() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    insert_test_user(&pool, "user1", "user1@test.com").await;
    let recipe_id = create_shared_recipe(&pool, &executor, "user1").await;

    insert_test_user(&pool, "user2", "user2@test.com").await;
    insert_test_user(&pool, "user3", "user3@test.com").await;
    insert_test_user(&pool, "user4", "user4@test.com").await;

    // Multiple ratings
    rate_recipe(
        RateRecipeCommand {
            recipe_id: recipe_id.clone(),
            stars: 5,
            review_text: None,
        },
        "user2",
        &executor,
        &pool,
    )
    .await
    .unwrap();
    run_projections(&pool, &executor).await;

    rate_recipe(
        RateRecipeCommand {
            recipe_id: recipe_id.clone(),
            stars: 4,
            review_text: None,
        },
        "user3",
        &executor,
        &pool,
    )
    .await
    .unwrap();
    run_projections(&pool, &executor).await;

    rate_recipe(
        RateRecipeCommand {
            recipe_id: recipe_id.clone(),
            stars: 3,
            review_text: None,
        },
        "user4",
        &executor,
        &pool,
    )
    .await
    .unwrap();
    run_projections(&pool, &executor).await;

    // Query stats
    let stats = query_rating_stats(&recipe_id, &pool).await.unwrap();
    assert_eq!(stats.review_count, 3);
    assert_eq!(stats.avg_rating, 4.0); // (5 + 4 + 3) / 3 = 4.0
}

#[tokio::test]
async fn test_query_recipe_ratings_chronological() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    insert_test_user(&pool, "user1", "user1@test.com").await;
    let recipe_id = create_shared_recipe(&pool, &executor, "user1").await;

    insert_test_user(&pool, "user2", "user2@test.com").await;
    insert_test_user(&pool, "user3", "user3@test.com").await;

    // Create ratings with delays to ensure different timestamps
    rate_recipe(
        RateRecipeCommand {
            recipe_id: recipe_id.clone(),
            stars: 5,
            review_text: Some("First review".to_string()),
        },
        "user2",
        &executor,
        &pool,
    )
    .await
    .unwrap();
    run_projections(&pool, &executor).await;

    rate_recipe(
        RateRecipeCommand {
            recipe_id: recipe_id.clone(),
            stars: 4,
            review_text: Some("Second review".to_string()),
        },
        "user3",
        &executor,
        &pool,
    )
    .await
    .unwrap();
    run_projections(&pool, &executor).await;

    // Query ratings
    let ratings = query_recipe_ratings(&recipe_id, &pool).await.unwrap();
    assert_eq!(ratings.len(), 2);

    // Verify chronological order (most recent first)
    assert_eq!(ratings[0].user_id, "user3");
    assert_eq!(ratings[1].user_id, "user2");
}

#[tokio::test]
async fn test_query_rating_stats_no_ratings() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    insert_test_user(&pool, "user1", "user1@test.com").await;
    let recipe_id = create_shared_recipe(&pool, &executor, "user1").await;

    // Query stats for recipe with no ratings
    let stats = query_rating_stats(&recipe_id, &pool).await.unwrap();
    assert_eq!(stats.review_count, 0);
    assert_eq!(stats.avg_rating, 0.0);
}

#[tokio::test]
async fn test_rate_recipe_with_optional_review_text() {
    let pool = setup_test_db().await;
    let executor = setup_evento_executor(pool.clone()).await;

    insert_test_user(&pool, "user1", "user1@test.com").await;
    let recipe_id = create_shared_recipe(&pool, &executor, "user1").await;
    insert_test_user(&pool, "user2", "user2@test.com").await;

    // Rating without review text
    let command = RateRecipeCommand {
        recipe_id: recipe_id.clone(),
        stars: 4,
        review_text: None,
    };

    rate_recipe(command, "user2", &executor, &pool)
        .await
        .unwrap();
    run_projections(&pool, &executor).await;

    let rating = query_user_rating(&recipe_id, "user2", &pool)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rating.stars, 4);
    assert_eq!(rating.review_text, None);
}
