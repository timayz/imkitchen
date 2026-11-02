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
    /// A whole dish was added by hand: `recipe_ids` is the resulting list.
    DishAdded {
        dish_id: String,
        recipe_ids: Vec<String>,
        ingredients: Vec<Ingredient>,
    },
    /// The household size chosen for these recipes when they were generated
    /// or added. A separate variant rather than a field on `ListGenerated` /
    /// `RecipeAdded` / `DishAdded` because bitcode is positional: existing
    /// variants must never change shape (see `MealPreferences::RecipeTypesChanged`).
    ServingsSet {
        recipe_ids: Vec<String>,
        household_size: u16,
    },
    /// The recipes of one composed dish that made it into the list, in meal
    /// order: the kitchen shows them as one row. Follows `ListGenerated` (one
    /// per dish meal, the dish's own recipes before random courses are filled
    /// in) or `DishAdded`. A recipe belongs to at most one group.
    DishPlaced {
        dish_id: String,
        recipe_ids: Vec<String>,
    },
    /// A whole dish group left the list: `recipe_ids` is the resulting list,
    /// like `RecipeRemoved`.
    DishRemoved {
        dish_id: String,
        recipe_ids: Vec<String>,
        ingredients: Vec<Ingredient>,
    },
}
