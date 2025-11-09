use bincode::{Decode, Encode};
use evento::AggregatorName;

#[derive(AggregatorName, Encode, Decode)]
pub struct LifePremiumToggled {
    pub expire_at: u64,
}
