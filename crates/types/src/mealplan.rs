//! Legacy calendar meal plan — **no longer written**.
//!
//! The meal plan was replaced by the per-user recipe list (see
//! [`crate::shopping`]). These declarations stay so the events already in the
//! store keep decoding and `events.lock` stays append-only: bitcode is
//! positional, so nothing here may change shape.

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
    pub date: u64,
    pub household_size: u16,
    pub appetizer: Option<SlotRecipe>,
    pub main_course: SlotRecipe,
    pub accompaniment: Option<SlotRecipe>,
    pub dessert: Option<SlotRecipe>,
    pub beverage: Option<SlotRecipe>,
    pub condiment: Option<SlotRecipe>,
}

#[derive(
    Encode, Decode, EnumString, Display, AsRefStr, Clone, Debug, Default, PartialEq, Deserialize,
)]
pub enum DaySlotStatus {
    #[default]
    Idle,
    Cooking(u8),
    Completed,
}

/// Never used by the app. Kept because the `events.lock` scanner resolves
/// nested type names by bare identifier, and this `Status` is part of the
/// recorded shape of the `imkitchen-core/contact/Contact` view.
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

#[evento::aggregate]
pub enum MealPlan {
    DaysGenerated {
        start: u64,
        slots: Vec<Slot>,
        household_size: u16,
    },

    SlotRecipeStatusChanged {
        date: u64,
        recipe_id: String,
        status: DaySlotStatus,
    },
}
