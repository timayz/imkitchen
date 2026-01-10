pub mod meal_preferences;
pub mod password;
pub mod subscription;

use bitcode::{Decode, Encode};
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

#[evento::aggregator]
pub enum User {
    Registered {
        email: String,
        lang: String,
        timezone: String,
    },
    LoggedIn {
        access_id: String,
        lang: String,
        timezone: String,
        user_agent: String,
    },
    UsernameChanged {
        value: String,
    },
    Logout {
        access_id: String,
    },
    MadeAdmin,
    Suspended,
    Activated,
}
