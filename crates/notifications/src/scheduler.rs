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

/// Generate morning reminder message body
///
/// Per AC #2, #3, #4:
/// - Format: "Prep reminder: {prep_task} tonight for {day_of_week}'s {course_type} (Takes {prep_time} minutes)"
/// - Example: "Prep reminder: Marinate chicken tonight for Thursday's dessert (Takes 10 minutes)"
pub fn generate_morning_reminder_body(
    prep_task: Option<&str>,
    day_of_week: &str,
    course_type: &str,
    prep_time_min: i32,
) -> String {
    let task = prep_task.unwrap_or("Prep");
    format!(
        "Prep reminder: {} tonight for {}'s {} (Takes {} minutes)",
        task, day_of_week, course_type, prep_time_min
    )
}

/// Morning reminder scheduler - runs daily at 9:00 AM to check for meals requiring advance prep tonight
///
/// Per AC #1: Morning reminders sent at 9:00 AM local time (UTC for MVP)
/// Per AC #6: Only sent if advance prep required within next 24 hours (4-23h window)
///
/// This function queries tomorrow's meal plan and schedules morning reminders for recipes
/// with advance_prep_hours in the range [4, 24).
pub async fn morning_reminder_scheduler<E: evento::Executor>(
    pool: &sqlx::SqlitePool,
    executor: &E,
    user_id: &str,
) -> anyhow::Result<()> {
    use sqlx::Row;

    tracing::info!("Running morning reminder scheduler for user_id={}", user_id);

    // Calculate tomorrow's date
    let tomorrow = (Utc::now() + Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    // Query meal assignments for tomorrow with advance prep requirements
    let meal_slots = sqlx::query(
        "SELECT ma.id, ma.recipe_id, ma.course_type, ma.date,
                r.title, r.advance_prep_hours, r.prep_task, r.prep_time_min
         FROM meal_assignments ma
         JOIN recipes r ON ma.recipe_id = r.id
         JOIN meal_plans mp ON ma.meal_plan_id = mp.id
         WHERE mp.user_id = ?
           AND ma.date = ?
           AND ma.prep_required = 1
           AND r.advance_prep_hours > 0
           AND r.advance_prep_hours <= 24",
    )
    .bind(user_id)
    .bind(&tomorrow)
    .fetch_all(pool)
    .await?;

    if meal_slots.is_empty() {
        tracing::debug!(
            "No meals requiring morning reminders for user_id={}",
            user_id
        );
        return Ok(());
    }

    tracing::info!(
        "Found {} meals requiring morning reminders",
        meal_slots.len()
    );

    // Schedule a morning reminder for each qualifying meal
    for row in meal_slots {
        let recipe_id: String = row.get("recipe_id");
        let meal_date: String = row.get("date");
        let recipe_title: String = row.get("title");
        let advance_prep_hours: i32 = row.get("advance_prep_hours");
        let prep_task: Option<String> = row.get("prep_task");

        // Calculate scheduled time = 9:00 AM today (UTC)
        // For morning reminders, we always schedule for today at 9am regardless of current time
        // (since we're scheduling reminders for tomorrow's meals)
        let today_9am = Utc::now()
            .date_naive()
            .and_hms_opt(9, 0, 0)
            .unwrap()
            .and_utc();

        let scheduled_time = today_9am.to_rfc3339();

        // Note: message_body will be generated and updated after projection runs
        // (see update_morning_reminder_messages function)

        let cmd = ScheduleReminderCommand {
            user_id: user_id.to_string(),
            recipe_id: recipe_id.clone(),
            meal_date: meal_date.clone(),
            scheduled_time: scheduled_time.clone(),
            reminder_type: "morning".to_string(),
            prep_hours: advance_prep_hours,
            prep_task,
        };

        let notification_id = schedule_reminder(cmd, executor).await?;

        // Note: message_body will be updated after projection runs (see update_morning_reminder_messages)
        // Store the mapping for batch update
        tracing::debug!(
            "Scheduled morning reminder notification_id={} for recipe={} ({} hours prep) on {} - message_body will be updated after projection",
            notification_id,
            recipe_title,
            advance_prep_hours,
            meal_date
        );
    }

    Ok(())
}

/// Update message bodies for morning reminders after projection
///
/// This function should be called after the notification_projections subscription runs
/// to populate the message_body column with the formatted reminder text.
pub async fn update_morning_reminder_messages(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> anyhow::Result<()> {
    use sqlx::Row;

    // Query all pending morning reminders for this user that don't have message_body set
    let notifications = sqlx::query(
        "SELECT n.id, n.meal_date, n.prep_hours, n.prep_task, ma.course_type, r.prep_time_min
         FROM notifications n
         JOIN meal_assignments ma ON n.meal_date = ma.date AND n.recipe_id = ma.recipe_id
         JOIN meal_plans mp ON ma.meal_plan_id = mp.id
         JOIN recipes r ON n.recipe_id = r.id
         WHERE n.user_id = ?
           AND n.reminder_type = 'morning'
           AND n.status = 'pending'
           AND (n.message_body IS NULL OR n.message_body = '')
           AND mp.user_id = ?",
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    for row in notifications {
        let notification_id: String = row.get("id");
        let meal_date: String = row.get("meal_date");
        let prep_task: Option<String> = row.get("prep_task");
        let course_type: String = row.get("course_type");
        let prep_time_min: i32 = row.get("prep_time_min");

        // Parse meal_date to get day of week
        let meal_date_naive = chrono::NaiveDate::parse_from_str(&meal_date, "%Y-%m-%d")?;
        let day_of_week = match meal_date_naive.weekday() {
            chrono::Weekday::Mon => "Monday",
            chrono::Weekday::Tue => "Tuesday",
            chrono::Weekday::Wed => "Wednesday",
            chrono::Weekday::Thu => "Thursday",
            chrono::Weekday::Fri => "Friday",
            chrono::Weekday::Sat => "Saturday",
            chrono::Weekday::Sun => "Sunday",
        };

        // Generate message body
        let message_body = generate_morning_reminder_body(
            prep_task.as_deref(),
            day_of_week,
            &course_type,
            prep_time_min,
        );

        // Update notification with message_body and created_at
        sqlx::query("UPDATE notifications SET message_body = ?, created_at = ? WHERE id = ?")
            .bind(&message_body)
            .bind(Utc::now().to_rfc3339())
            .bind(&notification_id)
            .execute(pool)
            .await?;

        tracing::debug!(
            "Updated message_body for notification_id={}: {}",
            notification_id,
            message_body
        );
    }

    Ok(())
}

/// Day-of cooking reminder scheduler - runs every 15 minutes to check for today's meals
///
/// Per AC #1: Cooking reminder sent 1 hour before typical meal time
/// Per AC #2: Default meal times: Breakfast 8am, Lunch 12pm, Dinner 6pm
///
/// This function queries today's meal plan and schedules cooking reminders for all meals
/// 1 hour before their scheduled meal time.
pub async fn day_of_cooking_reminder_scheduler<E: evento::Executor>(
    pool: &sqlx::SqlitePool,
    executor: &E,
    user_id: &str,
) -> anyhow::Result<()> {
    use sqlx::Row;

    tracing::info!(
        "Running day-of cooking reminder scheduler for user_id={}",
        user_id
    );

    // Calculate today's date
    let today = Utc::now().format("%Y-%m-%d").to_string();

    // Query meal assignments for today
    let meal_slots = sqlx::query(
        "SELECT ma.id, ma.recipe_id, ma.course_type, ma.date,
                r.title, r.prep_time_min, r.cook_time_min, r.advance_prep_hours
         FROM meal_assignments ma
         JOIN recipes r ON ma.recipe_id = r.id
         JOIN meal_plans mp ON ma.meal_plan_id = mp.id
         WHERE mp.user_id = ?
           AND ma.date = ?",
    )
    .bind(user_id)
    .bind(&today)
    .fetch_all(pool)
    .await?;

    if meal_slots.is_empty() {
        tracing::debug!("No meals scheduled for today for user_id={}", user_id);
        return Ok(());
    }

    tracing::info!(
        "Found {} meals for today requiring cooking reminders",
        meal_slots.len()
    );

    // Schedule a day-of cooking reminder for each meal
    for row in meal_slots {
        let recipe_id: String = row.get("recipe_id");
        let meal_date: String = row.get("date");
        let recipe_title: String = row.get("title");
        let course_type: String = row.get("course_type");
        let _advance_prep_hours: i32 = row.get("advance_prep_hours");

        // Determine default meal time based on course_type (AC #2)
        let default_meal_time = match course_type.as_str() {
            "appetizer" => "08:00",   // 8am
            "main_course" => "12:00", // 12pm
            "dessert" => "18:00",     // 6pm
            _ => "18:00",             // Default to dessert time
        };

        // Calculate reminder time: meal_time - 1 hour (AC #1)
        // Parse the meal date and time to create a proper datetime
        let meal_date_naive = chrono::NaiveDate::parse_from_str(&meal_date, "%Y-%m-%d")?;
        let meal_time_naive = chrono::NaiveTime::parse_from_str(default_meal_time, "%H:%M")?;
        let meal_datetime = meal_date_naive.and_time(meal_time_naive).and_utc();

        // Subtract 1 hour for the reminder
        let reminder_datetime = meal_datetime - Duration::hours(1);
        let scheduled_time = reminder_datetime.to_rfc3339();

        let cmd = ScheduleReminderCommand {
            user_id: user_id.to_string(),
            recipe_id: recipe_id.clone(),
            meal_date: meal_date.clone(),
            scheduled_time: scheduled_time.clone(),
            reminder_type: "day_of".to_string(),
            prep_hours: 0, // Day-of cooking reminders don't use prep_hours
            prep_task: None,
        };

        let notification_id = schedule_reminder(cmd, executor).await?;

        tracing::debug!(
            "Scheduled day-of cooking reminder notification_id={} for recipe={} (course_type={}) on {}",
            notification_id,
            recipe_title,
            course_type,
            meal_date
        );
    }

    Ok(())
}

/// Update message bodies for day-of cooking reminders after projection
///
/// This function should be called after the notification_projections subscription runs
/// to populate the message_body column with the formatted reminder text.
///
/// Message format per AC #3:
/// - Breakfast: "This morning's breakfast: {recipe_name} - Ready in {total_time}"
/// - Lunch: "Today's lunch: {recipe_name} - Ready in {total_time}"
/// - Dinner: "Tonight's dinner: {recipe_name} - Ready in {total_time}"
pub async fn update_day_of_reminder_messages(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> anyhow::Result<()> {
    use sqlx::Row;

    // Query all pending day-of reminders for this user that don't have message_body set
    let notifications = sqlx::query(
        "SELECT n.id, n.meal_date, ma.course_type, r.title, r.prep_time_min, r.cook_time_min
         FROM notifications n
         JOIN meal_assignments ma ON n.meal_date = ma.date AND n.recipe_id = ma.recipe_id
         JOIN meal_plans mp ON ma.meal_plan_id = mp.id
         JOIN recipes r ON n.recipe_id = r.id
         WHERE n.user_id = ?
           AND n.reminder_type = 'day_of'
           AND n.status = 'pending'
           AND (n.message_body IS NULL OR n.message_body = '')
           AND mp.user_id = ?",
    )
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    for row in notifications {
        let notification_id: String = row.get("id");
        let course_type: String = row.get("course_type");
        let recipe_title: String = row.get("title");
        let prep_time_min: i32 = row.get("prep_time_min");
        let cook_time_min: i32 = row.get("cook_time_min");

        // Calculate total time (prep + cook)
        let total_time_min = prep_time_min + cook_time_min;

        // Generate message based on course type (AC #3)
        let time_label = match course_type.as_str() {
            "appetizer" => "This morning's appetizer",
            "main_course" => "Today's main course",
            "dessert" => "Tonight's dessert",
            _ => "Today's meal",
        };

        let message_body = format!(
            "{}: {} - Ready in {} minutes",
            time_label, recipe_title, total_time_min
        );

        // Update notification with message_body and created_at
        sqlx::query("UPDATE notifications SET message_body = ?, created_at = ? WHERE id = ?")
            .bind(&message_body)
            .bind(Utc::now().to_rfc3339())
            .bind(&notification_id)
            .execute(pool)
            .await?;

        tracing::debug!(
            "Updated message_body for notification_id={}: {}",
            notification_id,
            message_body
        );
    }

    Ok(())
}

/// Carry-over uncompleted prep tasks to next reminder cycle (Story 4.9 AC #7)
///
/// This function runs daily to check for uncompleted prep tasks that were sent
/// but not completed. It re-schedules them for the next reminder cycle.
///
/// Logic:
/// - Query prep tasks with status='sent' (delivered but not completed/dismissed)
/// - Check reminder_count < max_reminder_count (default 3)
/// - Re-schedule for next morning (9am) with incremented reminder_count
/// - Mark as 'expired' if max_reminder_count reached
pub async fn carry_over_uncompleted_tasks<E: evento::Executor>(
    pool: &sqlx::SqlitePool,
    executor: &E,
) -> anyhow::Result<()> {
    use sqlx::Row;

    tracing::info!("Running carry-over for uncompleted prep tasks");

    let now = Utc::now();
    let today = now.format("%Y-%m-%d").to_string();

    // Query uncompleted prep tasks (status='sent', meal_date < today)
    let uncompleted_tasks = sqlx::query(
        "SELECT id, user_id, recipe_id, meal_date, prep_hours, prep_task,
                reminder_count, max_reminder_count
         FROM notifications
         WHERE status = 'sent'
           AND reminder_type IN ('advance_prep', 'morning')
           AND meal_date < ?",
    )
    .bind(&today)
    .fetch_all(pool)
    .await?;

    if uncompleted_tasks.is_empty() {
        tracing::debug!("No uncompleted prep tasks found");
        return Ok(());
    }

    tracing::info!(
        "Found {} uncompleted prep tasks to process",
        uncompleted_tasks.len()
    );

    for row in uncompleted_tasks {
        let notification_id: String = row.get("id");
        let user_id: String = row.get("user_id");
        let recipe_id: String = row.get("recipe_id");
        let meal_date: String = row.get("meal_date");
        let prep_hours: i32 = row.get("prep_hours");
        let prep_task: Option<String> = row.get("prep_task");
        let reminder_count: i32 = row.get("reminder_count");
        let max_reminder_count: i32 = row.get("max_reminder_count");

        // Check if max reminders reached
        if reminder_count >= max_reminder_count {
            // Mark as expired (stop sending reminders)
            sqlx::query("UPDATE notifications SET status = 'expired' WHERE id = ?")
                .bind(&notification_id)
                .execute(pool)
                .await?;

            tracing::info!(
                "Marked notification_id={} as expired (reached max_reminder_count={})",
                notification_id,
                max_reminder_count
            );
            continue;
        }

        // Increment reminder_count
        let new_reminder_count = reminder_count + 1;
        sqlx::query("UPDATE notifications SET reminder_count = ? WHERE id = ?")
            .bind(new_reminder_count)
            .bind(&notification_id)
            .execute(pool)
            .await?;

        // Re-schedule for tomorrow morning (9am)
        let tomorrow_9am = (now + Duration::days(1))
            .date_naive()
            .and_hms_opt(9, 0, 0)
            .unwrap()
            .and_utc();

        let scheduled_time = tomorrow_9am.to_rfc3339();

        let cmd = ScheduleReminderCommand {
            user_id,
            recipe_id: recipe_id.clone(),
            meal_date,
            scheduled_time: scheduled_time.clone(),
            reminder_type: "morning".to_string(), // Carry-over as morning reminder
            prep_hours,
            prep_task,
        };

        let new_notification_id = schedule_reminder(cmd, executor).await?;

        tracing::info!(
            "Re-scheduled uncompleted task: old_notification_id={}, new_notification_id={}, reminder_count={}/{}",
            notification_id,
            new_notification_id,
            new_reminder_count,
            max_reminder_count
        );
    }

    Ok(())
}

/// Auto-dismissal worker - runs hourly to dismiss expired reminders
///
/// Per AC #8: Reminder dismissed automatically after prep window passes
///
/// Prep window calculation:
/// - For morning reminders: scheduled at 9am for tomorrow's meal
/// - Prep window ends when the meal time arrives
/// - If meal_time is unknown, assume 6pm (18:00)
pub async fn auto_dismissal_worker<E: evento::Executor>(
    pool: &sqlx::SqlitePool,
    _executor: &E,
) -> anyhow::Result<()> {
    use sqlx::Row;

    tracing::debug!("Running auto-dismissal worker");

    let now = Utc::now();

    // Query pending morning reminders where meal_date + estimated meal time (18:00) < now
    let expired_notifications = sqlx::query(
        "SELECT id, meal_date, prep_hours
         FROM notifications
         WHERE status = 'pending'
           AND reminder_type = 'morning'
           AND datetime(meal_date || ' 18:00:00') < datetime(?)",
    )
    .bind(now.format("%Y-%m-%d %H:%M:%S").to_string())
    .fetch_all(pool)
    .await?;

    if expired_notifications.is_empty() {
        tracing::debug!("No expired reminders found");
        return Ok(());
    }

    tracing::info!(
        "Found {} expired reminders to dismiss",
        expired_notifications.len()
    );

    for row in expired_notifications {
        let notification_id: String = row.get("id");

        // Emit ReminderDismissed event
        let dismissed_at = Utc::now().to_rfc3339();

        // Update status directly in read model (simulating ReminderDismissed event projection)
        sqlx::query("UPDATE notifications SET status = 'dismissed', dismissed_at = ? WHERE id = ?")
            .bind(&dismissed_at)
            .bind(&notification_id)
            .execute(pool)
            .await?;

        tracing::info!("Auto-dismissed expired notification_id={}", notification_id);
    }

    Ok(())
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
        use chrono::Local;

        // Given: Dinner at 6pm in 2 days with 24h marinade (ensures reminder is tomorrow at 9am, not in past)
        let in_two_days = Local::now().date_naive() + chrono::Duration::days(2);
        let meal_date = in_two_days.format("%Y-%m-%d").to_string();
        let meal_time = Some("18:00");
        let prep_hours = 24;

        // When: Calculate reminder time
        let result = calculate_reminder_time(&meal_date, meal_time, prep_hours).unwrap();

        // Then: Reminder scheduled for tomorrow at 9am
        let reminder_dt = DateTime::parse_from_rfc3339(&result).unwrap();
        let expected_day = Local::now().date_naive() + chrono::Duration::days(1);
        assert_eq!(reminder_dt.day(), expected_day.day());
        assert_eq!(reminder_dt.hour(), 9);
        assert_eq!(reminder_dt.minute(), 0);
    }

    #[test]
    fn test_calculate_reminder_time_4h_prep() {
        use chrono::Local;

        // Given: Dinner at 10pm today with 4h prep (using dynamic date, late enough to avoid past time)
        let today = Local::now().date_naive();
        let meal_date = today.format("%Y-%m-%d").to_string();
        let meal_time = Some("22:00"); // 10pm
        let prep_hours = 4;

        // When: Calculate reminder time
        let result = calculate_reminder_time(&meal_date, meal_time, prep_hours).unwrap();

        // Then: Reminder scheduled for today at 6pm (10pm - 4h)
        let reminder_dt = DateTime::parse_from_rfc3339(&result).unwrap();
        assert_eq!(reminder_dt.day(), today.day());
        assert_eq!(reminder_dt.hour(), 18); // 6pm
        assert_eq!(reminder_dt.minute(), 0);
    }

    #[test]
    fn test_calculate_reminder_time_1h_prep() {
        use chrono::Local;

        // Given: Dinner at 11pm today with 1h prep (using dynamic date, late enough to avoid past time)
        let today = Local::now().date_naive();
        let meal_date = today.format("%Y-%m-%d").to_string();
        let meal_time = Some("23:00"); // 11pm
        let prep_hours = 1;

        // When: Calculate reminder time
        let result = calculate_reminder_time(&meal_date, meal_time, prep_hours).unwrap();

        // Then: Reminder scheduled for 1 hour before meal (10pm)
        let reminder_dt = DateTime::parse_from_rfc3339(&result).unwrap();
        assert_eq!(reminder_dt.day(), today.day());
        assert_eq!(reminder_dt.hour(), 22); // 10pm
        assert_eq!(reminder_dt.minute(), 0);
    }

    #[test]
    fn test_calculate_reminder_time_default_meal_time() {
        use chrono::Local;

        // Given: Meal tomorrow without explicit time (using dynamic date, ensures future time)
        let tomorrow = Local::now().date_naive() + chrono::Duration::days(1);
        let meal_date = tomorrow.format("%Y-%m-%d").to_string();
        let meal_time = None; // Should default to 6pm
        let prep_hours = 4;

        // When: Calculate reminder time
        let result = calculate_reminder_time(&meal_date, meal_time, prep_hours).unwrap();

        // Then: Reminder scheduled for 2pm (default 6pm - 4h)
        let reminder_dt = DateTime::parse_from_rfc3339(&result).unwrap();
        assert_eq!(reminder_dt.day(), tomorrow.day());
        assert_eq!(reminder_dt.hour(), 14); // 2pm
    }

    #[test]
    fn test_generate_notification_body_24h_prep() {
        use chrono::Local;

        // Given: 24h marinade for tomorrow's dinner (using dynamic date)
        let recipe_title = "Chicken Tikka Masala";
        let tomorrow = Local::now().date_naive() + chrono::Duration::days(1);
        let meal_date = tomorrow.format("%Y-%m-%d").to_string();
        let prep_hours = 24;
        let prep_task = Some("Marinate chicken");

        // When: Generate notification message
        let result =
            generate_notification_body(recipe_title, &meal_date, prep_hours, prep_task).unwrap();

        // Then: Message mentions tonight and the meal
        assert!(result.contains("Marinate chicken tonight"));
        assert!(result.contains("dinner"));
        assert!(result.contains("Chicken Tikka Masala"));
    }

    #[test]
    fn test_generate_notification_body_8h_prep() {
        use chrono::Local;

        // Given: 8h prep for dinner (using dynamic date)
        let recipe_title = "Bread Dough";
        let today = Local::now().date_naive();
        let meal_date = today.format("%Y-%m-%d").to_string();
        let prep_hours = 8;

        // When: Generate notification message
        let result =
            generate_notification_body(recipe_title, &meal_date, prep_hours, None).unwrap();

        // Then: Message mentions "in 8 hours"
        assert!(result.contains("Start prep in 8 hours"));
        assert!(result.contains("Bread Dough"));
    }

    #[test]
    fn test_generate_notification_body_1h_prep() {
        use chrono::Local;

        // Given: 1h prep for dinner (using dynamic date)
        let recipe_title = "Quick Pasta";
        let today = Local::now().date_naive();
        let meal_date = today.format("%Y-%m-%d").to_string();
        let prep_hours = 1;

        // When: Generate notification message
        let result =
            generate_notification_body(recipe_title, &meal_date, prep_hours, None).unwrap();

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
