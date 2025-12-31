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
)]
pub enum Subject {
    #[default]
    GeneralInquiry,
    TechnicalSupport,
    BillingQuestion,
    FeatureRequest,
    BugReport,
    PartnershipOpportunity,
    Other,
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
pub enum Status {
    #[default]
    Unread,
    Read,
    Resolved,
}
