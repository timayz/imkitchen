//! Access control service tests
//!
//! Tests verify that the AccessControlService correctly enforces freemium restrictions
//! with proper priority: global bypass > per-user bypass > premium > free tier

mod helpers;

use helpers::{
    cleanup_test_databases, create_test_config, create_test_config_with_bypass,
    setup_test_databases,
};
use imkitchen::access_control::AccessControlService;
use imkitchen::queries::user::subscribe_user_query;
use imkitchen_user::command::{subscribe_user_command, Command};
use imkitchen_user::event::EventMetadata;
use sqlx::SqlitePool;
use ulid::Ulid;

/// Helper to create a test user with profile
async fn create_test_user(
    comando: &Command<evento::Sqlite>,
    evento: &evento::Sqlite,
    validation_pool: &SqlitePool,
    query_pool: &SqlitePool,
    email: &str,
    premium_bypass: bool,
    is_premium_active: bool,
) -> anyhow::Result<String> {
    let metadata = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    // Register user
    let input = imkitchen_user::command::RegisterUserInput {
        email: email.to_string(),
        password: "Password123".to_string(), // Meets validation: uppercase, lowercase, number
        is_admin: Some(false),
    };
    let user_id = comando.register_user(input, metadata.clone()).await?;

    // Process events synchronously
    subscribe_user_command(validation_pool.clone())
        .unsafe_oneshot(evento)
        .await?;
    subscribe_user_query(query_pool.clone())
        .unsafe_oneshot(evento)
        .await?;

    // Update profile with premium settings
    sqlx::query(
        "INSERT INTO user_profiles (user_id, is_premium_active, premium_bypass)
         VALUES (?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET
            is_premium_active = excluded.is_premium_active,
            premium_bypass = excluded.premium_bypass",
    )
    .bind(&user_id)
    .bind(is_premium_active)
    .bind(premium_bypass)
    .execute(query_pool)
    .await?;

    Ok(user_id)
}

#[tokio::test]
async fn test_global_bypass_allows_all_access() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;
    let config = create_test_config_with_bypass(); // Global bypass enabled

    // Create access control service with global bypass
    let service = AccessControlService::new(config, dbs.queries.clone());

    // Create a free tier user (no premium, no bypass)
    let comando = Command::new(dbs.evento().clone());
    let user_id = create_test_user(
        &comando,
        dbs.evento(),
        &dbs.validation,
        &dbs.queries,
        "test@example.com",
        false,
        false,
    )
    .await?;

    // Global bypass should allow access to all weeks
    for week in 1..=5 {
        let can_view = service.can_view_week(&user_id, week).await?;
        assert!(
            can_view,
            "Global bypass should allow access to week {}",
            week
        );

        let can_shop = service.can_access_shopping_list(&user_id, week).await?;
        assert!(
            can_shop,
            "Global bypass should allow shopping list for week {}",
            week
        );
    }

    // Global bypass should allow unlimited favorites
    let can_add = service.can_add_favorite(&user_id).await?;
    assert!(can_add, "Global bypass should allow adding favorites");

    cleanup_test_databases(dbs).await?;
    Ok(())
}

#[tokio::test]
async fn test_per_user_bypass_allows_all_access() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;
    let config = create_test_config(); // Global bypass disabled

    let service = AccessControlService::new(config, dbs.queries.clone());

    // Create user with per-user bypass (not premium)
    let comando = Command::new(dbs.evento().clone());
    let user_id = create_test_user(
        &comando,
        dbs.evento(),
        &dbs.validation,
        &dbs.queries,
        "bypass@example.com",
        true,
        false,
    )
    .await?;

    // Per-user bypass should allow access to all weeks
    for week in 1..=5 {
        let can_view = service.can_view_week(&user_id, week).await?;
        assert!(
            can_view,
            "Per-user bypass should allow access to week {}",
            week
        );

        let can_shop = service.can_access_shopping_list(&user_id, week).await?;
        assert!(
            can_shop,
            "Per-user bypass should allow shopping list for week {}",
            week
        );
    }

    // Per-user bypass should allow unlimited favorites
    let can_add = service.can_add_favorite(&user_id).await?;
    assert!(can_add, "Per-user bypass should allow adding favorites");

    cleanup_test_databases(dbs).await?;
    Ok(())
}

#[tokio::test]
async fn test_premium_user_has_full_access() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;
    let config = create_test_config(); // Global bypass disabled

    let service = AccessControlService::new(config, dbs.queries.clone());

    // Create premium user (no bypass)
    let comando = Command::new(dbs.evento().clone());
    let user_id = create_test_user(
        &comando,
        dbs.evento(),
        &dbs.validation,
        &dbs.queries,
        "premium@example.com",
        false,
        true,
    )
    .await?;

    // Premium should allow access to all weeks
    for week in 1..=5 {
        let can_view = service.can_view_week(&user_id, week).await?;
        assert!(can_view, "Premium user should access week {}", week);

        let can_shop = service.can_access_shopping_list(&user_id, week).await?;
        assert!(
            can_shop,
            "Premium user should access shopping list for week {}",
            week
        );
    }

    // Premium should allow unlimited favorites
    let can_add = service.can_add_favorite(&user_id).await?;
    assert!(can_add, "Premium user should add unlimited favorites");

    cleanup_test_databases(dbs).await?;
    Ok(())
}

#[tokio::test]
async fn test_free_tier_restricted_to_week_1() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;
    let config = create_test_config(); // Global bypass disabled

    let service = AccessControlService::new(config, dbs.queries.clone());

    // Create free tier user
    let comando = Command::new(dbs.evento().clone());
    let user_id = create_test_user(
        &comando,
        dbs.evento(),
        &dbs.validation,
        &dbs.queries,
        "free@example.com",
        false,
        false,
    )
    .await?;

    // Week 1 should be accessible
    assert!(
        service.can_view_week(&user_id, 1).await?,
        "Free tier should access week 1"
    );
    assert!(
        service.can_access_shopping_list(&user_id, 1).await?,
        "Free tier should access week 1 shopping list"
    );

    // Weeks 2-5 should be restricted
    for week in 2..=5 {
        assert!(
            !service.can_view_week(&user_id, week).await?,
            "Free tier should NOT access week {}",
            week
        );
        assert!(
            !service.can_access_shopping_list(&user_id, week).await?,
            "Free tier should NOT access shopping list for week {}",
            week
        );
    }

    cleanup_test_databases(dbs).await?;
    Ok(())
}

#[tokio::test]
async fn test_free_tier_favorite_limit() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;
    let config = create_test_config();

    let service = AccessControlService::new(config, dbs.queries.clone());

    // Create free tier user
    let comando = Command::new(dbs.evento().clone());
    let user_id = create_test_user(
        &comando,
        dbs.evento(),
        &dbs.validation,
        &dbs.queries,
        "freeuser@example.com",
        false,
        false,
    )
    .await?;

    // Free tier should allow adding favorites (count_user_favorites returns 0 as placeholder)
    // NOTE: This test validates the logic, actual favorite counting will work in Story 2.3
    let can_add = service.can_add_favorite(&user_id).await?;
    assert!(
        can_add,
        "Free tier with 0 favorites should be able to add (under 10 limit)"
    );

    cleanup_test_databases(dbs).await?;
    Ok(())
}

#[tokio::test]
async fn test_bypass_priority_order() -> anyhow::Result<()> {
    // Test 1: Global bypass overrides everything
    {
        let dbs = setup_test_databases().await?;
        let config = create_test_config_with_bypass();
        let service = AccessControlService::new(config, dbs.queries.clone());
        let comando = Command::new(dbs.evento().clone());

        // Free tier user, but global bypass enabled
        let user_id = create_test_user(
            &comando,
            dbs.evento(),
            &dbs.validation,
            &dbs.queries,
            "test1@example.com",
            false,
            false,
        )
        .await?;

        assert!(
            service.can_view_week(&user_id, 5).await?,
            "Global bypass should override free tier restriction"
        );
        cleanup_test_databases(dbs).await?;
    }

    // Test 2: Per-user bypass overrides free tier
    {
        let dbs = setup_test_databases().await?;
        let config = create_test_config();
        let service = AccessControlService::new(config, dbs.queries.clone());
        let comando = Command::new(dbs.evento().clone());

        // Non-premium user with per-user bypass
        let user_id = create_test_user(
            &comando,
            dbs.evento(),
            &dbs.validation,
            &dbs.queries,
            "test2@example.com",
            true,
            false,
        )
        .await?;

        assert!(
            service.can_view_week(&user_id, 5).await?,
            "Per-user bypass should override free tier restriction"
        );
        cleanup_test_databases(dbs).await?;
    }

    // Test 3: Premium overrides free tier
    {
        let dbs = setup_test_databases().await?;
        let config = create_test_config();
        let service = AccessControlService::new(config, dbs.queries.clone());
        let comando = Command::new(dbs.evento().clone());

        // Premium user without bypass
        let user_id = create_test_user(
            &comando,
            dbs.evento(),
            &dbs.validation,
            &dbs.queries,
            "test3@example.com",
            false,
            true,
        )
        .await?;

        assert!(
            service.can_view_week(&user_id, 5).await?,
            "Premium should override free tier restriction"
        );
        cleanup_test_databases(dbs).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_multiple_bypass_mechanisms_independently() -> anyhow::Result<()> {
    // User with both per-user bypass AND premium should work
    {
        let dbs = setup_test_databases().await?;
        let config = create_test_config();
        let service = AccessControlService::new(config, dbs.queries.clone());
        let comando = Command::new(dbs.evento().clone());

        let user_id = create_test_user(
            &comando,
            dbs.evento(),
            &dbs.validation,
            &dbs.queries,
            "both@example.com",
            true,
            true,
        )
        .await?;

        assert!(
            service.can_view_week(&user_id, 5).await?,
            "User with both bypass and premium should have access"
        );
        assert!(
            service.can_add_favorite(&user_id).await?,
            "User with both should add favorites"
        );
        cleanup_test_databases(dbs).await?;
    }

    // Each mechanism works independently
    {
        let dbs = setup_test_databases().await?;
        let config = create_test_config();
        let service = AccessControlService::new(config, dbs.queries.clone());
        let comando = Command::new(dbs.evento().clone());

        // Just bypass
        let bypass_user = create_test_user(
            &comando,
            dbs.evento(),
            &dbs.validation,
            &dbs.queries,
            "just_bypass@example.com",
            true,
            false,
        )
        .await?;
        assert!(service.can_view_week(&bypass_user, 3).await?);

        // Just premium
        let premium_user = create_test_user(
            &comando,
            dbs.evento(),
            &dbs.validation,
            &dbs.queries,
            "just_premium@example.com",
            false,
            true,
        )
        .await?;
        assert!(service.can_view_week(&premium_user, 4).await?);

        cleanup_test_databases(dbs).await?;
    }

    Ok(())
}
