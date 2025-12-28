#[evento::aggregator]
pub enum Subscription {
    LifePremiumToggled { expire_at: u64 },
}
