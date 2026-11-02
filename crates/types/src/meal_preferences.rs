use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

use crate::recipe::{DietaryRestriction, IngredientCategory, RecipeType};

/// Where the meals of a generated list come from. A sum type rather than two
/// booleans so that "neither" — a list with nothing to build meals from — is
/// unrepresentable in storage.
#[derive(
    Encode,
    Decode,
    EnumString,
    Display,
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Deserialize,
    Serialize,
    AsRefStr,
)]
pub enum MealSources {
    /// The user's dishes first, random main courses for the remaining meals.
    /// The behaviour from before this preference existed.
    #[default]
    DishesAndMainCourses,
    /// Only the user's dishes, as composed.
    Dishes,
    /// Only random main courses; dishes are ignored.
    MainCourses,
}

impl MealSources {
    /// Parse the two settings toggles; `None` when both are off.
    pub fn from_flags(dishes: bool, main_courses: bool) -> Option<Self> {
        match (dishes, main_courses) {
            (true, true) => Some(Self::DishesAndMainCourses),
            (true, false) => Some(Self::Dishes),
            (false, true) => Some(Self::MainCourses),
            (false, false) => None,
        }
    }

    pub fn dishes(&self) -> bool {
        matches!(self, Self::DishesAndMainCourses | Self::Dishes)
    }

    pub fn main_courses(&self) -> bool {
        matches!(self, Self::DishesAndMainCourses | Self::MainCourses)
    }
}

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

    /// The order aisles appear in on the groceries page. Its own variant for
    /// the same reason as `RecipeTypesChanged`.
    AisleOrderChanged { aisles: Vec<IngredientCategory> },

    /// Whether generated meals come from the user's dishes, from random main
    /// courses, or both. Its own variant for the same reason as
    /// `RecipeTypesChanged`.
    MealSourcesChanged { sources: MealSources },
}

#[cfg(test)]
mod tests {
    use super::MealSources;

    #[test]
    fn flags_round_trip_and_neither_is_rejected() {
        for sources in [
            MealSources::DishesAndMainCourses,
            MealSources::Dishes,
            MealSources::MainCourses,
        ] {
            assert_eq!(
                MealSources::from_flags(sources.dishes(), sources.main_courses()),
                Some(sources)
            );
        }
        assert_eq!(MealSources::from_flags(false, false), None);
    }
}
