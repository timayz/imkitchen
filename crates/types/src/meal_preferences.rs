use crate::recipe::{DietaryRestriction, RecipeType};

#[evento::aggregate]
pub enum MealPreferences {
    Changed {
        household_size: u16,
        dietary_restrictions: Vec<DietaryRestriction>,
        cuisine_variety_weight: f32,
    },

    /// Which optional `RecipeType`s to include when generating a meal plan.
    /// A separate variant rather than a field on `Changed`: bitcode is not
    /// schema-evolving, so extending `Changed` would break decoding of every
    /// event already in the log.
    RecipeTypesChanged { recipe_types: Vec<RecipeType> },
}
