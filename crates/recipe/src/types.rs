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

pub trait IngredientUnitFormat {
    fn format(&self, value: u32) -> String;
}

impl IngredientUnitFormat for Option<IngredientUnit> {
    fn format(&self, value: u32) -> String {
        match self {
            Some(IngredientUnit::ML) => {
                if value >= 1000 {
                    let liters = value as f64 / 1000.0;
                    if liters.fract() == 0.0 {
                        format!("{} L", liters as u32)
                    } else {
                        format!("{} L", liters)
                    }
                } else {
                    format!("{} ml", value)
                }
            }
            Some(IngredientUnit::G) => {
                if value >= 1000 {
                    let kg = value as f64 / 1000.0;
                    if kg.fract() == 0.0 {
                        format!("{} kg", kg as u32)
                    } else {
                        format!("{} kg", kg)
                    }
                } else {
                    format!("{} g", value)
                }
            }
            None => format!("{}", value),
        }
    }
}

#[derive(Encode, Decode, Clone, Deserialize)]
pub struct Ingredient {
    pub name: String,
    pub quantity: u32,
    pub unit: Option<IngredientUnit>,
}

impl Ingredient {
    pub fn key(&self) -> String {
        format!("{}-{}", self.name, self.unit.format(0))
    }
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
