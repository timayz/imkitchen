use chrono::{DateTime, Datelike, Duration, Timelike, Utc};

mod common;

// Helper function to create a test user
async fn create_test_user(pool: &sqlx::SqlitePool, user_id: &str) {
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, tier, recipe_count, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(format!("{}@example.com", user_id))
    .bind("hashed_password")
    .bind("free")
    .bind(0)
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .unwrap();
}

/// AC #1: Morning reminders sent at 9:00 AM local time
/// AC #6: Only sent if advance prep required within next 24 hours
#[tokio::test]
async fn test_morning_reminder_scheduled_at_9am_for_tomorrows_meal() {
    let (pool, executor) = common::setup_test_db().await;

    // Given: A meal plan for tomorrow with a recipe requiring 12h advance prep
    let user_id = "test-user-001";
    let recipe_id = "recipe-chicken-tikka";

    let tomorrow = (Utc::now() + Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    create_test_user(&pool, user_id).await;

    // Create recipe with 12h advance prep
    sqlx::query(
        "INSERT INTO recipes (id, user_id, title, ingredients, instructions, prep_time_min, cook_time_min, advance_prep_hours, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(recipe_id)
    .bind(user_id)
    .bind("Chicken Tikka Masala")
    .bind(r#"[{"name": "chicken", "quantity": 500, "unit": "g"}]"#)
    .bind(r#"["Marinate chicken overnight"]"#)
    .bind(20)
    .bind(30)
    .bind(12) // 12 hours advance prep
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    // Create meal plan and meal assignment for tomorrow's dinner
    let meal_plan_id = "meal-plan-001";
    sqlx::query(
        "INSERT INTO meal_plans (id, user_id, start_date, status, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(meal_plan_id)
    .bind(user_id)
    .bind(&tomorrow)
    .bind("active")
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO meal_assignments (id, meal_plan_id, date, meal_type, recipe_id, prep_required)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind("assignment-001")
    .bind(meal_plan_id)
    .bind(&tomorrow)
    .bind("dinner")
    .bind(recipe_id)
    .bind(1)
    .execute(&pool)
    .await
    .unwrap();

    // When: Run morning_reminder_scheduler for this user
    notifications::morning_reminder_scheduler(&pool, &executor, user_id)
        .await
        .unwrap();

    // Process evento events synchronously to project ReminderScheduled to notifications table
    notifications::notification_projections(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Update message bodies after projection
    notifications::update_morning_reminder_messages(&pool, user_id)
        .await
        .unwrap();

    // Then: A ReminderScheduled event should be emitted with scheduled_time = 9:00 AM TODAY
    // Query the notifications table (projection from ReminderScheduled event)
    let notification: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT id, scheduled_time, reminder_type, message_body
         FROM notifications
         WHERE user_id = ? AND recipe_id = ? AND status = 'pending'",
    )
    .bind(user_id)
    .bind(recipe_id)
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert!(
        notification.is_some(),
        "Morning reminder should be scheduled"
    );
    let (notification_id, scheduled_time, reminder_type, _message_body) = notification.unwrap();

    // Verify reminder type is "morning"
    assert_eq!(reminder_type, "morning");

    // Verify scheduled time is 9:00 AM today (UTC)
    let scheduled_dt = DateTime::parse_from_rfc3339(&scheduled_time).unwrap();
    let today = Utc::now().date_naive();
    assert_eq!(
        scheduled_dt.naive_utc().date(),
        today,
        "Should be scheduled for today"
    );
    assert_eq!(scheduled_dt.hour(), 9, "Should be scheduled at 9:00 AM");
    assert_eq!(
        scheduled_dt.minute(),
        0,
        "Should be scheduled at exactly 9:00"
    );

    // Verify notification ID is not empty
    assert!(!notification_id.is_empty());
}

/// AC #2 & #3: Reminder content format
/// Expected: "Prep reminder: {task} tonight for {day_of_week}'s {meal_type}"
/// Example: "Prep reminder: Marinate chicken tonight for Thursday's dinner"
#[tokio::test]
async fn test_morning_reminder_message_format() {
    let (pool, executor) = common::setup_test_db().await;

    // Given: Recipe with prep_task="Marinate chicken" scheduled for Thursday
    let user_id = "test-user-002";
    let recipe_id = "recipe-thursday-dinner";

    create_test_user(&pool, user_id).await;

    // Use tomorrow and calculate its day of week for the test
    let tomorrow = (Utc::now() + Duration::days(1)).date_naive();
    let tomorrow_str = tomorrow.format("%Y-%m-%d").to_string();

    // Create recipe
    sqlx::query(
        "INSERT INTO recipes (id, user_id, title, ingredients, instructions, prep_time_min, cook_time_min, advance_prep_hours, prep_task, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(recipe_id)
    .bind(user_id)
    .bind("Chicken Tikka Masala")
    .bind(r#"[{"name": "chicken", "quantity": 500, "unit": "g"}]"#)
    .bind(r#"["Marinate chicken"]"#)
    .bind(10) // 10 minutes active prep
    .bind(30)
    .bind(12) // 12h advance prep
    .bind("Marinate chicken") // prep_task
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    // Create meal plan and meal assignment for tomorrow's dinner
    let meal_plan_id = "meal-plan-002";
    sqlx::query(
        "INSERT INTO meal_plans (id, user_id, start_date, status, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(meal_plan_id)
    .bind(user_id)
    .bind(&tomorrow_str)
    .bind("active")
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO meal_assignments (id, meal_plan_id, date, meal_type, recipe_id, prep_required)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind("assignment-002")
    .bind(meal_plan_id)
    .bind(&tomorrow_str)
    .bind("dinner")
    .bind(recipe_id)
    .bind(1)
    .execute(&pool)
    .await
    .unwrap();

    // When: Run morning_reminder_scheduler
    notifications::morning_reminder_scheduler(&pool, &executor, user_id)
        .await
        .unwrap();

    // Process evento events synchronously
    notifications::notification_projections(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Update message bodies after projection
    notifications::update_morning_reminder_messages(&pool, user_id)
        .await
        .unwrap();

    // Then: Message should match format: "Prep reminder: Marinate chicken tonight for Thursday's dinner"
    let notification: Option<String> = sqlx::query_scalar(
        "SELECT message_body FROM notifications WHERE user_id = ? AND recipe_id = ? AND status = 'pending'"
    )
    .bind(user_id)
    .bind(recipe_id)
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert!(notification.is_some(), "Notification should exist");
    let message = notification.unwrap();
    eprintln!("DEBUG: message_body = '{}'", message);

    // Get day of week for tomorrow
    let day_of_week = match tomorrow.weekday() {
        chrono::Weekday::Mon => "Monday",
        chrono::Weekday::Tue => "Tuesday",
        chrono::Weekday::Wed => "Wednesday",
        chrono::Weekday::Thu => "Thursday",
        chrono::Weekday::Fri => "Friday",
        chrono::Weekday::Sat => "Saturday",
        chrono::Weekday::Sun => "Sunday",
    };

    assert!(
        message.contains("Prep reminder:"),
        "Should start with 'Prep reminder:'"
    );
    assert!(
        message.contains("Marinate chicken"),
        "Should mention prep task"
    );
    assert!(message.contains("tonight"), "Should mention 'tonight'");
    assert!(
        message.contains(day_of_week),
        "Should mention day of week ({})",
        day_of_week
    );
    assert!(message.contains("dinner"), "Should mention meal type");
}

/// AC #4: Reminder includes estimated prep time
#[tokio::test]
async fn test_morning_reminder_includes_prep_time() {
    let (pool, executor) = common::setup_test_db().await;

    let user_id = "test-user-003";
    let recipe_id = "recipe-with-prep-time";

    let tomorrow = (Utc::now() + Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    create_test_user(&pool, user_id).await;

    // Create recipe with specific prep_time_min
    sqlx::query(
        "INSERT INTO recipes (id, user_id, title, ingredients, instructions, prep_time_min, cook_time_min, advance_prep_hours, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(recipe_id)
    .bind(user_id)
    .bind("Quick Marinade")
    .bind(r#"[]"#)
    .bind(r#"[]"#)
    .bind(10) // 10 minutes prep time
    .bind(20)
    .bind(8) // 8h advance prep
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    // Create meal plan and meal assignment
    let meal_plan_id = "meal-plan-003";
    sqlx::query(
        "INSERT INTO meal_plans (id, user_id, start_date, status, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(meal_plan_id)
    .bind(user_id)
    .bind(&tomorrow)
    .bind("active")
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO meal_assignments (id, meal_plan_id, date, meal_type, recipe_id, prep_required)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind("assignment-003")
    .bind(meal_plan_id)
    .bind(&tomorrow)
    .bind("dinner")
    .bind(recipe_id)
    .bind(1)
    .execute(&pool)
    .await
    .unwrap();

    // When: Run morning_reminder_scheduler
    notifications::morning_reminder_scheduler(&pool, &executor, user_id)
        .await
        .unwrap();

    // Process evento events synchronously
    notifications::notification_projections(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Update message bodies after projection
    notifications::update_morning_reminder_messages(&pool, user_id)
        .await
        .unwrap();

    // Then: Message should include prep time estimate
    let message: Option<String> = sqlx::query_scalar(
        "SELECT message_body FROM notifications WHERE user_id = ? AND recipe_id = ?",
    )
    .bind(user_id)
    .bind(recipe_id)
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert!(message.is_some());
    let msg = message.unwrap();
    assert!(
        msg.contains("10 minutes") || msg.contains("Takes 10"),
        "Should mention prep time estimate"
    );
}

/// AC #6: No morning reminder for recipes with prep_hours > 24
#[tokio::test]
async fn test_no_morning_reminder_for_24h_plus_prep() {
    let (pool, executor) = common::setup_test_db().await;

    let user_id = "test-user-004";
    let recipe_id = "recipe-long-prep";

    let tomorrow = (Utc::now() + Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    create_test_user(&pool, user_id).await;

    // Create recipe with 30h advance prep (should use advance_prep reminder, not morning)
    sqlx::query(
        "INSERT INTO recipes (id, user_id, title, ingredients, instructions, prep_time_min, cook_time_min, advance_prep_hours, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(recipe_id)
    .bind(user_id)
    .bind("Long Marinade Recipe")
    .bind(r#"[]"#)
    .bind(r#"[]"#)
    .bind(15)
    .bind(45)
    .bind(30) // 30 hours = exceeds 24h threshold
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    // Create meal plan and meal assignment
    let meal_plan_id = "meal-plan-004";
    sqlx::query(
        "INSERT INTO meal_plans (id, user_id, start_date, status, created_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(meal_plan_id)
    .bind(user_id)
    .bind(&tomorrow)
    .bind("active")
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO meal_assignments (id, meal_plan_id, date, meal_type, recipe_id, prep_required)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind("assignment-004")
    .bind(meal_plan_id)
    .bind(&tomorrow)
    .bind("dinner")
    .bind(recipe_id)
    .bind(1)
    .execute(&pool)
    .await
    .unwrap();

    // When: Run morning_reminder_scheduler
    notifications::morning_reminder_scheduler(&pool, &executor, user_id)
        .await
        .unwrap();

    // Then: NO morning reminder should be scheduled (30h prep uses advance_prep reminder instead)
    let notification_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = ? AND recipe_id = ? AND reminder_type = 'morning'"
    )
    .bind(user_id)
    .bind(recipe_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        notification_count, 0,
        "Should NOT create morning reminder for 24h+ prep"
    );
}

/// AC #8: Expired reminders auto-dismissed after prep window
#[tokio::test]
async fn test_expired_reminders_auto_dismissed() {
    let (pool, executor) = common::setup_test_db().await;

    let user_id = "test-user-005";
    let notification_id = "notification-expired";

    // Given: A morning reminder scheduled for yesterday 9am with 12h prep window (expired)
    let yesterday_9am = (Utc::now() - Duration::days(1))
        .date_naive()
        .and_hms_opt(9, 0, 0)
        .unwrap()
        .and_utc()
        .to_rfc3339();

    sqlx::query(
        "INSERT INTO notifications (id, user_id, recipe_id, meal_date, scheduled_time, reminder_type, prep_hours, status, message_body, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(notification_id)
    .bind(user_id)
    .bind("recipe-test")
    .bind((Utc::now()).format("%Y-%m-%d").to_string()) // Meal was today
    .bind(&yesterday_9am)
    .bind("morning")
    .bind(12)
    .bind("pending")
    .bind("Test reminder")
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    // When: Run auto_dismissal_worker
    notifications::auto_dismissal_worker(&pool, &executor)
        .await
        .unwrap();

    // Then: Reminder should be marked as dismissed (status changed from 'pending' to 'dismissed')
    let status: Option<String> =
        sqlx::query_scalar("SELECT status FROM notifications WHERE id = ?")
            .bind(notification_id)
            .fetch_optional(&pool)
            .await
            .unwrap();

    assert_eq!(
        status,
        Some("dismissed".to_string()),
        "Expired reminder should be auto-dismissed"
    );
}

/// Edge case: No morning reminder if meal plan doesn't exist for tomorrow
#[tokio::test]
async fn test_no_morning_reminder_without_tomorrows_meal() {
    let (pool, executor) = common::setup_test_db().await;

    let user_id = "test-user-006";

    // No meal plans created for this user

    // When: Run morning_reminder_scheduler
    notifications::morning_reminder_scheduler(&pool, &executor, user_id)
        .await
        .unwrap();

    // Then: No reminders should be created
    let notification_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(
        notification_count, 0,
        "Should not create reminders without meal plans"
    );
}
