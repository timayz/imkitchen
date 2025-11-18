use bincode::{Decode, Encode};
use evento::AggregatorName;
use serde::Deserialize;
use strum::{Display, EnumString, VariantArray};

#[derive(
    Encode, Decode, EnumString, Display, VariantArray, Default, Clone, Debug, PartialEq, Deserialize,
)]
pub enum RecipeType {
    Appetizer,
    #[default]
    MainCourse,
    Dessert,
    Accompaniment,
}

#[derive(Encode, Decode, Clone, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub quantity: u16,
    pub unit: String,
}

#[derive(Encode, Decode, Clone, Deserialize)]
pub struct Instruction {
    pub description: String,
    pub time_next: u16,
}

#[derive(
    Encode, Decode, EnumString, VariantArray, Display, Clone, Debug, Default, PartialEq, Deserialize,
)]
pub enum CuisineType {
    American,
    #[default]
    Caribbean,
    Chinese,
    Italian,
    French,
    Indian,
    Japanese,
    Mediterranean,
    Mexican,
    Thai,
}

#[derive(
    Encode, Decode, EnumString, VariantArray, Display, PartialEq, Clone, Debug, Deserialize,
)]
pub enum DietaryRestriction {
    Vegetarian,
    Vegan,
    GlutenFree,
    DairyFree,
    NutFree,
    LowCarb,
}

impl DietaryRestriction {
    pub fn exists_in<'a>(
        &self,
        iterator: impl IntoIterator<Item = &'a DietaryRestriction>,
    ) -> bool {
        iterator.into_iter().any(|d| d == self)
    }
}

#[derive(
    Encode, Decode, EnumString, Display, VariantArray, PartialEq, Clone, Debug, Deserialize,
)]
pub enum AccompanimentType {
    Rice,
    Pasta,
    Bread,
    Fries,
    Salad,
    Vegetables,
}

impl AccompanimentType {
    pub fn exists_in<'a>(&self, iterator: impl IntoIterator<Item = &'a AccompanimentType>) -> bool {
        iterator.into_iter().any(|d| d == self)
    }
}

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
