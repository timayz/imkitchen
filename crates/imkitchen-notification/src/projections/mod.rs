// Evento projections for notification views

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::NotificationType;

/// Read model for notification list view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationListView {
    pub notification_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub scheduled_for: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}