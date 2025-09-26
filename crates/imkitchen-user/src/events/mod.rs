// User domain events

use chrono::{DateTime, Utc};
use imkitchen_shared::{DomainEvent, FamilySize, SkillLevel};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegistered {
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub family_size: FamilySize,
    pub cooking_skill_level: SkillLevel,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for UserRegistered {
    fn event_id(&self) -> Uuid {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn event_type(&self) -> &'static str {
        "UserRegistered"
    }
}
