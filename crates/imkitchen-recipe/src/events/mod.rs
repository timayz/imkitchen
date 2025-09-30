// Recipe domain events

use chrono::{DateTime, Utc};
use imkitchen_shared::{Difficulty, DomainEvent};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Ingredient, Instruction, RecipeCategory};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCreated {
    pub event_id: Uuid,
    pub recipe_id: Uuid,
    pub title: String,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
    pub prep_time_minutes: u32,
    pub cook_time_minutes: u32,
    pub difficulty: Difficulty,
    pub category: RecipeCategory,
    pub created_by: Uuid,
    pub is_public: bool,
    pub tags: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeUpdated {
    pub event_id: Uuid,
    pub recipe_id: Uuid,
    pub title: Option<String>,
    pub ingredients: Option<Vec<Ingredient>>,
    pub instructions: Option<Vec<Instruction>>,
    pub prep_time_minutes: Option<u32>,
    pub cook_time_minutes: Option<u32>,
    pub difficulty: Option<Difficulty>,
    pub category: Option<RecipeCategory>,
    pub is_public: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for RecipeUpdated {
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
        "RecipeUpdated"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDeleted {
    pub event_id: Uuid,
    pub recipe_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for RecipeDeleted {
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
        "RecipeDeleted"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientAdded {
    pub event_id: Uuid,
    pub recipe_id: Uuid,
    pub ingredient: Ingredient,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for IngredientAdded {
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
        "IngredientAdded"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionModified {
    pub event_id: Uuid,
    pub recipe_id: Uuid,
    pub step_number: u32,
    pub instruction: Instruction,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for InstructionModified {
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
        "InstructionModified"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeArchived {
    pub event_id: Uuid,
    pub recipe_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for RecipeArchived {
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
        "RecipeArchived"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeRestored {
    pub event_id: Uuid,
    pub recipe_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for RecipeRestored {
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
        "RecipeRestored"
    }
}
