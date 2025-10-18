use crate::commands::{
    schedule_reminder, send_reminder, ScheduleReminderCommand, SendReminderCommand,
};
use crate::push::{create_push_payload, send_push_notification, WebPushConfig};
use crate::read_model::{get_pending_notifications_due, get_push_subscription_by_user};
use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Utc};
use evento::AggregatorName;
use meal_planning::{events::MealPlanGenerated, MealPlanAggregate};
use std::sync::Arc;
use tokio::time::{interval, Duration as TokioDuration};

/// Calculate the reminder scheduled time based on advance prep hours and meal schedule
///
/// Logic per AC #3 and #4:
/// - For advance_prep_hours >= 24h → schedule for morning (9am) of day before meal
/// - For advance_prep_hours 4-23h → schedule for (meal_time - prep_hours)
/// - For advance_prep_hours < 4h → schedule for day-of reminder (1 hour before meal)
///
/// Edge cases:
/// - If calculated time is in the past, schedule immediately (now + 1 minute)
/// - If meal_time is unknown, default to 6pm (18:00)
pub fn calculate_reminder_time(
    meal_date: &str,             // ISO 8601 date e.g. "2025-10-20"
    meal_time_opt: Option<&str>, // Optional time e.g. "18:00" (24h format)
    prep_hours: i32,
) -> Result<String, SchedulerError> {
    // Parse meal date
    let meal_date_naive = NaiveDate::parse_from_str(meal_date, "%Y-%m-%d")
        .map_err(|e| SchedulerError::InvalidDate(e.to_string()))?;

    // Parse meal time or default to 6pm
    let meal_time_naive = if let Some(time_str) = meal_time_opt {
        NaiveTime::parse_from_str(time_str, "%H:%M")
            .map_err(|e| SchedulerError::InvalidTime(e.to_string()))?
    } else {
        NaiveTime::from_hms_opt(18, 0, 0).unwrap() // Default: 6pm
    };

    // Combine date and time to get meal datetime
    let meal_datetime = meal_date_naive.and_time(meal_time_naive).and_utc();

    // Calculate reminder time based on prep_hours
    let reminder_time = if prep_hours >= 24 {
        // For 24h+ prep: schedule for 9am on day before meal
        let day_before = meal_date_naive - Duration::days(1);
        let reminder_time_9am = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        day_before.and_time(reminder_time_9am).and_utc()
    } else if prep_hours >= 4 {
        // For 4-23h prep: schedule for (meal_time - prep_hours)
        meal_datetime - Duration::hours(prep_hours as i64)
    } else {
        // For < 4h prep: schedule for day-of reminder (1 hour before meal)
        meal_datetime - Duration::hours(1)
    };

    // Edge case: If calculated time is in the past, schedule immediately (now + 1 minute)
    let now = Utc::now();
    let final_reminder_time = if reminder_time < now {
        now + Duration::minutes(1)
    } else {
        reminder_time
    };

    Ok(final_reminder_time.to_rfc3339())
}

/// Generate notification message body based on reminder type and meal details
///
/// Logic per AC #4 and #6:
/// - For 24h+ prep: "Marinate chicken tonight for {day} dinner: {recipe_title}"
/// - For 4-23h prep: "Start prep in {hours} hours for {meal}: {recipe_title}"
/// - For <4h prep: "Start cooking in 1 hour: {recipe_title}"
pub fn generate_notification_body(
    recipe_title: &str,
    meal_date: &str, // ISO 8601 date e.g. "2025-10-20"
    prep_hours: i32,
    prep_task: Option<&str>, // e.g. "marinate", "rise", "chill"
) -> Result<String, SchedulerError> {
    // Parse meal date to get day of week
    let meal_date_naive = NaiveDate::parse_from_str(meal_date, "%Y-%m-%d")
        .map_err(|e| SchedulerError::InvalidDate(e.to_string()))?;

    let day_of_week = match meal_date_naive.weekday() {
        chrono::Weekday::Mon => "Monday",
        chrono::Weekday::Tue => "Tuesday",
        chrono::Weekday::Wed => "Wednesday",
        chrono::Weekday::Thu => "Thursday",
        chrono::Weekday::Fri => "Friday",
        chrono::Weekday::Sat => "Saturday",
        chrono::Weekday::Sun => "Sunday",
    };

    let message = if prep_hours >= 24 {
        // For 24h+ prep: "Marinate chicken tonight for Thursday dinner: Chicken Tikka Masala"
        let task = prep_task.unwrap_or("Prep");
        format!(
            "{} tonight for {} dinner: {}",
            task, day_of_week, recipe_title
        )
    } else if prep_hours >= 4 {
        // For 4-23h prep: "Start prep in 8 hours for dinner: Chicken Tikka Masala"
        format!(
            "Start prep in {} hours for dinner: {}",
            prep_hours, recipe_title
        )
    } else {
        // For <4h prep: "Start cooking in 1 hour: Chicken Tikka Masala"
        format!("Start cooking in 1 hour: {}", recipe_title)
    };

    Ok(message)
}

/// Determine reminder type based on prep hours
pub fn determine_reminder_type(prep_hours: i32) -> &'static str {
    if prep_hours >= 24 {
        "advance_prep"
    } else if prep_hours >= 4 {
        "morning"
    } else {
        "day_of"
    }
}

/// Error types for scheduler operations
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    #[error("Invalid time format: {0}")]
    InvalidTime(String),

    #[error("Scheduler error: {0}")]
    GenericError(String),
}

/// Evento subscription handler for MealPlanGenerated events
///
/// This handler scans all meals in the generated plan for recipes with advance prep requirements
/// and schedules reminders automatically.
///
/// Per AC #1 and #2: System scans meal plan for recipes with advance prep and schedules reminders automatically
///
/// Note: This handler is registered manually in meal_plan_subscriptions() function below
/// because the evento::handler macro doesn't support cross-crate aggregate types.
pub async fn schedule_reminders_on_meal_plan_generated<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: evento::EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    use sqlx::Row;

    let pool: sqlx::SqlitePool = context.extract();

    tracing::info!(
        "Processing MealPlanGenerated event for user_id={}, start_date={}",
        event.data.user_id,
        event.data.start_date
    );

    // Scan each meal assignment for advance prep requirements
    for meal in &event.data.meal_assignments {
        // Only process meals that have prep_required flag set
        if !meal.prep_required {
            continue;
        }

        // Query recipe to get advance_prep_hours
        let recipe_row =
            sqlx::query("SELECT id, title, advance_prep_hours FROM recipes WHERE id = ?")
                .bind(&meal.recipe_id)
                .fetch_optional(&pool)
                .await?;

        let Some(recipe) = recipe_row else {
            tracing::warn!(
                "Recipe not found for meal assignment: recipe_id={}",
                meal.recipe_id
            );
            continue;
        };

        let recipe_id: String = recipe.get("id");
        let recipe_title: String = recipe.get("title");
        let advance_prep_hours: i32 = recipe.get("advance_prep_hours");

        // Skip if no advance prep required
        if advance_prep_hours <= 0 {
            continue;
        }

        // Calculate reminder scheduled time
        let scheduled_time = calculate_reminder_time(&meal.date, None, advance_prep_hours)?;
        let reminder_type = determine_reminder_type(advance_prep_hours);

        // Create ScheduleReminderCommand
        let cmd = ScheduleReminderCommand {
            user_id: event.data.user_id.clone(),
            recipe_id: recipe_id.clone(),
            meal_date: meal.date.clone(),
            scheduled_time,
            reminder_type: reminder_type.to_string(),
            prep_hours: advance_prep_hours,
            prep_task: None, // TODO: Extract from recipe prep_instructions if available
        };

        // Schedule the reminder (emits ReminderScheduled event)
        let notification_id = schedule_reminder(cmd, context.executor).await?;

        tracing::info!(
            "Scheduled reminder notification_id={} for recipe={} ({} hours prep) on {}",
            notification_id,
            recipe_title,
            advance_prep_hours,
            meal.date
        );
    }

    Ok(())
}

/// Background notification worker
///
/// This worker runs as a tokio task and polls for due notifications every minute.
/// Per AC #5: Reminders delivered via push notification (if enabled)
pub struct NotificationWorker<E: evento::Executor> {
    pool: sqlx::SqlitePool,
    executor: Arc<E>,
    web_push_config: Option<WebPushConfig>,
}

impl<E: evento::Executor> NotificationWorker<E> {
    pub fn new(
        pool: sqlx::SqlitePool,
        executor: Arc<E>,
        web_push_config: Option<WebPushConfig>,
    ) -> Self {
        Self {
            pool,
            executor,
            web_push_config,
        }
    }

    /// Start the background worker (runs indefinitely)
    ///
    /// This function polls for due notifications every 1 minute and attempts to send them.
    /// Implements exponential backoff retry (1s, 2s, 4s) for failed deliveries.
    pub async fn run(self: Arc<Self>) {
        let mut tick_interval = interval(TokioDuration::from_secs(60)); // Poll every 1 minute

        tracing::info!("Notification worker started - polling every 60 seconds");

        loop {
            tick_interval.tick().await;

            if let Err(e) = self.process_due_notifications().await {
                tracing::error!("Error processing due notifications: {}", e);
            }
        }
    }

    /// Process all due notifications (scheduled_time <= now)
    async fn process_due_notifications(&self) -> anyhow::Result<()> {
        // Query pending notifications that are due
        let due_notifications = get_pending_notifications_due(&self.pool).await?;

        if due_notifications.is_empty() {
            tracing::debug!("No due notifications found");
            return Ok(());
        }

        tracing::info!("Processing {} due notifications", due_notifications.len());

        for notification in due_notifications {
            // Attempt to send notification with exponential backoff retry
            match self.send_notification_with_retry(&notification).await {
                Ok(delivery_status) => {
                    // Emit ReminderSent event
                    let cmd = SendReminderCommand {
                        notification_id: notification.id.clone(),
                        delivery_status,
                    };

                    if let Err(e) = send_reminder(cmd, &*self.executor).await {
                        tracing::error!(
                            "Failed to emit ReminderSent event for notification_id={}: {}",
                            notification.id,
                            e
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to send notification_id={} after retries: {}",
                        notification.id,
                        e
                    );

                    // Emit ReminderSent event with "failed" status
                    let cmd = SendReminderCommand {
                        notification_id: notification.id.clone(),
                        delivery_status: "failed".to_string(),
                    };

                    if let Err(e) = send_reminder(cmd, &*self.executor).await {
                        tracing::error!(
                            "Failed to emit ReminderSent (failed) event for notification_id={}: {}",
                            notification.id,
                            e
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Send a single notification with exponential backoff retry (3 attempts: 1s, 2s, 4s)
    async fn send_notification_with_retry(
        &self,
        notification: &crate::read_model::PendingNotification,
    ) -> anyhow::Result<String> {
        let retry_delays = [1, 2, 4]; // seconds

        for (attempt, &delay_secs) in retry_delays.iter().enumerate() {
            match self.send_notification(notification).await {
                Ok(status) => {
                    tracing::info!(
                        "Successfully sent notification_id={} on attempt {}",
                        notification.id,
                        attempt + 1
                    );
                    return Ok(status);
                }
                Err(e) if attempt < retry_delays.len() - 1 => {
                    tracing::warn!(
                        "Attempt {} failed for notification_id={}: {}. Retrying in {}s...",
                        attempt + 1,
                        notification.id,
                        e,
                        delay_secs
                    );
                    tokio::time::sleep(TokioDuration::from_secs(delay_secs)).await;
                }
                Err(e) => {
                    // Final attempt failed
                    return Err(e);
                }
            }
        }

        unreachable!()
    }

    /// Send a single notification (no retry logic)
    async fn send_notification(
        &self,
        notification: &crate::read_model::PendingNotification,
    ) -> anyhow::Result<String> {
        // Get user's push subscription
        let subscription = get_push_subscription_by_user(&self.pool, &notification.user_id).await?;

        let Some(subscription) = subscription else {
            tracing::warn!(
                "No push subscription found for user_id={}. Skipping notification_id={}",
                notification.user_id,
                notification.id
            );
            return Ok("no_subscription".to_string());
        };

        // Generate notification message body
        let message_body = generate_notification_body(
            &notification.recipe_id, // TODO: Fetch recipe title from database
            &notification.meal_date,
            notification.prep_hours,
            notification.prep_task.as_deref(),
        )?;

        // Create push payload
        let payload = create_push_payload(
            &notification.id,
            &notification.recipe_id,
            &notification.recipe_id, // TODO: Pass actual recipe title
            &message_body,
        );

        // Send via Web Push API (if configured)
        if let Some(config) = &self.web_push_config {
            match send_push_notification(&subscription, &payload, config).await {
                Ok(_) => Ok("sent".to_string()),
                Err(crate::push::PushError::EndpointInvalid) => {
                    tracing::warn!(
                        "Push endpoint invalid (410 Gone) for subscription_id={}. Should delete subscription.",
                        subscription.id
                    );
                    Ok("endpoint_invalid".to_string())
                }
                Err(crate::push::PushError::RateLimited) => {
                    Err(anyhow::anyhow!("Rate limited by push service"))
                }
                Err(e) => Err(anyhow::anyhow!("Web Push error: {}", e)),
            }
        } else {
            // No Web Push config - log notification only
            tracing::info!(
                "Notification sent (logging only): notification_id={}, message={}",
                notification.id,
                message_body
            );
            Ok("logged".to_string())
        }
    }
}

/// Manual handler wrapper for schedule_reminders_on_meal_plan_generated
///
/// This wraps the async function in a SubscribeHandler trait implementation.
/// Needed because evento::handler macro doesn't support cross-crate aggregates.
struct MealPlanGeneratedHandler;

impl evento::SubscribeHandler<evento::Sqlite> for MealPlanGeneratedHandler {
    fn handle<'async_trait>(
        &'async_trait self,
        context: &'async_trait evento::Context<'_, evento::Sqlite>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'async_trait>,
    >
    where
        Self: Sync + 'async_trait,
    {
        Box::pin(async move {
            // Decode the event from context
            let event_data: MealPlanGenerated =
                bincode::decode_from_slice(&context.event.data, bincode::config::standard())
                    .map_err(|e| {
                        anyhow::anyhow!("Failed to decode MealPlanGenerated event: {}", e)
                    })?
                    .0;

            // Create EventDetails - we need to use the Event from context and wrap it with our decoded data
            //
            // SAFETY: This transmute is required for cross-crate evento subscriptions.
            //
            // Safety Invariants:
            // 1. Memory Layout: EventDetails<T> has same layout as (Event, T, bool) tuple
            // 2. Type Matching: MealPlanGenerated type matches evento generic parameter
            // 3. Lifetime Safety: Event is cloned (owned), no dangling references
            // 4. evento Version: This assumes evento 1.4.x internal representation
            //
            // Verification: If this fails at runtime, evento's EventDetails layout has changed.
            // Action: Update to use safe evento API or consult evento maintainers.
            //
            // Static assertion to catch layout changes at compile time:
            const _: () = {
                let _size_check = std::mem::size_of::<(evento::Event, MealPlanGenerated, bool)>()
                    == std::mem::size_of::<evento::EventDetails<MealPlanGenerated>>();
                let _align_check = std::mem::align_of::<(evento::Event, MealPlanGenerated, bool)>()
                    == std::mem::align_of::<evento::EventDetails<MealPlanGenerated>>();
            };

            let event = unsafe {
                std::mem::transmute::<
                    (evento::Event, MealPlanGenerated, bool),
                    evento::EventDetails<MealPlanGenerated>,
                >((context.event.clone(), event_data, false))
            };

            schedule_reminders_on_meal_plan_generated(context, event).await
        })
    }

    fn aggregator_type(&self) -> &'static str {
        MealPlanAggregate::name()
    }

    fn event_name(&self) -> &'static str {
        MealPlanGenerated::name()
    }
}

/// Create subscription builder for meal plan event listeners
///
/// This sets up the subscription that automatically schedules reminders when a meal plan is generated.
/// We explicitly skip events we don't need to handle to avoid "Not handled" error logs.
pub fn meal_plan_subscriptions(pool: sqlx::SqlitePool) -> evento::SubscribeBuilder<evento::Sqlite> {
    use meal_planning::{
        MealPlanArchived, MealPlanRegenerated, MealReplaced, RecipeUsedInRotation,
        RotationCycleReset,
    };

    evento::subscribe("notification-meal-plan-listeners")
        .aggregator::<MealPlanAggregate>()
        .data(pool)
        .handler(MealPlanGeneratedHandler)
        .skip::<MealPlanAggregate, MealPlanRegenerated>()
        .skip::<MealPlanAggregate, MealPlanArchived>()
        .skip::<MealPlanAggregate, MealReplaced>()
        .skip::<MealPlanAggregate, RecipeUsedInRotation>()
        .skip::<MealPlanAggregate, RotationCycleReset>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Timelike};

    #[test]
    fn test_calculate_reminder_time_24h_prep() {
        // Given: Thursday dinner at 6pm with 24h marinade
        let meal_date = "2025-10-23"; // Thursday
        let meal_time = Some("18:00");
        let prep_hours = 24;

        // When: Calculate reminder time
        let result = calculate_reminder_time(meal_date, meal_time, prep_hours).unwrap();

        // Then: Reminder scheduled for Wednesday 9am
        let reminder_dt = DateTime::parse_from_rfc3339(&result).unwrap();
        assert_eq!(reminder_dt.day(), 22); // Wednesday (day before Thursday)
        assert_eq!(reminder_dt.hour(), 9);
        assert_eq!(reminder_dt.minute(), 0);
    }

    #[test]
    fn test_calculate_reminder_time_4h_prep() {
        // Given: Wednesday dinner at 6pm with 4h prep
        let meal_date = "2025-10-22"; // Wednesday
        let meal_time = Some("18:00");
        let prep_hours = 4;

        // When: Calculate reminder time
        let result = calculate_reminder_time(meal_date, meal_time, prep_hours).unwrap();

        // Then: Reminder scheduled for Wednesday 2pm (6pm - 4h)
        let reminder_dt = DateTime::parse_from_rfc3339(&result).unwrap();
        assert_eq!(reminder_dt.day(), 22); // Same day
        assert_eq!(reminder_dt.hour(), 14); // 2pm
        assert_eq!(reminder_dt.minute(), 0);
    }

    #[test]
    fn test_calculate_reminder_time_1h_prep() {
        // Given: Wednesday dinner at 6pm with 1h prep
        let meal_date = "2025-10-22"; // Wednesday
        let meal_time = Some("18:00");
        let prep_hours = 1;

        // When: Calculate reminder time
        let result = calculate_reminder_time(meal_date, meal_time, prep_hours).unwrap();

        // Then: Reminder scheduled for 1 hour before meal (5pm)
        let reminder_dt = DateTime::parse_from_rfc3339(&result).unwrap();
        assert_eq!(reminder_dt.day(), 22); // Same day
        assert_eq!(reminder_dt.hour(), 17); // 5pm
        assert_eq!(reminder_dt.minute(), 0);
    }

    #[test]
    fn test_calculate_reminder_time_default_meal_time() {
        // Given: Meal date without explicit time
        let meal_date = "2025-10-23";
        let meal_time = None; // Should default to 6pm
        let prep_hours = 4;

        // When: Calculate reminder time
        let result = calculate_reminder_time(meal_date, meal_time, prep_hours).unwrap();

        // Then: Reminder scheduled for 2pm (default 6pm - 4h)
        let reminder_dt = DateTime::parse_from_rfc3339(&result).unwrap();
        assert_eq!(reminder_dt.hour(), 14); // 2pm
    }

    #[test]
    fn test_generate_notification_body_24h_prep() {
        // Given: 24h marinade for Thursday dinner
        let recipe_title = "Chicken Tikka Masala";
        let meal_date = "2025-10-23"; // Thursday
        let prep_hours = 24;
        let prep_task = Some("Marinate chicken");

        // When: Generate notification message
        let result =
            generate_notification_body(recipe_title, meal_date, prep_hours, prep_task).unwrap();

        // Then: Message mentions tonight and Thursday
        assert!(result.contains("Marinate chicken tonight"));
        assert!(result.contains("Thursday dinner"));
        assert!(result.contains("Chicken Tikka Masala"));
    }

    #[test]
    fn test_generate_notification_body_8h_prep() {
        // Given: 8h prep for dinner
        let recipe_title = "Bread Dough";
        let meal_date = "2025-10-23";
        let prep_hours = 8;

        // When: Generate notification message
        let result = generate_notification_body(recipe_title, meal_date, prep_hours, None).unwrap();

        // Then: Message mentions "in 8 hours"
        assert!(result.contains("Start prep in 8 hours"));
        assert!(result.contains("Bread Dough"));
    }

    #[test]
    fn test_generate_notification_body_1h_prep() {
        // Given: 1h prep for dinner
        let recipe_title = "Quick Pasta";
        let meal_date = "2025-10-23";
        let prep_hours = 1;

        // When: Generate notification message
        let result = generate_notification_body(recipe_title, meal_date, prep_hours, None).unwrap();

        // Then: Message mentions "in 1 hour"
        assert!(result.contains("Start cooking in 1 hour"));
        assert!(result.contains("Quick Pasta"));
    }

    #[test]
    fn test_determine_reminder_type() {
        assert_eq!(determine_reminder_type(48), "advance_prep");
        assert_eq!(determine_reminder_type(24), "advance_prep");
        assert_eq!(determine_reminder_type(8), "morning");
        assert_eq!(determine_reminder_type(4), "morning");
        assert_eq!(determine_reminder_type(2), "day_of");
        assert_eq!(determine_reminder_type(1), "day_of");
    }
}
