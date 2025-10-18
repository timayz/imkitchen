use bincode::{Decode, Encode};
use evento::AggregatorName;
use serde::{Deserialize, Serialize};

/// ReminderScheduled event emitted when a preparation reminder is scheduled
///
/// This event is the source of truth for notification scheduling in the event sourced system.
/// Uses String types for bincode compatibility (UUID and timestamps serialized as strings).
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct ReminderScheduled {
    pub notification_id: String,     // UUID of the notification
    pub user_id: String,             // Owner of the reminder
    pub recipe_id: String,           // Recipe requiring advance prep
    pub meal_date: String,           // ISO 8601 date of the meal
    pub scheduled_time: String,      // RFC3339 formatted timestamp when reminder should fire
    pub reminder_type: String,       // "advance_prep", "morning", "day_of"
    pub prep_hours: i32,             // Hours of advance prep required (from recipe)
    pub prep_task: Option<String>,   // Specific task: "marinate", "rise", "chill", etc.
}

/// ReminderSent event emitted when a notification is delivered to the user
///
/// This event captures successful delivery via Web Push API or other channels.
/// Maintains full audit trail of notification delivery attempts.
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct ReminderSent {
    pub notification_id: String, // UUID of the notification
    pub sent_at: String,         // RFC3339 formatted timestamp
    pub delivery_status: String, // "sent", "failed", "endpoint_invalid"
}

/// ReminderDismissed event emitted when user dismisses a notification
///
/// User action to mark the reminder as acknowledged/completed.
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct ReminderDismissed {
    pub notification_id: String, // UUID of the notification
    pub dismissed_at: String,    // RFC3339 formatted timestamp
}

/// ReminderSnoozed event emitted when user snoozes a notification
///
/// User action to delay the reminder by a specified duration (1h, 2h, 4h).
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct ReminderSnoozed {
    pub notification_id: String, // UUID of the notification
    pub snoozed_until: String,   // RFC3339 formatted timestamp when reminder should refire
    pub snooze_duration_hours: i32, // Duration of snooze (1, 2, or 4)
}

/// PushSubscriptionCreated event emitted when user subscribes to Web Push notifications
///
/// This event stores the browser push subscription details for future notification delivery.
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct PushSubscriptionCreated {
    pub subscription_id: String, // UUID of the subscription
    pub user_id: String,         // Owner of the subscription
    pub endpoint: String,        // Web Push endpoint URL
    pub p256dh_key: String,      // Base64-encoded public key
    pub auth_key: String,        // Base64-encoded auth secret
    pub created_at: String,      // RFC3339 formatted timestamp
}
