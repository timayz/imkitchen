//! The recipes: each one a user's own. There is no sharing: nobody but its
//! owner reads a recipe or changes it.

use bitcode::{Decode, Encode};

use crate::image::RecipeImage;

/// One recipe of one user, under a fresh id (`evento::create`). Times are
/// minutes, 0 when the recipe says none.
#[evento::aggregate(name = "imkitchen/Recipe")]
pub enum Recipe {
    /// Written by `owner_id`, whose it stays.
    Created {
        owner_id: String,
        title: String,
        description: String,
        ingredients: Vec<Ingredient>,
        /// The steps, in the order they are made.
        instructions: Vec<Instruction>,
        prep_minutes: u32,
        cook_minutes: u32,
        servings: u32,
    },
    /// Rewritten by its owner: everything it says, as a whole.
    Edited {
        title: String,
        description: String,
        ingredients: Vec<Ingredient>,
        instructions: Vec<Instruction>,
        prep_minutes: u32,
        cook_minutes: u32,
        servings: u32,
    },
    /// Gone for good, its picture let go of.
    Deleted,
    /// Shown with a picture, in place of the one it had if it had one.
    ImageSet { image: RecipeImage },
    /// Shown without a picture again.
    ImageRemoved,
}

/// What a recipe takes of one thing. Persisted in events: its fields are
/// frozen as the variants are.
#[derive(Debug, Clone, PartialEq, Default, Encode, Decode)]
pub struct Ingredient {
    pub name: String,
    /// How much of it, in `unit`; `None` when the recipe does not say
    /// ("salt").
    pub quantity: Option<f64>,
    /// What `quantity` counts ("g", "tbsp"); `None` for the thing itself
    /// ("2 eggs").
    pub unit: Option<String>,
}

/// One step of a recipe. Persisted in events: its fields are frozen as the
/// variants are.
#[derive(Debug, Clone, PartialEq, Eq, Default, Encode, Decode)]
pub struct Instruction {
    pub text: String,
    /// The minutes to wait once the step is done ("bake 25 minutes"), what
    /// a timer is started from; `None` when there is nothing to wait for.
    pub timer_minutes: Option<u32>,
}
