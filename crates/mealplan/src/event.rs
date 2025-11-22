use bincode::{Decode, Encode};
use evento::AggregatorName;

use crate::{Slot, Status};

#[derive(AggregatorName, Encode, Decode)]
pub struct GenerateRequested {
    pub status: Status,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct WeekGenerated {
    pub week: u64,
    pub status: Status,
    pub slots: Vec<Slot>,
}
