use crate::read_model::PushSubscription;
use serde::{Deserialize, Serialize};

/// Web Push configuration (VAPID keys)
#[derive(Debug, Clone)]
pub struct WebPushConfig {
    pub vapid_public_key: String,
    pub vapid_private_key: String,
    pub subject: String, // e.g., "mailto:contact@imkitchen.app"
}

/// Notification payload structure for Web Push
#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
    pub icon: String,
    pub badge: String,
    pub actions: Vec<NotificationAction>,
    pub data: NotificationData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationAction {
    pub action: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationData {
    pub recipe_id: String,
    pub notification_id: String,
    pub url: String,
}

/// Create a notification payload for prep reminders
///
/// This is called by the background worker to generate the notification JSON
/// that will be sent via Web Push API.
pub fn create_push_payload(
    notification_id: &str,
    recipe_id: &str,
    _recipe_title: &str,
    message_body: &str,
) -> NotificationPayload {
    NotificationPayload {
        title: "Prep Reminder".to_string(),
        body: message_body.to_string(),
        icon: "/static/icons/icon-192.png".to_string(),
        badge: "/static/icons/badge-72.png".to_string(),
        actions: vec![
            NotificationAction {
                action: "view".to_string(),
                title: "View Recipe".to_string(),
            },
            NotificationAction {
                action: "dismiss".to_string(),
                title: "Dismiss".to_string(),
            },
        ],
        data: NotificationData {
            recipe_id: recipe_id.to_string(),
            notification_id: notification_id.to_string(),
            url: format!(
                "/recipes/{}?highlight=prep&notification_id={}",
                recipe_id, notification_id
            ),
        },
    }
}

/// Create a notification payload for day-of cooking reminders
///
/// Per AC #4: Reminder displays recipe image and key info
/// Per AC #5: Tapping opens recipe detail in cooking mode
/// Per AC #6: User can dismiss or snooze (30 min, 1 hour)
///
/// This function generates a Web Push notification payload with:
/// - Recipe image as icon
/// - Cooking mode deep link (/recipes/{id}?mode=cooking)
/// - Action buttons: Snooze 30min, Snooze 1hour, Dismiss
pub fn create_cooking_push_payload(
    notification_id: &str,
    recipe_id: &str,
    _recipe_title: &str,
    recipe_image_url: &str,
    message_body: &str,
) -> NotificationPayload {
    NotificationPayload {
        title: "Cooking Reminder".to_string(),
        body: message_body.to_string(),
        // AC #4: Use recipe image as notification icon
        icon: if recipe_image_url.is_empty() {
            "/static/icons/icon-192.png".to_string()
        } else {
            recipe_image_url.to_string()
        },
        badge: "/static/icons/badge-72.png".to_string(),
        // AC #6: Action buttons for snooze (30min, 1hour) and dismiss
        actions: vec![
            NotificationAction {
                action: "snooze_30".to_string(),
                title: "Snooze 30 min".to_string(),
            },
            NotificationAction {
                action: "snooze_60".to_string(),
                title: "Snooze 1 hour".to_string(),
            },
            NotificationAction {
                action: "dismiss".to_string(),
                title: "Dismiss".to_string(),
            },
        ],
        data: NotificationData {
            recipe_id: recipe_id.to_string(),
            notification_id: notification_id.to_string(),
            // AC #5: Deep link with mode=cooking parameter
            url: format!("/recipes/{}?mode=cooking", recipe_id),
        },
    }
}

/// Send a push notification using Web Push API
///
/// This function:
/// 1. Serializes the notification payload to JSON
/// 2. Creates VAPID signature from private key
/// 3. Builds WebPushMessage with subscription details
/// 4. Sends via web-push crate (which uses isahc HTTP client)
/// 5. Handles response status codes (200=success, 410=expired, 429=rate limit)
pub async fn send_push_notification(
    subscription: &PushSubscription,
    payload: &NotificationPayload,
    config: &WebPushConfig,
) -> Result<(), PushError> {
    use web_push::{
        IsahcWebPushClient, SubscriptionInfo, VapidSignatureBuilder, WebPushClient,
        WebPushMessageBuilder,
    };

    // Serialize payload to JSON
    let payload_json =
        serde_json::to_string(payload).map_err(|e| PushError::SerializationError(e.to_string()))?;

    // Create subscription info from stored subscription data
    let subscription_info = SubscriptionInfo::new(
        &subscription.endpoint,
        &subscription.p256dh_key,
        &subscription.auth_key,
    );

    // Build VAPID signature
    let mut sig_builder =
        VapidSignatureBuilder::from_pem(config.vapid_private_key.as_bytes(), &subscription_info)
            .map_err(|e| {
                PushError::VapidError(format!("Failed to build VAPID signature: {:?}", e))
            })?;

    sig_builder.add_claim("sub", config.subject.as_str());

    let signature = sig_builder
        .build()
        .map_err(|e| PushError::VapidError(format!("Failed to build VAPID signature: {:?}", e)))?;

    // Build the Web Push message
    let mut message_builder = WebPushMessageBuilder::new(&subscription_info);
    message_builder.set_payload(
        web_push::ContentEncoding::Aes128Gcm,
        payload_json.as_bytes(),
    );
    message_builder.set_vapid_signature(signature);

    let message = message_builder
        .build()
        .map_err(|e| PushError::MessageBuildError(format!("Failed to build message: {:?}", e)))?;

    // Create HTTP client and send
    let client = IsahcWebPushClient::new()
        .map_err(|e| PushError::ClientError(format!("Failed to create client: {:?}", e)))?;

    // Send the notification
    match client.send(message).await {
        Ok(()) => {
            tracing::info!(
                "Web Push notification sent successfully to user_id={}",
                subscription.user_id
            );
            Ok(())
        }
        Err(web_push::WebPushError::EndpointNotValid(_)) => {
            // Endpoint no longer valid (410 Gone) - subscription should be deleted
            tracing::warn!(
                "Web Push endpoint invalid (410 Gone) for user_id={}",
                subscription.user_id
            );
            Err(PushError::EndpointInvalid)
        }
        Err(web_push::WebPushError::EndpointNotFound(_)) => {
            // Endpoint not found (404) - subscription should be deleted
            tracing::warn!(
                "Web Push endpoint not found (404) for user_id={}",
                subscription.user_id
            );
            Err(PushError::EndpointInvalid)
        }
        Err(web_push::WebPushError::ServerError {
            retry_after: Some(retry_after),
            ..
        }) => {
            // Server error with retry-after duration (could be 429 or 5xx)
            tracing::warn!(
                "Web Push server error for user_id={}, retry after: {:?}",
                subscription.user_id,
                retry_after
            );
            Err(PushError::RateLimited)
        }
        Err(web_push::WebPushError::ServerError {
            retry_after: None, ..
        }) => {
            // Server error without retry-after
            tracing::error!("Web Push server error for user_id={}", subscription.user_id);
            Err(PushError::ServerError(
                "Server error without retry-after".to_string(),
            ))
        }
        Err(e) => {
            tracing::error!(
                "Web Push error for user_id={}: {:?}",
                subscription.user_id,
                e
            );
            Err(PushError::SendError(format!("{:?}", e)))
        }
    }
}

/// Error types for Web Push operations
#[derive(Debug, thiserror::Error)]
pub enum PushError {
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("VAPID signature error: {0}")]
    VapidError(String),

    #[error("Client error: {0}")]
    ClientError(String),

    #[error("Message build error: {0}")]
    MessageBuildError(String),

    #[error("Send error: {0}")]
    SendError(String),

    #[error("Endpoint invalid (410 Gone) - subscription should be deleted")]
    EndpointInvalid,

    #[error("Rate limited (429 Too Many Requests)")]
    RateLimited,

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}
