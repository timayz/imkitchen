use bincode::{Decode, Encode};
use serde::Deserialize;
use strum::{AsRefStr, Display, EnumString, VariantArray};

#[derive(Encode, Decode)]
pub struct Slot {
    pub appetizer_id: String,
    pub main_course_id: String,
    pub dessert_id: String,
    pub accompaniment_id: Option<String>,
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
