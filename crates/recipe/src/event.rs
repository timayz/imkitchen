use bincode::{Decode, Encode};
use evento::AggregatorName;

use crate::{
    AccompanimentType, CuisineType, DietaryRestriction, Ingredient, Instruction, RecipeType,
};

#[derive(AggregatorName, Encode, Decode)]
pub struct Created {
    pub name: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Imported {
    pub name: String,
    pub description: String,
    pub recipe_type: RecipeType,
    pub cuisine_type: CuisineType,
    pub prep_time: u16,
    pub cook_time: u16,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
    pub advance_prep: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RecipeTypeChanged {
    pub recipe_type: RecipeType,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct BasicInformationChanged {
    pub name: String,
    pub description: String,
    pub prep_time: u16,
    pub cook_time: u16,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct IngredientsChanged {
    pub ingredients: Vec<Ingredient>,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct InstructionsChanged {
    pub instructions: Vec<Instruction>,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct DietaryRestrictionsChanged {
    pub dietary_restrictions: Vec<DietaryRestriction>,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct CuisineTypeChanged {
    pub cuisine_type: CuisineType,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct MainCourseOptionsChanged {
    pub accepts_accompaniment: bool,
    pub preferred_accompaniment_types: Vec<AccompanimentType>,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct AdvancePrepChanged {
    pub advance_prep: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct SharedToCommunity {
    pub shared: bool,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct MadePrivate {
    pub shared: bool,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Deleted {
    pub deleted: bool,
}
