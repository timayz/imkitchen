// CQRS queries for notification data

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingNotificationsQuery {
    pub user_id: Uuid,
}