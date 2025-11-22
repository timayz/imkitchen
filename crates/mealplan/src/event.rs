use bincode::{Decode, Encode};
use evento::AggregatorName;

use crate::Slot;

#[derive(AggregatorName, Encode, Decode)]
pub struct GenerateRequested;

#[derive(AggregatorName, Encode, Decode)]
pub struct WeekGenerated {
    pub week: u64,
    pub slots: Vec<Slot>,
}
