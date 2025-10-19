use chrono::Utc;
use sqlx::SqlitePool;
use user::read_model::{
    can_prompt_for_notification_permission, query_user_notification_permission,
};

/// Helper to setup test database
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

/// Helper to setup test user with permission status
async fn setup_test_user(
    pool: &SqlitePool,
    user_id: &str,
    permission_status: &str,
    denial_timestamp: Option<String>,
) -> anyhow::Result<()> {
    // Insert test user
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, tier, recipe_count, created_at, notification_permission_status, last_permission_denial_at)
         VALUES (?, ?, ?, 'free', 0, ?, ?, ?)"
    )
    .bind(user_id)
    .bind("test@example.com")
    .bind("hash")
    .bind(Utc::now().to_rfc3339())
    .bind(permission_status)
    .bind(denial_timestamp)
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_permission_denied_records_timestamp() -> anyhow::Result<()> {
    // Setup
    let pool = setup_test_db().await;
    let user_id = "user-test-1";
    let denial_time = Utc::now().to_rfc3339();

    // AC #5, #8: Setup user with denied permission and timestamp
    setup_test_user(&pool, user_id, "denied", Some(denial_time.clone())).await?;

    // Verify permission status and timestamp can be queried
    let permission = query_user_notification_permission(user_id, &pool)
        .await?
        .expect("User should exist");

    assert_eq!(permission.permission_status, "denied");
    assert!(permission.last_permission_denial_at.is_some());
    assert_eq!(permission.last_permission_denial_at.unwrap(), denial_time);

    Ok(())
}

#[tokio::test]
async fn test_grace_period_prevents_re_prompt() -> anyhow::Result<()> {
    // Setup
    let pool = setup_test_db().await;
    let user_id = "user-test-2";
    let denial_time = Utc::now().to_rfc3339(); // Denied just now

    // AC #8: Setup user with recent denial (within grace period)
    setup_test_user(&pool, user_id, "denied", Some(denial_time)).await?;

    // AC #8: Grace period should prevent re-prompting within 30 days
    let can_prompt = can_prompt_for_notification_permission(user_id, &pool).await?;
    assert!(
        !can_prompt,
        "Should not be able to prompt within grace period"
    );

    Ok(())
}

#[tokio::test]
async fn test_permission_granted_allows_prompt() -> anyhow::Result<()> {
    // Setup
    let pool = setup_test_db().await;
    let user_id = "user-test-3";

    // Setup user with granted permission (no denial timestamp)
    setup_test_user(&pool, user_id, "granted", None).await?;

    // Verify permission status
    let permission = query_user_notification_permission(user_id, &pool)
        .await?
        .expect("User should exist");

    assert_eq!(permission.permission_status, "granted");
    assert!(permission.last_permission_denial_at.is_none());

    // Should still be able to prompt (for changing settings)
    let can_prompt = can_prompt_for_notification_permission(user_id, &pool).await?;
    assert!(can_prompt);

    Ok(())
}

#[tokio::test]
async fn test_permission_skipped() -> anyhow::Result<()> {
    // Setup
    let pool = setup_test_db().await;
    let user_id = "user-test-4";

    // AC #3: Setup user who skipped permission request
    setup_test_user(&pool, user_id, "skipped", None).await?;

    // Verify
    let permission = query_user_notification_permission(user_id, &pool)
        .await?
        .expect("User should exist");

    assert_eq!(permission.permission_status, "skipped");
    assert!(permission.last_permission_denial_at.is_none());

    // AC #8: Skipped status allows prompting later
    let can_prompt = can_prompt_for_notification_permission(user_id, &pool).await?;
    assert!(can_prompt, "Should be able to prompt after skipping");

    Ok(())
}
