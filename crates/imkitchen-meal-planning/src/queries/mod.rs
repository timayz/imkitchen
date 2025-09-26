// CQRS queries for meal planning data

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealPlanByWeekQuery {
    pub user_id: Uuid,
    pub week_start_date: NaiveDate,
}
