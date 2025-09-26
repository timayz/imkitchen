// Meal planning domain events

use chrono::{DateTime, NaiveDate, Utc};
use imkitchen_shared::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealPlanGenerated {
    pub event_id: Uuid,
    pub meal_plan_id: Uuid,
    pub user_id: Uuid,
    pub week_start_date: NaiveDate,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for MealPlanGenerated {
    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.meal_plan_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn event_type(&self) -> &'static str {
        "MealPlanGenerated"
    }
}
