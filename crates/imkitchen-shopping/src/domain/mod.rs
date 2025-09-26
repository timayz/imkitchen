// Shopping list domain logic

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Shopping list aggregate root
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ShoppingList {
    pub shopping_list_id: Uuid,
    pub meal_plan_id: Uuid,
    pub user_id: Uuid,
    #[validate(length(min = 1))]
    pub items: Vec<ShoppingItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingItem {
    pub ingredient_name: String,
    pub quantity: String,
    pub unit: String,
    pub is_checked: bool,
}