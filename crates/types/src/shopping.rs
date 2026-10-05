use crate::recipe::Ingredient;
use bitcode::{Decode, Encode};
use serde::Deserialize;
use strum::{AsRefStr, Display, EnumString};

/// Cooking cursor of one recipe in the user's list. `Idle` is the ingredient
/// screen, `Cooking(n)` instruction `n`, and `Completed` means the recipe has
/// been cooked (it stays in the list, marked, until removed or regenerated).
#[derive(
    Encode, Decode, EnumString, Display, AsRefStr, Clone, Debug, Default, PartialEq, Deserialize,
)]
pub enum RecipeStatus {
    #[default]
    Idle,
    Cooking(u8),
    Completed,
}

impl RecipeStatus {
    pub fn is_idle(&self) -> bool {
        matches!(self, RecipeStatus::Idle)
    }

    pub fn is_cooking(&self) -> bool {
        matches!(self, RecipeStatus::Cooking(_))
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, RecipeStatus::Completed)
    }
}

#[evento::aggregate]
pub enum Shopping {
    Checked {
        ingredient: String,
    },
    Unchecked {
        ingredient: String,
    },
    /// Legacy: date-ranged generation from the meal plan. No longer written.
    Generated {
        ingredients: Vec<Ingredient>,
        from_date: u64,
        days: u8,
    },
    /// Legacy companion of `Generated`. No longer written.
    RecipeSetGenerated {
        recipe_ids: Vec<String>,
    },
    RecipeAdded {
        recipe_id: String,
        recipe_ids: Vec<String>,
        ingredients: Vec<Ingredient>,
    },
    RecipeRemoved {
        recipe_id: String,
        recipe_ids: Vec<String>,
        ingredients: Vec<Ingredient>,
    },
    /// The whole list was replaced by generation: checks and cooking statuses
    /// reset.
    ListGenerated {
        recipe_ids: Vec<String>,
        ingredients: Vec<Ingredient>,
    },
    RecipeStatusChanged {
        recipe_id: String,
        status: RecipeStatus,
    },
}
