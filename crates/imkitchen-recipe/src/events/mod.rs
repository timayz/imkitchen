// Recipe domain events

use chrono::{DateTime, Utc};
use imkitchen_shared::{Difficulty, DomainEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCreated {
    pub event_id: Uuid,
    pub recipe_id: Uuid,
    pub title: String,
    pub difficulty: Difficulty,
    pub created_by: Uuid,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for RecipeCreated {
    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.recipe_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn event_type(&self) -> &'static str {
        "RecipeCreated"
    }
}
