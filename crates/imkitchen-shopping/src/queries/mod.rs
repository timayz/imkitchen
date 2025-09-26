// CQRS queries for shopping data

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingListByMealPlanQuery {
    pub meal_plan_id: Uuid,
}