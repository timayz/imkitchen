// Evento projections for recipe views

use chrono::{DateTime, Utc};
use imkitchen_shared::Difficulty;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Read model for recipe list view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeListView {
    pub recipe_id: Uuid,
    pub title: String,
    pub prep_time_minutes: u32,
    pub cook_time_minutes: u32,
    pub difficulty: Difficulty,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}