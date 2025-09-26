// Meal planning domain logic

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Meal plan aggregate root
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MealPlan {
    pub meal_plan_id: Uuid,
    pub user_id: Uuid,
    pub week_start_date: NaiveDate,
    #[validate(length(min = 21, max = 21))] // 7 days × 3 meals
    pub meal_slots: Vec<MealSlot>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealSlot {
    pub day_of_week: u8, // 0-6 (Monday to Sunday)
    pub meal_type: MealType,
    pub recipe_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}