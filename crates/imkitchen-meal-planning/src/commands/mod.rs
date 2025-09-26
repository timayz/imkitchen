// CQRS commands for meal planning operations

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GenerateMealPlanCommand {
    pub user_id: Uuid,
    pub week_start_date: NaiveDate,
}
