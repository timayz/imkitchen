use bincode::{Decode, Encode};
use evento::AggregatorName;

#[derive(Encode, Decode)]
pub struct Slot;

#[derive(AggregatorName, Encode, Decode)]
pub struct GenerateRequested {
    pub week: u16,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Generated {
    pub slots: Vec<Slot>,
}
