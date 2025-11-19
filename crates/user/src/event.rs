use bincode::{Decode, Encode};
use evento::AggregatorName;
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
    Root,
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

#[derive(AggregatorName, Encode, Decode)]
pub struct RegistrationRequested {
    pub email: String,
    pub password_hash: String,
    pub status: Status,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RegistrationSucceeded {
    pub email: String,
    pub status: Status,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RegistrationFailed {
    pub reason: String,
    pub status: Status,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct LoggedIn {
    pub lang: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct MadeAdmin {
    pub role: Role,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Suspended {
    pub state: State,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Activated {
    pub state: State,
}
