use bitcode::{Decode, Encode};
use serde::Deserialize;
use strum::{AsRefStr, Display, EnumString, VariantArray};

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub struct SlotRecipe {
    pub id: String,
    pub name: String,
}

#[derive(Encode, Decode, Clone, PartialEq, Debug)]
pub struct Slot {
    pub day: u64,
    pub appetizer: Option<SlotRecipe>,
    pub main_course: SlotRecipe,
    pub accompaniment: Option<SlotRecipe>,
    pub dessert: Option<SlotRecipe>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct DaySlotRecipe {
    pub id: String,
    pub name: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub advance_prep: String,
}

impl DaySlotRecipe {
    pub fn total_prep_time(&self) -> u16 {
        self.prep_time + self.cook_time
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

#[evento::aggregator]
pub enum MealPlan {
    WeekGenerated {
        start: u64,
        end: u64,
        slots: Vec<Slot>,
        household_size: u16,
    },
}
