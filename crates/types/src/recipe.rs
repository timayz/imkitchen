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
    sqlx::Type,
)]
pub enum RecipeType {
    Appetizer,
    #[default]
    MainCourse,
    Dessert,
    Accompaniment,
    Beverage,
    Condiment,
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

    pub fn json_key(&self) -> String {
        serde_json::json!({
            "name": self.key()
        })
        .to_string()
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
    sqlx::Type,
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
}

impl DietaryRestriction {
    pub fn exists_in<'a>(
        &self,
        iterator: impl IntoIterator<Item = &'a DietaryRestriction>,
    ) -> bool {
        iterator.into_iter().any(|d| d == self)
    }
}

#[evento::aggregate]
pub enum Recipe {
    Created {
        name: String,
        owner_name: Option<String>,
    },

    Imported {
        name: String,
        owner_name: Option<String>,
        origin: Option<String>,
        description: String,
        recipe_type: RecipeType,
        cuisine_type: CuisineType,
        household_size: u16,
        prep_time: u16,
        cook_time: u16,
        ingredients: Vec<Ingredient>,
        instructions: Vec<Instruction>,
        advance_prep: String,
        dietary_restrictions: Vec<DietaryRestriction>,
        accepts_accompaniment: bool,
    },

    RecipeTypeChanged {
        recipe_type: RecipeType,
    },

    BasicInformationChanged {
        name: String,
        origin: Option<String>,
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

    // Byte-free trigger. The original upload bytes are stashed transiently in
    // `recipe_thumbnail` (device='original') by the upload command and consumed
    // by the resize subscription, so no image bytes ever enter the event log.
    ThumbnailUploaded,

    // Byte-free marker. The resized variant bytes are written directly to
    // `recipe_thumbnail` (the authoritative image store); this event only
    // signals which device variant was (re)generated so downstream version and
    // blur-placeholder projections can react.
    ThumbnailResized {
        device: String,
    },

    MadePrivate,
    Deleted,
}

#[cfg(test)]
mod tests {
    use super::{ThumbnailResized, ThumbnailUploaded};

    // The m0009 data migration strips image bytes out of existing thumbnail
    // event blobs with pure SQL, relying on the fact that the new byte-free
    // bitcode encoding is a *prefix* of the old one: a unit `ThumbnailUploaded`
    // encodes to an empty blob, and `ThumbnailResized { device }` encodes to
    // just the length-prefixed device string. If a bitcode upgrade ever changes
    // this layout, the migration's byte-slicing would silently corrupt data, so
    // pin the exact bytes here.
    #[test]
    fn thumbnail_uploaded_encodes_empty() {
        assert!(bitcode::encode(&ThumbnailUploaded).is_empty());
    }

    #[test]
    fn thumbnail_resized_encodes_to_device_prefix() {
        // 0x06 = len("mobile"), then the utf8 bytes — matches substr(data,1,7).
        assert_eq!(
            bitcode::encode(&ThumbnailResized {
                device: "mobile".to_string(),
            }),
            b"\x06mobile",
        );
        assert_eq!(
            bitcode::encode(&ThumbnailResized {
                device: "tablet".to_string(),
            }),
            b"\x06tablet",
        );
        // 0x07 = len("desktop") — matches substr(data,1,8).
        assert_eq!(
            bitcode::encode(&ThumbnailResized {
                device: "desktop".to_string(),
            }),
            b"\x07desktop",
        );
    }

    #[test]
    fn stripped_resized_blob_decodes() {
        let decoded: ThumbnailResized = bitcode::decode(b"\x06mobile").unwrap();
        assert_eq!(decoded.device, "mobile");
    }
}
