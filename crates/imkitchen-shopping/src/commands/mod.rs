// CQRS commands for shopping operations

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GenerateShoppingListCommand {
    pub meal_plan_id: Uuid,
    pub user_id: Uuid,
}