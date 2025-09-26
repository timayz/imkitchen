// Notification domain events

use chrono::{DateTime, Utc};
use imkitchen_shared::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::NotificationType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationScheduled {
    pub event_id: Uuid,
    pub notification_id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub scheduled_for: DateTime<Utc>,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for NotificationScheduled {
    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.notification_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn event_type(&self) -> &'static str {
        "NotificationScheduled"
    }
}
