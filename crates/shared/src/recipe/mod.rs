pub mod rating;

use bitcode::{Decode, Encode};
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
pub enum IngredientCategory {
    /// Frozen products: frozen vegetables, ready meals, ice cream, frozen seafood
    #[default]
    Frozen,
    /// Fresh/refrigerated products: dairy, deli meats, prepared meals, fresh desserts
    Refrigerated,
    /// Grocery - savory: canned goods, appetizers, pasta/rice, sauces, oils, world foods
    Grocery,
    /// Fruits and vegetables: fresh fruits, fresh vegetables, herbs, organic produce
    FruitsAndVegetables,
    /// Butcher: red meat, white meat, ground meat, specialty items
    Butcher,
    /// Fishmonger: fresh fish, seafood, prepared seafood dishes, sushi
    Seafood,
    /// Dairy and eggs: milk, butter, cream, yogurt, eggs
    DairyAndEggs,
    /// Bakery: bread, pastries, cakes, gluten-free options
    Bakery,
    /// Snacks and confectionery: cookies, chocolate, chips, energy bars
    SnacksAndConfectionery,
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

#[derive(Encode, Decode, Clone, Deserialize, Debug, PartialEq)]
pub struct Ingredient {
    pub name: String,
    pub quantity: u32,
    pub unit: Option<IngredientUnit>,
    pub category: Option<IngredientCategory>,
}

impl Ingredient {
    pub fn key(&self) -> String {
        format!("{}-{}", self.name, self.unit.format(0))
    }
}

#[derive(Encode, Decode, Clone, Deserialize, Debug, PartialEq)]
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
