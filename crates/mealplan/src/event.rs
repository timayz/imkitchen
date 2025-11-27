use bincode::{Decode, Encode};
use evento::AggregatorName;

use crate::{Slot, Status};

#[derive(AggregatorName, Encode, Decode)]
pub struct GenerateRequested {
    pub status: Status,
    pub weeks: Vec<(u64, u64)>,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct GenerationFailed {
    pub status: Status,
    pub reason: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct WeekGenerated {
    pub start: u64,
    pub end: u64,
    pub status: Status,
    pub slots: Vec<Slot>,
}
