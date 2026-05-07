#[evento::aggregator]
pub enum UserProfile {
    Changed { description: String },
}
