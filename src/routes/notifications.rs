use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Json},
    Extension, Form,
};
use notifications::{
    commands::{
        complete_prep_task, dismiss_reminder, snooze_reminder, subscribe_to_push,
        CompletePrepTaskCommand, DismissReminderCommand, SnoozeReminderCommand,
        SubscribeToPushCommand,
    },
    read_model::{
        get_notification_by_id, get_push_subscription_status, get_user_pending_notifications,
        UserNotification,
    },
};
use serde::Deserialize;
use user::{
    commands::{change_notification_permission, ChangeNotificationPermissionCommand},
    read_model::can_prompt_for_notification_permission,
};

use crate::error::AppError;
use crate::middleware::auth::Auth;
use crate::routes::AppState;

/// Template for the notifications page
#[derive(Template)]
#[template(path = "pages/notifications.html")]
pub struct NotificationsTemplate {
    user: Option<()>,
    notifications: Vec<UserNotification>,
    push_enabled: bool,
    vapid_public_key: String,
    pub current_path: String,
}

/// GET /notifications - Show notifications page
///
/// Displays the user's pending prep reminders with dismiss/snooze actions.
/// AC #6: Displays recipe name, prep task, timing guidance
pub async fn notifications_page(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = &auth.user_id;

    let notifications = get_user_pending_notifications(&state.db_pool, user_id).await?;

    // TODO: Check if user has active push subscription
    let push_enabled = false;

    let template = NotificationsTemplate {
        user: Some(()),
        notifications,
        push_enabled,
        vapid_public_key: state.vapid_public_key.clone(),
        current_path: "/notifications".to_string(),
    };

    Ok(Html(template.render().map_err(|e| {
        AppError::InternalError(format!("Template error: {}", e))
    })?))
}

/// GET /api/notifications - List user's pending notifications
///
/// Returns JSON array of pending notifications for the authenticated user.
/// AC #6: Displays recipe name, prep task, timing guidance
pub async fn list_notifications(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = &auth.user_id;

    let notifications = get_user_pending_notifications(&state.db_pool, user_id).await?;

    Ok(Json(notifications))
}

/// POST /api/notifications/:id/dismiss - Mark reminder as complete
///
/// AC #7: User can dismiss notifications
///
/// **Security**: Returns PermissionDenied for both not found and unauthorized to prevent
/// notification ID enumeration via timing side-channel.
pub async fn dismiss_notification(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
    Path(notification_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Validate that notification belongs to user
    // Security: Always return PermissionDenied (not NotificationNotFound) to prevent enumeration
    let notification = get_notification_by_id(&state.db_pool, &notification_id)
        .await
        .map_err(|_| AppError::PermissionDenied)?
        .ok_or(AppError::PermissionDenied)?;

    if notification.user_id != auth.user_id {
        return Err(AppError::PermissionDenied);
    }

    let cmd = DismissReminderCommand { notification_id };

    dismiss_reminder(cmd, &state.evento_executor).await?;

    // Return empty HTML to remove the element (TwinSpark will swap the target with this)
    Ok(Html(""))
}

/// POST /api/notifications/:id/complete - Mark prep task as complete
///
/// AC #1, #2: User can mark prep tasks as complete via "Mark Complete" button
/// AC #5, #6: Completion tracked per recipe/meal slot, removes from active reminders
///
/// **Security**: Returns PermissionDenied for both not found and unauthorized to prevent
/// notification ID enumeration via timing side-channel.
pub async fn complete_prep_task_handler(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
    Path(notification_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Validate that notification belongs to user
    // Security: Always return PermissionDenied (not NotificationNotFound) to prevent enumeration
    let notification = get_notification_by_id(&state.db_pool, &notification_id)
        .await
        .map_err(|_| AppError::PermissionDenied)?
        .ok_or(AppError::PermissionDenied)?;

    if notification.user_id != auth.user_id {
        return Err(AppError::PermissionDenied);
    }

    let cmd = CompletePrepTaskCommand {
        notification_id,
        recipe_id: notification.recipe_id,
    };

    complete_prep_task(cmd, &state.evento_executor).await?;

    // Return empty HTML to remove the element (TwinSpark will swap the target with this)
    Ok(Html(""))
}

/// Form data for snoozing a notification
#[derive(Deserialize)]
pub struct SnoozeForm {
    /// Duration in hours (1, 2, or 4)
    duration_hours: i32,
}

/// POST /api/notifications/:id/snooze - Snooze reminder
///
/// AC #8: User can snooze reminder (1 hour, 2 hours, 4 hours)
///
/// **Security**: Returns PermissionDenied for both not found and unauthorized to prevent
/// notification ID enumeration via timing side-channel.
pub async fn snooze_notification(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
    Path(notification_id): Path<String>,
    Form(form): Form<SnoozeForm>,
) -> Result<impl IntoResponse, AppError> {
    // Validate that notification belongs to user
    // Security: Always return PermissionDenied (not NotificationNotFound) to prevent enumeration
    let notification = get_notification_by_id(&state.db_pool, &notification_id)
        .await
        .map_err(|_| AppError::PermissionDenied)?
        .ok_or(AppError::PermissionDenied)?;

    if notification.user_id != auth.user_id {
        return Err(AppError::PermissionDenied);
    }

    let cmd = SnoozeReminderCommand {
        notification_id,
        snooze_duration_hours: form.duration_hours,
    };

    snooze_reminder(cmd, &state.evento_executor).await?;

    // Return empty HTML to remove the element (snoozed notifications are hidden from pending view)
    // TwinSpark will swap the target with this empty response
    Ok(Html(""))
}

/// JSON body for push subscription
#[derive(Deserialize)]
pub struct PushSubscriptionBody {
    endpoint: String,
    p256dh_key: String,
    auth_key: String,
}

/// POST /api/notifications/subscribe - Save Web Push subscription
///
/// AC #5: User can enable push notifications
pub async fn subscribe_push(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
    Json(body): Json<PushSubscriptionBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = auth.user_id.clone();

    let cmd = SubscribeToPushCommand {
        user_id,
        endpoint: body.endpoint,
        p256dh_key: body.p256dh_key,
        auth_key: body.auth_key,
    };

    let subscription_id = subscribe_to_push(cmd, &state.evento_executor).await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "subscription_id": subscription_id
    })))
}

/// POST /api/notifications/permission - Record notification permission change
///
/// AC #3, #5, #8: Track user's permission decision and denial timestamp for grace period
pub async fn record_permission_change(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let permission_status = body
        .get("permission_status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("permission_status is required".to_string()))?;

    // Validate permission_status
    if !["granted", "denied", "skipped"].contains(&permission_status) {
        return Err(AppError::BadRequest(
            "permission_status must be 'granted', 'denied', or 'skipped'".to_string(),
        ));
    }

    let cmd = ChangeNotificationPermissionCommand {
        user_id: auth.user_id.clone(),
        permission_status: permission_status.to_string(),
    };

    change_notification_permission(cmd, &state.evento_executor).await?;

    Ok(Json(serde_json::json!({
        "status": "success"
    })))
}

/// GET /api/notifications/status - Get push notification status
///
/// AC #7: Settings page shows current notification status and subscription count
pub async fn get_notification_status(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let push_status = get_push_subscription_status(&state.db_pool, &auth.user_id).await?;

    let can_prompt = can_prompt_for_notification_permission(&auth.user_id, &state.db_pool).await?;

    Ok(Json(serde_json::json!({
        "enabled": push_status.enabled,
        "subscription_count": push_status.subscription_count,
        "can_prompt": can_prompt
    })))
}
