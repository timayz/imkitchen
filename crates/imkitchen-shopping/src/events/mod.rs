// Shopping domain events

use chrono::{DateTime, Utc};
use imkitchen_shared::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingListGenerated {
    pub event_id: Uuid,
    pub shopping_list_id: Uuid,
    pub meal_plan_id: Uuid,
    pub user_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for ShoppingListGenerated {
    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.shopping_list_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn event_type(&self) -> &'static str {
        "ShoppingListGenerated"
    }
}
