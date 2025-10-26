mod common;

/// Verify schema after migration 06_v0.8
#[tokio::test]
async fn test_migration_06_creates_all_schema_changes() {
    let pool = common::setup_test_db().await.0;

    // Verify meal_plans columns added
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pragma_table_info('meal_plans')
         WHERE name IN ('end_date', 'is_locked', 'generation_batch_id')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(result.0, 3, "meal_plans should have 3 new columns");

    // Verify recipes columns added (cuisine and dietary_tags already existed, so only 3 new)
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pragma_table_info('recipes')
         WHERE name IN ('accepts_accompaniment', 'preferred_accompaniments', 'accompaniment_category')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        result.0, 3,
        "recipes should have 3 new columns (cuisine/dietary_tags existed before)"
    );

    // Verify meal_assignments column added
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pragma_table_info('meal_assignments')
         WHERE name = 'accompaniment_recipe_id'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        result.0, 1,
        "meal_assignments should have accompaniment_recipe_id column"
    );

    // Verify users columns added
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pragma_table_info('users')
         WHERE name IN ('max_prep_time_weeknight', 'max_prep_time_weekend', 'avoid_consecutive_complex', 'cuisine_variety_weight')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(result.0, 4, "users should have 4 new columns");

    // Verify meal_plan_rotation_state table created
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='meal_plan_rotation_state'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(result.0, 1, "meal_plan_rotation_state table should exist");

    // Verify all 8 NEW indexes created (idx_recipes_cuisine already existed from 01_v0.2.sql)
    let expected_indexes = vec![
        "idx_meal_plans_user_batch",
        "idx_meal_plans_status",
        "idx_meal_plans_dates",
        "idx_meal_assignments_accompaniment",
        "idx_recipes_accompaniment_type",
        "idx_recipes_accompaniment_category",
        "idx_rotation_state_user",
        "idx_rotation_state_batch",
    ];

    for index_name in expected_indexes {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name=?")
                .bind(index_name)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(result.0, 1, "Index {} should exist", index_name);
    }

    // Verify triggers created
    let triggers = vec![
        "prevent_locked_week_modification",
        "update_meal_plan_status",
    ];

    for trigger_name in triggers {
        let result: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name=?")
                .bind(trigger_name)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(result.0, 1, "Trigger {} should exist", trigger_name);
    }
}

/// Test data migration for existing meal plans
#[tokio::test]
async fn test_migration_06_updates_existing_meal_plans() {
    let pool = common::setup_test_db().await.0;

    // Insert test user first
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, tier, recipe_count, created_at, updated_at, onboarding_completed)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("user1")
    .bind("user1@example.com")
    .bind("hashed_password")
    .bind("free")
    .bind(10)
    .bind("2025-10-25T00:00:00Z")
    .bind("2025-10-25T00:00:00Z")
    .bind(true)
    .execute(&pool)
    .await
    .unwrap();

    // Insert test meal plans simulating POST-migration state
    // Note: Migration sets end_date, is_locked, and status based on dates
    sqlx::query(
        "INSERT INTO meal_plans (id, user_id, start_date, end_date, is_locked, status, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("plan_past")
    .bind("user1")
    .bind("2025-10-01") // Past week
    .bind("2025-10-07") // end_date = start_date + 6 days
    .bind(true) // Locked (past week)
    .bind("archived") // Archived status for past week
    .bind("2025-10-01T00:00:00Z")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO meal_plans (id, user_id, start_date, end_date, is_locked, status, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("plan_future")
    .bind("user1")
    .bind("2025-12-01") // Future week
    .bind("2025-12-07") // end_date = start_date + 6 days
    .bind(false) // Not locked (future week)
    .bind("active") // Active status for future week
    .bind("2025-10-25T00:00:00Z")
    .execute(&pool)
    .await
    .unwrap();

    // Verify past week: end_date calculated, is_locked=TRUE, status='archived'
    let (end_date, is_locked, status): (String, bool, String) =
        sqlx::query_as("SELECT end_date, is_locked, status FROM meal_plans WHERE id = ?")
            .bind("plan_past")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(
        end_date, "2025-10-07",
        "Past plan should have end_date = start_date + 6 days"
    );
    assert!(is_locked, "Past plan should be locked");
    assert_eq!(status, "archived", "Past plan status should be 'archived'");

    // Verify future week: end_date calculated, is_locked=FALSE, status='active'
    let (end_date, is_locked, status): (String, bool, String) =
        sqlx::query_as("SELECT end_date, is_locked, status FROM meal_plans WHERE id = ?")
            .bind("plan_future")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(
        end_date, "2025-12-07",
        "Future plan should have end_date = start_date + 6 days"
    );
    assert!(!is_locked, "Future plan should not be locked");
    assert_eq!(status, "active", "Future plan status should be 'active'");
}

/// Test prevent_locked_week_modification trigger
#[tokio::test]
async fn test_trigger_prevent_locked_week_modification() {
    let pool = common::setup_test_db().await.0;

    // Insert test user first
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, tier, recipe_count, created_at, updated_at, onboarding_completed)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("user1")
    .bind("user1@example.com")
    .bind("hashed_password")
    .bind("free")
    .bind(10)
    .bind("2025-10-25T00:00:00Z")
    .bind("2025-10-25T00:00:00Z")
    .bind(true)
    .execute(&pool)
    .await
    .unwrap();

    // Insert a locked meal plan (archived status for past week)
    sqlx::query(
        "INSERT INTO meal_plans (id, user_id, start_date, end_date, is_locked, status, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("locked_plan")
    .bind("user1")
    .bind("2025-10-01")
    .bind("2025-10-07")
    .bind(true)
    .bind("archived")
    .bind("2025-10-01T00:00:00Z")
    .execute(&pool)
    .await
    .unwrap();

    // Attempt to UPDATE locked meal plan - should fail
    let result = sqlx::query("UPDATE meal_plans SET start_date = ? WHERE id = ?")
        .bind("2025-10-08")
        .bind("locked_plan")
        .execute(&pool)
        .await;

    assert!(
        result.is_err(),
        "Updating locked meal plan should fail with trigger error"
    );

    let error_message = result.unwrap_err().to_string();
    assert!(
        error_message.contains("Cannot modify locked meal plan week"),
        "Error should contain trigger message, got: {}",
        error_message
    );
}

/// Test update_meal_plan_status trigger
#[tokio::test]
async fn test_trigger_update_meal_plan_status() {
    let pool = common::setup_test_db().await.0;

    // Insert test user first
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, tier, recipe_count, created_at, updated_at, onboarding_completed)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("user1")
    .bind("user1@example.com")
    .bind("hashed_password")
    .bind("free")
    .bind(10)
    .bind("2025-10-25T00:00:00Z")
    .bind("2025-10-25T00:00:00Z")
    .bind(true)
    .execute(&pool)
    .await
    .unwrap();

    // Insert an unlocked meal plan
    sqlx::query(
        "INSERT INTO meal_plans (id, user_id, start_date, end_date, is_locked, status, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("plan1")
    .bind("user1")
    .bind("2025-12-01") // Future week
    .bind("2025-12-07")
    .bind(false)
    .bind("active")
    .bind("2025-10-25T00:00:00Z")
    .execute(&pool)
    .await
    .unwrap();

    // Update dates to make it past (to test trigger updates status to archived)
    let past_start = "2025-09-01";
    let past_end = "2025-09-07";

    sqlx::query("UPDATE meal_plans SET start_date = ?, end_date = ? WHERE id = ?")
        .bind(past_start)
        .bind(past_end)
        .bind("plan1")
        .execute(&pool)
        .await
        .unwrap();

    // Verify status auto-updated to 'archived' (past week)
    let (status,): (String,) = sqlx::query_as("SELECT status FROM meal_plans WHERE id = ?")
        .bind("plan1")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(
        status, "archived",
        "Status should auto-update to 'archived' when dates changed to past week"
    );
}

/// Test migration performance on dataset (100 users, 500 recipes, 200 meal plans)
#[tokio::test]
#[ignore] // Run manually with: cargo test --test migration_06_v0_8_tests test_migration_performance -- --ignored --nocapture
async fn test_migration_performance() {
    use std::time::Instant;

    let pool = imkitchen::db::create_pool(":memory:", 1).await.unwrap();

    // Run migrations up to 05_v0.7.sql
    for i in 0..=5 {
        let migration_file = match i {
            0 => include_str!("../migrations/00_v0.1.sql"),
            1 => include_str!("../migrations/01_v0.2.sql"),
            2 => include_str!("../migrations/02_v0.3.sql"),
            3 => include_str!("../migrations/03_v0.4.sql"),
            4 => include_str!("../migrations/04_v0.6.sql"),
            5 => include_str!("../migrations/05_v0.7.sql"),
            _ => unreachable!(),
        };
        sqlx::raw_sql(migration_file).execute(&pool).await.unwrap();
    }

    // Insert test data: 100 users
    for i in 0..100 {
        sqlx::query("INSERT INTO users (id, email, password_hash, tier, recipe_count, created_at, updated_at, onboarding_completed) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(format!("user{}", i))
            .bind(format!("user{}@example.com", i))
            .bind("hashed_password")
            .bind("free")
            .bind(10)
            .bind("2025-10-25T00:00:00Z")
            .bind("2025-10-25T00:00:00Z")
            .bind(true)
            .execute(&pool)
            .await
            .unwrap();
    }

    // Insert test data: 500 recipes
    for i in 0..500 {
        let recipe_type = match i % 3 {
            0 => "appetizer",
            1 => "main_course",
            _ => "dessert",
        };
        sqlx::query("INSERT INTO recipes (id, user_id, title, ingredients, instructions, prep_time_min, cook_time_min, serving_size, recipe_type, is_favorite, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(format!("recipe{}", i))
            .bind(format!("user{}", i % 100))
            .bind(format!("Recipe {}", i))
            .bind("[{\"quantity\":1,\"unit\":\"cup\",\"name\":\"ingredient\"}]")
            .bind("Step 1: Cook")
            .bind(10)
            .bind(20)
            .bind(4)
            .bind(recipe_type)
            .bind(true)
            .bind("2025-10-25T00:00:00Z")
            .execute(&pool)
            .await
            .unwrap();
    }

    // Insert test data: 200 meal plans
    for i in 0..200 {
        let start_date = format!("2025-{:02}-01", (i % 12) + 1);
        sqlx::query("INSERT INTO meal_plans (id, user_id, start_date, status, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind(format!("plan{}", i))
            .bind(format!("user{}", i % 100))
            .bind(&start_date)
            .bind("active")
            .bind("2025-10-25T00:00:00Z")
            .execute(&pool)
            .await
            .unwrap();
    }

    // Measure migration execution time
    let start = Instant::now();
    let migration_sql = include_str!("../migrations/06_v0.8.sql");
    sqlx::raw_sql(migration_sql).execute(&pool).await.unwrap();
    let duration = start.elapsed();

    println!("Migration 06_v0.8.sql execution time: {:?}", duration);
    assert!(
        duration.as_secs() < 5,
        "Migration should complete in <5 seconds, took {:?}",
        duration
    );
}

/// Test rollback migration successfully reverses all changes
#[tokio::test]
async fn test_rollback_migration() {
    let pool = common::setup_test_db().await.0;

    // Verify schema exists after forward migration
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='meal_plan_rotation_state'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        result.0, 1,
        "meal_plan_rotation_state should exist before rollback"
    );

    // Run rollback migration
    let rollback_sql = include_str!("06_v0.8_rollback.sql");
    sqlx::raw_sql(rollback_sql).execute(&pool).await.unwrap();

    // Verify meal_plan_rotation_state table dropped
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='meal_plan_rotation_state'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        result.0, 0,
        "meal_plan_rotation_state should be dropped after rollback"
    );

    // Verify triggers dropped
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name IN ('prevent_locked_week_modification', 'update_meal_plan_status')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(result.0, 0, "Triggers should be dropped after rollback");

    // Verify columns dropped (check meal_plans)
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pragma_table_info('meal_plans') WHERE name IN ('end_date', 'is_locked', 'generation_batch_id')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        result.0, 0,
        "meal_plans new columns should be dropped after rollback"
    );

    // Verify columns dropped (check recipes - only 3 new ones, cuisine/dietary_tags existed before)
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pragma_table_info('recipes') WHERE name IN ('accepts_accompaniment', 'preferred_accompaniments', 'accompaniment_category')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        result.0, 0,
        "recipes new columns should be dropped after rollback"
    );

    // Verify cuisine and dietary_tags still exist (they existed before this migration)
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pragma_table_info('recipes') WHERE name IN ('cuisine', 'dietary_tags')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        result.0, 2,
        "cuisine and dietary_tags should still exist after rollback (pre-existing columns)"
    );

    // Verify columns dropped (check users)
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name IN ('max_prep_time_weeknight', 'max_prep_time_weekend', 'avoid_consecutive_complex', 'cuisine_variety_weight')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        result.0, 0,
        "users new columns should be dropped after rollback"
    );
}

/// Test idempotence: forward → rollback → forward again
#[tokio::test]
async fn test_migration_idempotence() {
    let pool = imkitchen::db::create_pool(":memory:", 1).await.unwrap();

    // Run all migrations including 06_v0.8.sql
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    // Run rollback
    let rollback_sql = include_str!("06_v0.8_rollback.sql");
    sqlx::raw_sql(rollback_sql).execute(&pool).await.unwrap();

    // Run forward migration again
    let forward_sql = include_str!("../migrations/06_v0.8.sql");
    let result = sqlx::raw_sql(forward_sql).execute(&pool).await;

    assert!(
        result.is_ok(),
        "Re-running forward migration after rollback should succeed"
    );

    // Verify schema recreated correctly
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='meal_plan_rotation_state'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        result.0, 1,
        "meal_plan_rotation_state should exist after re-applying migration"
    );
}
