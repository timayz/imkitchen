use bincode::{Decode, Encode};
use serde::Deserialize;
use strum::{AsRefStr, Display, EnumString, VariantArray};

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
pub enum Role {
    #[default]
    User,
    Admin,
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
pub enum State {
    #[default]
    Active,
    Suspended,
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

#[derive(Debug, Deserialize)]
pub enum UserSortBy {
    RecentlyJoined,
    Name,
    MostRecipes,
    MostActive,
}
