// Notification domain logic

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Notification aggregate root
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Notification {
    pub notification_id: Uuid,
    pub user_id: Uuid,
    #[validate(length(min = 1))]
    pub title: String,
    #[validate(length(min = 1))]
    pub message: String,
    pub notification_type: NotificationType,
    pub scheduled_for: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    MealPrepReminder,
    ShoppingReminder,
    RecipeRecommendation,
}
