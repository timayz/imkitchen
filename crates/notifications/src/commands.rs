use base64::Engine;
use chrono::Utc;
use uuid::Uuid;

use crate::events::{
    PushSubscriptionCreated, ReminderDismissed, ReminderScheduled, ReminderSent, ReminderSnoozed,
};

/// Command to schedule a preparation reminder
#[derive(Debug, Clone)]
pub struct ScheduleReminderCommand {
    pub user_id: String,
    pub recipe_id: String,
    pub meal_date: String,         // ISO 8601 date
    pub scheduled_time: String,    // RFC3339 timestamp
    pub reminder_type: String,     // "advance_prep", "morning", "day_of"
    pub prep_hours: i32,           // Hours of advance prep required
    pub prep_task: Option<String>, // "marinate", "rise", "chill", etc.
}

/// Command to send a reminder (triggered by background worker)
#[derive(Debug, Clone)]
pub struct SendReminderCommand {
    pub notification_id: String,
    pub delivery_status: String, // "sent", "failed", "endpoint_invalid"
}

/// Command to dismiss a reminder
#[derive(Debug, Clone)]
pub struct DismissReminderCommand {
    pub notification_id: String,
}

/// Command to snooze a reminder
#[derive(Debug, Clone)]
pub struct SnoozeReminderCommand {
    pub notification_id: String,
    pub snooze_duration_hours: i32, // Must be 1, 2, or 4
}

/// Command to subscribe to push notifications
#[derive(Debug, Clone)]
pub struct SubscribeToPushCommand {
    pub user_id: String,
    pub endpoint: String,
    pub p256dh_key: String,
    pub auth_key: String,
}

/// Error types for notification commands
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Invalid snooze duration: {0} hours. Must be 1, 2, or 4.")]
    InvalidSnoozeDuration(i32),

    #[error("Event store error: {0}")]
    EventStoreError(#[from] anyhow::Error),

    #[error("Notification not found: {0}")]
    NotificationNotFoundError(String),

    #[error("Invalid reminder type: {0}")]
    InvalidReminderType(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Schedule a preparation reminder
///
/// This command:
/// 1. Validates reminder_type is one of: "advance_prep", "morning", "day_of"
/// 2. Creates NotificationAggregate and emits ReminderScheduled event
/// 3. Returns the notification_id (which is the evento aggregator_id)
///
/// Note: evento::create() generates a ULID for the aggregator_id, which becomes the notification_id
pub async fn schedule_reminder<E: evento::Executor>(
    cmd: ScheduleReminderCommand,
    executor: &E,
) -> Result<String, NotificationError> {
    // Validate reminder_type
    if !["advance_prep", "morning", "day_of"].contains(&cmd.reminder_type.as_str()) {
        return Err(NotificationError::InvalidReminderType(cmd.reminder_type));
    }

    // Note: We don't generate notification_id here - evento::create() will generate it
    // The notification_id will be set to the aggregator_id in the event handler

    // Create event (notification_id will be populated from aggregator_id in the aggregate handler)
    let event = ReminderScheduled {
        notification_id: String::new(), // Will be overwritten by aggregator_id
        user_id: cmd.user_id,
        recipe_id: cmd.recipe_id,
        meal_date: cmd.meal_date,
        scheduled_time: cmd.scheduled_time,
        reminder_type: cmd.reminder_type,
        prep_hours: cmd.prep_hours,
        prep_task: cmd.prep_task,
    };

    // Use evento::create to emit the event and get the aggregator_id
    let notification_id = evento::create::<crate::aggregate::NotificationAggregate>()
        .data(&event)
        .map_err(|e| NotificationError::EventStoreError(e.into()))?
        .metadata(&true)
        .map_err(|e| NotificationError::EventStoreError(e.into()))?
        .commit(executor)
        .await
        .map_err(|e| NotificationError::EventStoreError(e.into()))?;

    Ok(notification_id)
}

/// Send a reminder (triggered by background worker)
///
/// This command:
/// 1. Emits ReminderSent event with delivery status
/// 2. Background worker calls this after attempting Web Push delivery
pub async fn send_reminder<E: evento::Executor>(
    cmd: SendReminderCommand,
    executor: &E,
) -> Result<(), NotificationError> {
    let sent_at = Utc::now().to_rfc3339();

    let event = ReminderSent {
        notification_id: cmd.notification_id.clone(),
        sent_at,
        delivery_status: cmd.delivery_status,
    };

    // Append event to existing aggregate
    evento::save::<crate::aggregate::NotificationAggregate>(&cmd.notification_id)
        .data(&event)
        .map_err(|e| NotificationError::EventStoreError(e.into()))?
        .metadata(&true)
        .map_err(|e| NotificationError::EventStoreError(e.into()))?
        .commit(executor)
        .await
        .map_err(|e| NotificationError::EventStoreError(e.into()))?;

    Ok(())
}

/// Dismiss a reminder
///
/// This command:
/// 1. Emits ReminderDismissed event
/// 2. User clicks "Dismiss" button in UI
pub async fn dismiss_reminder<E: evento::Executor>(
    cmd: DismissReminderCommand,
    executor: &E,
) -> Result<(), NotificationError> {
    let dismissed_at = Utc::now().to_rfc3339();

    let event = ReminderDismissed {
        notification_id: cmd.notification_id.clone(),
        dismissed_at,
    };

    // Append event to existing aggregate
    evento::save::<crate::aggregate::NotificationAggregate>(&cmd.notification_id)
        .data(&event)
        .map_err(|e| NotificationError::EventStoreError(e.into()))?
        .metadata(&true)
        .map_err(|e| NotificationError::EventStoreError(e.into()))?
        .commit(executor)
        .await
        .map_err(|e| NotificationError::EventStoreError(e.into()))?;

    Ok(())
}

/// Snooze a reminder
///
/// This command:
/// 1. Validates snooze_duration_hours is 1, 2, or 4
/// 2. Calculates new scheduled_time (current time + duration)
/// 3. Emits ReminderSnoozed event
pub async fn snooze_reminder<E: evento::Executor>(
    cmd: SnoozeReminderCommand,
    executor: &E,
) -> Result<(), NotificationError> {
    // Validate snooze duration
    if ![1, 2, 4].contains(&cmd.snooze_duration_hours) {
        return Err(NotificationError::InvalidSnoozeDuration(
            cmd.snooze_duration_hours,
        ));
    }

    // Calculate new scheduled time
    let now = Utc::now();
    let snoozed_until = now + chrono::Duration::hours(cmd.snooze_duration_hours as i64);
    let snoozed_until_str = snoozed_until.to_rfc3339();

    let event = ReminderSnoozed {
        notification_id: cmd.notification_id.clone(),
        snoozed_until: snoozed_until_str,
        snooze_duration_hours: cmd.snooze_duration_hours,
    };

    // Append event to existing aggregate
    evento::save::<crate::aggregate::NotificationAggregate>(&cmd.notification_id)
        .data(&event)
        .map_err(|e| NotificationError::EventStoreError(e.into()))?
        .metadata(&true)
        .map_err(|e| NotificationError::EventStoreError(e.into()))?
        .commit(executor)
        .await
        .map_err(|e| NotificationError::EventStoreError(e.into()))?;

    Ok(())
}

/// Subscribe to push notifications
///
/// This command:
/// 1. Validates endpoint URL (must be HTTPS)
/// 2. Validates base64-encoded keys
/// 3. Generates subscription_id (UUID)
/// 4. Creates PushSubscription aggregate and emits PushSubscriptionCreated event
pub async fn subscribe_to_push<E: evento::Executor>(
    cmd: SubscribeToPushCommand,
    executor: &E,
) -> Result<String, NotificationError> {
    // Security: Validate endpoint URL format (must be HTTPS)
    if !cmd.endpoint.starts_with("https://") {
        return Err(NotificationError::InvalidInput(
            "Push endpoint must use HTTPS protocol".to_string(),
        ));
    }

    // Security: Validate endpoint is a valid URL
    if url::Url::parse(&cmd.endpoint).is_err() {
        return Err(NotificationError::InvalidInput(
            "Invalid push endpoint URL format".to_string(),
        ));
    }

    // Security: Validate p256dh_key is valid base64
    if base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&cmd.p256dh_key)
        .is_err()
    {
        return Err(NotificationError::InvalidInput(
            "Invalid p256dh_key: must be valid base64".to_string(),
        ));
    }

    // Security: Validate auth_key is valid base64
    if base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&cmd.auth_key)
        .is_err()
    {
        return Err(NotificationError::InvalidInput(
            "Invalid auth_key: must be valid base64".to_string(),
        ));
    }

    // Generate subscription_id
    let subscription_id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    let event = PushSubscriptionCreated {
        subscription_id: subscription_id.clone(),
        user_id: cmd.user_id,
        endpoint: cmd.endpoint,
        p256dh_key: cmd.p256dh_key,
        auth_key: cmd.auth_key,
        created_at,
    };

    // Use evento::create to emit the event (separate aggregate from NotificationAggregate)
    evento::create::<crate::aggregate::PushSubscriptionAggregate>()
        .data(&event)
        .map_err(|e| NotificationError::EventStoreError(e.into()))?
        .metadata(&true)
        .map_err(|e| NotificationError::EventStoreError(e.into()))?
        .commit(executor)
        .await
        .map_err(|e| NotificationError::EventStoreError(e.into()))?;

    Ok(subscription_id)
}
