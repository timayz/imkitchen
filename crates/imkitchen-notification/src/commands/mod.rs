// CQRS commands for notification operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::domain::NotificationType;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ScheduleNotificationCommand {
    pub user_id: Uuid,
    #[validate(length(min = 1))]
    pub title: String,
    #[validate(length(min = 1))]
    pub message: String,
    pub notification_type: NotificationType,
    pub scheduled_for: DateTime<Utc>,
}