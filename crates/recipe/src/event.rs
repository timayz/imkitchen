use bincode::{Decode, Encode};
use evento::AggregatorName;
use strum::{AsRefStr, Display, EnumString};

#[derive(Encode, Decode, EnumString, AsRefStr, Display)]
pub enum RecipeType {
    Appetizer,
    MainCourse,
    Dessert,
    Accompaniment,
}

#[derive(Encode, Decode)]
pub struct Ingredient {
    pub name: String,
    pub unit: String,
    pub unit_type: String,
}

#[derive(Encode, Decode)]
pub struct Instruction {
    pub description: String,
}

#[derive(Encode, Decode, EnumString, AsRefStr, Display)]
pub enum CuisineType {
    Italian,
    Thai,
    Chinese,
    Japanese,
    Mexican,
    Indian,
    French,
    American,
    Mediterranean,
    Caribbean,
    Custom(String),
}

#[derive(Encode, Decode, EnumString, AsRefStr, Display)]
pub enum DietaryRestriction {
    Vegetarian,
    Vegan,
    GlutenFree,
    DairyFree,
    NutFree,
    LowCarb,
}

#[derive(Encode, Decode, EnumString, AsRefStr, Display)]
pub enum AccompanimentType {
    Rice,
    Pasta,
    Bread,
    Fries,
    Salad,
    Vegetables,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Created {
    pub name: String,
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
    pub accept_accompaniments: bool,
    pub preferred_accompaniment_types: Vec<AccompanimentType>,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct AdvandePreparationChanged {
    pub description: String,
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
