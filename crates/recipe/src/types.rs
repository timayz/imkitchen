use bincode::{Decode, Encode};
use serde::Deserialize;
use strum::{AsRefStr, Display, EnumString, VariantArray};

#[derive(
    Encode,
    Decode,
    EnumString,
    Display,
    VariantArray,
    Default,
    Clone,
    Debug,
    PartialEq,
    Deserialize,
    AsRefStr,
)]
pub enum RecipeType {
    Appetizer,
    #[default]
    MainCourse,
    Dessert,
    Accompaniment,
}

#[derive(
    Encode,
    Decode,
    EnumString,
    Display,
    VariantArray,
    Default,
    Clone,
    Debug,
    PartialEq,
    Deserialize,
    AsRefStr,
)]
pub enum IngredientUnit {
    #[default]
    G,
    ML,
}

#[derive(Encode, Decode, Clone, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub quantity: u16,
    pub unit: Option<IngredientUnit>,
}

#[derive(Encode, Decode, Clone, Deserialize)]
pub struct Instruction {
    pub description: String,
    pub time_next: u16,
}

#[derive(
    Encode,
    Decode,
    EnumString,
    VariantArray,
    Display,
    Clone,
    Debug,
    Default,
    PartialEq,
    Deserialize,
    AsRefStr,
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
    Encode,
    Decode,
    EnumString,
    VariantArray,
    Display,
    PartialEq,
    Clone,
    Debug,
    Deserialize,
    AsRefStr,
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

#[derive(Default, Debug, Deserialize, EnumString, Display, Clone)]
pub enum SortBy {
    #[default]
    RecentlyAdded,
}
