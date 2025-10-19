use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::events::{
    PrepTaskCompleted, PushSubscriptionCreated, ReminderDismissed, ReminderScheduled, ReminderSent,
    ReminderSnoozed,
};

/// NotificationAggregate representing the state of a notification/reminder entity
///
/// This aggregate is rebuilt from events using the evento event sourcing framework.
/// It stores the complete state of a single notification including scheduling and delivery status.
///
/// Note: All fields are String types for bincode compatibility (follows evento best practices)
#[derive(Debug, Default, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct NotificationAggregate {
    // Core identity
    pub notification_id: String,
    pub user_id: String,
    pub recipe_id: String,

    // Scheduling metadata
    pub meal_date: String,         // ISO 8601 date
    pub scheduled_time: String,    // RFC3339 timestamp when reminder should fire
    pub reminder_type: String,     // "advance_prep", "morning", "day_of"
    pub prep_hours: i32,           // Hours of advance prep required
    pub prep_task: Option<String>, // "marinate", "rise", "chill", etc.

    // Delivery status
    pub status: String, // "pending", "sent", "failed", "dismissed", "snoozed", "completed"
    pub sent_at: Option<String>,
    pub delivery_status: Option<String>,
    pub dismissed_at: Option<String>,
    pub snoozed_until: Option<String>, // RFC3339 timestamp when snoozed notification should refire
    pub completed_at: Option<String>,  // RFC3339 timestamp when prep task was completed
}

/// PushSubscriptionAggregate representing a user's Web Push subscription
///
/// This is a separate aggregate from NotificationAggregate because subscriptions
/// have independent lifecycle from individual notifications.
#[derive(Debug, Default, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct PushSubscriptionAggregate {
    pub subscription_id: String,
    pub user_id: String,
    pub endpoint: String,
    pub p256dh_key: String,
    pub auth_key: String,
    pub created_at: String,
}

/// Implement evento aggregator pattern for NotificationAggregate
///
/// The #[evento::aggregator] macro generates:
/// - Aggregator trait implementation with event dispatching
/// - AggregatorName trait implementation
/// - Event replay functionality
#[evento::aggregator]
impl NotificationAggregate {
    /// Handle ReminderScheduled event to initialize aggregate state
    ///
    /// This is called when replaying events from the event store to rebuild
    /// the aggregate's current state.
    async fn reminder_scheduled(
        &mut self,
        event: evento::EventDetails<ReminderScheduled>,
    ) -> anyhow::Result<()> {
        self.notification_id = event.aggregator_id.clone();
        self.user_id = event.data.user_id;
        self.recipe_id = event.data.recipe_id;
        self.meal_date = event.data.meal_date;
        self.scheduled_time = event.data.scheduled_time;
        self.reminder_type = event.data.reminder_type;
        self.prep_hours = event.data.prep_hours;
        self.prep_task = event.data.prep_task;
        self.status = "pending".to_string();
        Ok(())
    }

    /// Handle ReminderSent event to update delivery status
    ///
    /// This is called when the background worker successfully delivers a notification.
    async fn reminder_sent(
        &mut self,
        event: evento::EventDetails<ReminderSent>,
    ) -> anyhow::Result<()> {
        self.sent_at = Some(event.data.sent_at);
        self.delivery_status = Some(event.data.delivery_status.clone());

        // Update status based on delivery result
        if event.data.delivery_status == "sent" {
            self.status = "sent".to_string();
        } else {
            self.status = "failed".to_string();
        }
        Ok(())
    }

    /// Handle ReminderDismissed event to mark notification as complete
    ///
    /// This is called when the user dismisses a notification.
    async fn reminder_dismissed(
        &mut self,
        event: evento::EventDetails<ReminderDismissed>,
    ) -> anyhow::Result<()> {
        self.dismissed_at = Some(event.data.dismissed_at);
        self.status = "dismissed".to_string();
        Ok(())
    }

    /// Handle ReminderSnoozed event to update scheduled time
    ///
    /// This is called when the user snoozes a notification.
    async fn reminder_snoozed(
        &mut self,
        event: evento::EventDetails<ReminderSnoozed>,
    ) -> anyhow::Result<()> {
        self.snoozed_until = Some(event.data.snoozed_until.clone());
        self.scheduled_time = event.data.snoozed_until;
        // Mark as snoozed so UI can distinguish from regular pending
        self.status = "snoozed".to_string();
        Ok(())
    }

    /// Handle PrepTaskCompleted event to mark prep task as complete
    ///
    /// This is called when the user marks a prep task as complete.
    /// Distinguishes from dismiss: completed means user finished the task.
    async fn prep_task_completed(
        &mut self,
        event: evento::EventDetails<PrepTaskCompleted>,
    ) -> anyhow::Result<()> {
        self.completed_at = Some(event.data.completed_at);
        self.status = "completed".to_string();
        Ok(())
    }
}

/// Implement evento aggregator pattern for PushSubscriptionAggregate
#[evento::aggregator]
impl PushSubscriptionAggregate {
    /// Handle PushSubscriptionCreated event to initialize aggregate state
    async fn push_subscription_created(
        &mut self,
        event: evento::EventDetails<PushSubscriptionCreated>,
    ) -> anyhow::Result<()> {
        self.subscription_id = event.aggregator_id.clone();
        self.user_id = event.data.user_id;
        self.endpoint = event.data.endpoint;
        self.p256dh_key = event.data.p256dh_key;
        self.auth_key = event.data.auth_key;
        self.created_at = event.data.created_at;
        Ok(())
    }
}
