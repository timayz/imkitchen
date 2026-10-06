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
    Chef,
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

#[evento::aggregate]
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
    EmailChanged {
        value: String,
    },
    Logout {
        access_id: String,
    },
    MadeAdmin,
    RoleChanged {
        role: Role,
    },
    Suspended,
    Activated,
    AdConsentGranted,
    AdConsentRevoked,
    /// The user deleted their account. Every projection drops the personal
    /// data it holds; the `user` row is removed in the same command so the
    /// email can be registered again.
    Deleted,
}
