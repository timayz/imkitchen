use bincode::{Decode, Encode};
use evento::AggregatorName;
use imkitchen_recipe::Ingredient;

#[derive(AggregatorName, Encode, Decode)]
pub struct Checked {
    pub week: u64,
    pub ingredient: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Unchecked {
    pub week: u64,
    pub ingredient: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Resetted {
    pub week: u64,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Generated {
    pub week: u64,
    pub ingredients: Vec<Ingredient>,
}
