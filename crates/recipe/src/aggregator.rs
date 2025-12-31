use crate::{CuisineType, DietaryRestriction, Ingredient, Instruction, RecipeType};

#[evento::aggregator]
pub enum Recipe {
    Created {
        name: String,
        owner_name: Option<String>,
    },

    Imported {
        name: String,
        owner_name: Option<String>,
        description: String,
        recipe_type: RecipeType,
        cuisine_type: CuisineType,
        household_size: u16,
        prep_time: u16,
        cook_time: u16,
        ingredients: Vec<Ingredient>,
        instructions: Vec<Instruction>,
        advance_prep: String,
    },

    RecipeTypeChanged {
        recipe_type: RecipeType,
    },

    BasicInformationChanged {
        name: String,
        description: String,
        household_size: u16,
        prep_time: u16,
        cook_time: u16,
    },

    IngredientsChanged {
        ingredients: Vec<Ingredient>,
    },

    InstructionsChanged {
        instructions: Vec<Instruction>,
    },

    DietaryRestrictionsChanged {
        dietary_restrictions: Vec<DietaryRestriction>,
    },

    CuisineTypeChanged {
        cuisine_type: CuisineType,
    },

    MainCourseOptionsChanged {
        accepts_accompaniment: bool,
    },

    AdvancePrepChanged {
        advance_prep: String,
    },

    SharedToCommunity {
        owner_name: String,
    },
    MadePrivate,
    Deleted,
}
