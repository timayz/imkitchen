#[evento::aggregate]
pub enum UserProfile {
    Changed { description: String },
}
