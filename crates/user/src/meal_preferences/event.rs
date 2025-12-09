use bincode::{Decode, Encode};
use evento::AggregatorName;
use imkitchen_recipe::DietaryRestriction;

#[derive(AggregatorName, Encode, Decode)]
pub struct Created {
    pub household_size: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_variety_weight: f32,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Updated {
    pub household_size: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_variety_weight: f32,
}
