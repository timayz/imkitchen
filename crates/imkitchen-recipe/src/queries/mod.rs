// CQRS queries for recipe data

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeByIdQuery {
    pub recipe_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipesByUserQuery {
    pub user_id: Uuid,
}
