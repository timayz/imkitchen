// Evento projections for meal planning views

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::MealType;

/// Read model for meal plan calendar view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealPlanCalendarView {
    pub meal_plan_id: Uuid,
    pub user_id: Uuid,
    pub week_start_date: NaiveDate,
    pub meal_slots: Vec<MealSlotView>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealSlotView {
    pub day_of_week: u8,
    pub meal_type: MealType,
    pub recipe_id: Option<Uuid>,
    pub recipe_title: Option<String>,
}