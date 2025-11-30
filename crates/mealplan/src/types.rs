use bincode::{Decode, Encode};
use imkitchen_recipe::{Ingredient, Instruction};
use serde::Deserialize;
use strum::{AsRefStr, Display, EnumString, VariantArray};

#[derive(Encode, Decode)]
pub struct SlotRecipe {
    pub id: String,
    pub name: String,
}

#[derive(Encode, Decode)]
pub struct Slot {
    pub day: u64,
    pub appetizer: Option<SlotRecipe>,
    pub main_course: SlotRecipe,
    pub accompaniment: Option<SlotRecipe>,
    pub dessert: Option<SlotRecipe>,
}

#[derive(Encode, Decode, Default)]
pub struct DaySlotRecipe {
    pub id: String,
    pub name: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
    pub advance_prep: String,
}

impl DaySlotRecipe {
    pub fn total_prep_time(&self) -> u32 {
        (self.prep_time + self.cook_time).into()
    }
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
