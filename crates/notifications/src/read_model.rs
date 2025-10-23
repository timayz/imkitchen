use crate::aggregate::{NotificationAggregate, PushSubscriptionAggregate};
use crate::events::{
    PrepTaskCompleted, PushSubscriptionCreated, ReminderDismissed, ReminderScheduled, ReminderSent,
    ReminderSnoozed,
};
use evento::{AggregatorName, Context, EventDetails, Executor};
use serde::{Deserialize, Serialize};

/// Project ReminderScheduled event to notifications table
///
/// This evento subscription handler inserts a new row into the notifications table
/// when a ReminderScheduled event is emitted.
#[evento::handler(NotificationAggregate)]
pub async fn project_reminder_scheduled<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ReminderScheduled>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: sqlx::SqlitePool = context.extract();
    let notification_id = &event.aggregator_id;

    // Insert into notifications table
    sqlx::query(
        r#"
        INSERT INTO notifications (
            id, user_id, recipe_id, meal_date, scheduled_time, status,
            reminder_type, prep_hours, prep_task
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(notification_id)
    .bind(&event.data.user_id)
    .bind(&event.data.recipe_id)
    .bind(&event.data.meal_date)
    .bind(&event.data.scheduled_time)
    .bind("pending") // Initial status
    .bind(&event.data.reminder_type)
    .bind(event.data.prep_hours)
    .bind(&event.data.prep_task)
    .execute(&pool)
    .await?;

    tracing::info!(
        "Projected ReminderScheduled event for notification_id={}, scheduled_time={}",
        notification_id,
        event.data.scheduled_time
    );

    Ok(())
}

/// Project ReminderSent event to notifications table
///
/// This evento subscription handler updates the sent_at, delivery_status, and status columns
/// when a ReminderSent event is emitted by the background worker.
#[evento::handler(NotificationAggregate)]
pub async fn project_reminder_sent<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ReminderSent>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: sqlx::SqlitePool = context.extract();
    let notification_id = &event.aggregator_id;

    // Determine status based on delivery result
    let status = if event.data.delivery_status == "sent" {
        "sent"
    } else {
        "failed"
    };

    // Update notifications table
    sqlx::query(
        r#"
        UPDATE notifications
        SET sent_at = ?, delivery_status = ?, status = ?
        WHERE id = ?
        "#,
    )
    .bind(&event.data.sent_at)
    .bind(&event.data.delivery_status)
    .bind(status)
    .bind(notification_id)
    .execute(&pool)
    .await?;

    tracing::info!(
        "Projected ReminderSent event for notification_id={}, delivery_status={}",
        notification_id,
        event.data.delivery_status
    );

    Ok(())
}

/// Project ReminderDismissed event to notifications table
///
/// This evento subscription handler updates the dismissed_at and status columns
/// when a ReminderDismissed event is emitted (user clicks "Dismiss" button).
#[evento::handler(NotificationAggregate)]
pub async fn project_reminder_dismissed<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ReminderDismissed>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: sqlx::SqlitePool = context.extract();
    let notification_id = &event.aggregator_id;

    // Update notifications table
    sqlx::query(
        r#"
        UPDATE notifications
        SET dismissed_at = ?, status = 'dismissed'
        WHERE id = ?
        "#,
    )
    .bind(&event.data.dismissed_at)
    .bind(notification_id)
    .execute(&pool)
    .await?;

    tracing::info!(
        "Projected ReminderDismissed event for notification_id={}",
        notification_id
    );

    Ok(())
}

/// Project ReminderSnoozed event to notifications table
///
/// This evento subscription handler updates the scheduled_time, snoozed_until, and status
/// when a ReminderSnoozed event is emitted (user clicks "Snooze" button).
#[evento::handler(NotificationAggregate)]
pub async fn project_reminder_snoozed<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ReminderSnoozed>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: sqlx::SqlitePool = context.extract();
    let notification_id = &event.aggregator_id;

    // Update notifications table
    sqlx::query(
        r#"
        UPDATE notifications
        SET scheduled_time = ?, snoozed_until = ?, status = 'snoozed'
        WHERE id = ?
        "#,
    )
    .bind(&event.data.snoozed_until)
    .bind(&event.data.snoozed_until)
    .bind(notification_id)
    .execute(&pool)
    .await?;

    tracing::info!(
        "Projected ReminderSnoozed event for notification_id={}, new_scheduled_time={}",
        notification_id,
        event.data.snoozed_until
    );

    Ok(())
}

/// Project PrepTaskCompleted event to notifications table
///
/// This evento subscription handler updates the completed_at and status columns
/// when a PrepTaskCompleted event is emitted (user clicks "Mark Complete" button).
/// AC #2, #5, #6
#[evento::handler(NotificationAggregate)]
pub async fn project_prep_task_completed<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<PrepTaskCompleted>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: sqlx::SqlitePool = context.extract();
    let notification_id = &event.aggregator_id;

    // Update notifications table
    sqlx::query(
        r#"
        UPDATE notifications
        SET completed_at = ?, status = 'completed'
        WHERE id = ?
        "#,
    )
    .bind(&event.data.completed_at)
    .bind(notification_id)
    .execute(&pool)
    .await?;

    tracing::info!(
        "Projected PrepTaskCompleted event for notification_id={}, recipe_id={}",
        notification_id,
        event.data.recipe_id
    );

    Ok(())
}

/// Project PushSubscriptionCreated event to push_subscriptions table
///
/// This evento subscription handler inserts a new row into the push_subscriptions table
/// when a PushSubscriptionCreated event is emitted.
#[evento::handler(PushSubscriptionAggregate)]
pub async fn project_push_subscription_created<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<PushSubscriptionCreated>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: sqlx::SqlitePool = context.extract();
    let subscription_id = &event.aggregator_id;

    // Insert or replace subscription (idempotent - handles re-subscription with same endpoint)
    sqlx::query(
        r#"
        INSERT INTO push_subscriptions (id, user_id, endpoint, p256dh_key, auth_key, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(endpoint) DO UPDATE SET
            id = excluded.id,
            user_id = excluded.user_id,
            p256dh_key = excluded.p256dh_key,
            auth_key = excluded.auth_key,
            created_at = excluded.created_at
        "#,
    )
    .bind(subscription_id)
    .bind(&event.data.user_id)
    .bind(&event.data.endpoint)
    .bind(&event.data.p256dh_key)
    .bind(&event.data.auth_key)
    .bind(&event.data.created_at)
    .execute(&pool)
    .await?;

    tracing::info!(
        "Projected PushSubscriptionCreated event for subscription_id={}, user_id={}",
        subscription_id,
        event.data.user_id
    );

    Ok(())
}

/// Query to get pending notifications that are due (scheduled_time <= now)
///
/// This is called by the background worker to find notifications that need to be sent.
pub async fn get_pending_notifications_due(
    pool: &sqlx::SqlitePool,
) -> anyhow::Result<Vec<PendingNotification>> {
    let now = chrono::Utc::now().to_rfc3339();

    let notifications = sqlx::query_as::<_, PendingNotification>(
        r#"
        SELECT id, user_id, recipe_id, meal_date, scheduled_time, reminder_type, prep_hours, prep_task
        FROM notifications
        WHERE status = 'pending' AND scheduled_time <= ?
        ORDER BY scheduled_time ASC
        "#,
    )
    .bind(&now)
    .fetch_all(pool)
    .await?;

    Ok(notifications)
}

/// Query to get user's push subscription
///
/// This is called by the background worker to get the push endpoint for notification delivery.
pub async fn get_push_subscription_by_user(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> anyhow::Result<Option<PushSubscription>> {
    let subscription = sqlx::query_as::<_, PushSubscription>(
        r#"
        SELECT id, user_id, endpoint, p256dh_key, auth_key, created_at
        FROM push_subscriptions
        WHERE user_id = ?
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(subscription)
}

/// Query to get user's pending notifications (for UI display)
pub async fn get_user_pending_notifications(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> anyhow::Result<Vec<UserNotification>> {
    let notifications = sqlx::query_as::<_, UserNotification>(
        r#"
        SELECT id, user_id, recipe_id, meal_date, scheduled_time, reminder_type, prep_hours, prep_task, status, message_body
        FROM notifications
        WHERE user_id = ? AND status IN ('pending', 'sent')
        ORDER BY scheduled_time ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(notifications)
}

/// Query to get user's prep tasks for today
///
/// This is used for the dashboard "Prep Tasks for Today" section.
/// AC #3, #4: Shows both pending and completed prep tasks for today's meals.
pub async fn get_user_prep_tasks_for_today(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> anyhow::Result<Vec<UserNotification>> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let notifications = sqlx::query_as::<_, UserNotification>(
        r#"
        SELECT id, user_id, recipe_id, meal_date, scheduled_time, reminder_type, prep_hours, prep_task, status, message_body
        FROM notifications
        WHERE user_id = ?
          AND reminder_type IN ('advance_prep', 'morning')
          AND meal_date = ?
        ORDER BY status ASC, scheduled_time ASC
        "#,
    )
    .bind(user_id)
    .bind(&today)
    .fetch_all(pool)
    .await?;

    Ok(notifications)
}

/// Query to get a single notification by ID (for ownership validation)
pub async fn get_notification_by_id(
    pool: &sqlx::SqlitePool,
    notification_id: &str,
) -> anyhow::Result<Option<UserNotification>> {
    let notification = sqlx::query_as::<_, UserNotification>(
        r#"
        SELECT id, user_id, recipe_id, meal_date, scheduled_time, reminder_type, prep_hours, prep_task, status, message_body
        FROM notifications
        WHERE id = ?
        "#,
    )
    .bind(notification_id)
    .fetch_optional(pool)
    .await?;

    Ok(notification)
}

/// Query to get prep status for a specific recipe (Story 4.9 AC #8)
///
/// Returns the most recent prep task notification for the recipe.
/// Used on recipe detail page to show prep completion status.
pub async fn get_prep_status_for_recipe(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    recipe_id: &str,
) -> anyhow::Result<Option<UserNotification>> {
    let notification = sqlx::query_as::<_, UserNotification>(
        r#"
        SELECT id, user_id, recipe_id, meal_date, scheduled_time, reminder_type, prep_hours, prep_task, status, message_body
        FROM notifications
        WHERE user_id = ?
          AND recipe_id = ?
          AND reminder_type IN ('advance_prep', 'morning')
        ORDER BY scheduled_time DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(recipe_id)
    .fetch_optional(pool)
    .await?;

    Ok(notification)
}

/// Pending notification DTO (for background worker)
#[derive(Debug, sqlx::FromRow)]
pub struct PendingNotification {
    pub id: String,
    pub user_id: String,
    pub recipe_id: String,
    pub meal_date: String,
    pub scheduled_time: String,
    pub reminder_type: String,
    pub prep_hours: i32,
    pub prep_task: Option<String>,
}

/// Push subscription DTO
#[derive(Debug, sqlx::FromRow)]
pub struct PushSubscription {
    pub id: String,
    pub user_id: String,
    pub endpoint: String,
    pub p256dh_key: String,
    pub auth_key: String,
    pub created_at: String,
}

/// User notification DTO (for UI display)
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserNotification {
    pub id: String,
    pub user_id: String,
    pub recipe_id: String,
    pub meal_date: String,
    pub scheduled_time: String,
    pub reminder_type: String,
    pub prep_hours: i32,
    pub prep_task: Option<String>,
    pub status: String,
    pub message_body: Option<String>,
}

/// Get push subscription count for user (AC #7)
///
/// Returns the number of active push subscriptions the user has (multiple devices)
pub async fn get_push_subscription_count(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> anyhow::Result<i64> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM push_subscriptions WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok(count.0)
}

/// Push subscription status for settings page (AC #7)
#[derive(Debug, Serialize, Deserialize)]
pub struct PushSubscriptionStatus {
    pub enabled: bool,
    pub subscription_count: i64,
}

/// Get push subscription status for user (AC #7)
///
/// Returns whether push notifications are enabled and count of active subscriptions
pub async fn get_push_subscription_status(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> anyhow::Result<PushSubscriptionStatus> {
    let count = get_push_subscription_count(pool, user_id).await?;

    Ok(PushSubscriptionStatus {
        enabled: count > 0,
        subscription_count: count,
    })
}

/// Create subscription builder for all notification projections
///
/// This sets up all read model projections for the notification domain.
pub fn notification_projections(
    pool: sqlx::SqlitePool,
) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("notification-projections")
        .aggregator::<NotificationAggregate>()
        .aggregator::<PushSubscriptionAggregate>()
        .data(pool)
        .handler(project_reminder_scheduled())
        .handler(project_reminder_sent())
        .handler(project_reminder_dismissed())
        .handler(project_reminder_snoozed())
        .handler(project_prep_task_completed())
        .handler(project_push_subscription_created())
}
