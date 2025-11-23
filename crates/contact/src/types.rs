use bincode::{Decode, Encode};
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
    #[serde(rename = "General Inquiry")]
    #[strum(serialize = "General Inquiry")]
    GeneralInquiry,
    #[serde(rename = "Technical Support")]
    #[strum(serialize = "Technical Support")]
    TechnicalSupport,
    #[serde(rename = "Billing Question")]
    #[strum(serialize = "Billing Question")]
    BillingQuestion,
    #[serde(rename = "Feature Request")]
    #[strum(serialize = "Feature Request")]
    FeatureRequest,
    #[serde(rename = "Bug Report")]
    #[strum(serialize = "Bug Report")]
    BugReport,
    #[serde(rename = "Partnership Opportunity")]
    #[strum(serialize = "Partnership Opportunity")]
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
pub enum SortBy {
    #[default]
    #[serde(rename = "Most Recent")]
    #[strum(serialize = "Most Recent")]
    MostRecent,
    #[serde(rename = "Oldest First")]
    #[strum(serialize = "Oldest First")]
    OldestFirst,
}
