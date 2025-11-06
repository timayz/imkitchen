use bincode::{Decode, Encode};
use evento::AggregatorName;

#[derive(AggregatorName, Encode, Decode)]
pub struct Created {
    pub household_size: u8,
    pub dietary_restrictions: Vec<String>,
    pub cuisine_variety_weight: f32,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Updated {
    pub household_size: u8,
    pub dietary_restrictions: Vec<String>,
    pub cuisine_variety_weight: f32,
}
