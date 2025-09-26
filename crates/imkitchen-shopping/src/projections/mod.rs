// Evento projections for shopping views

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::ShoppingItem;

/// Read model for shopping list view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingListView {
    pub shopping_list_id: Uuid,
    pub meal_plan_id: Uuid,
    pub user_id: Uuid,
    pub items: Vec<ShoppingItem>,
    pub created_at: DateTime<Utc>,
}