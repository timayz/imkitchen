use bincode::{Decode, Encode};
use serde::Deserialize;
use strum::{AsRefStr, Display, EnumString, VariantArray};

#[derive(Encode, Decode)]
pub struct SlotRecipe {
    pub id: String,
    pub name: String,
}

impl From<&(String, String)> for SlotRecipe {
    fn from(value: &(String, String)) -> Self {
        SlotRecipe {
            id: value.0.to_owned(),
            name: value.1.to_owned(),
        }
    }
}

#[derive(Encode, Decode)]
pub struct Slot {
    pub day: u64,
    pub appetizer: Option<SlotRecipe>,
    pub main_course: SlotRecipe,
    pub accompaniment: Option<SlotRecipe>,
    pub dessert: Option<SlotRecipe>,
}

#[derive(
    Encode,
    Decode,
    EnumString,
    VariantArray,
    Display,
    AsRefStr,
    Clone,
    Debug,
    Default,
    PartialEq,
    Deserialize,
)]
pub enum Status {
    #[default]
    Idle,
    Processing,
    Failed,
}
