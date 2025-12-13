use bincode::{Decode, Encode};
use evento::AggregatorName;

#[derive(AggregatorName, Encode, Decode)]
pub struct ResetRequested {
    pub email: String,
    pub lang: String,
    pub host: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Resetted {
    pub password_hash: String,
}
