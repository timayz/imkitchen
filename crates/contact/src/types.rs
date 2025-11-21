#[derive(Debug, Deserialize)]
pub enum Subject {
    GeneralInquiry,
    TechnicalSupport,
    BillingQuestion,
    FeatureRequest,
    BugReport,
    PartnershipOpportunity,
    Other,
}

#[derive(Debug, Deserialize)]
pub enum Status {
    Unread,
    Read,
    Resolved,
}
