use bincode::{Decode, Encode};
use evento::AggregatorName;

#[derive(AggregatorName, Encode, Decode)]
pub struct FormSubmitted {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
    pub status: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct MarkedReadAndReply {
    pub status: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Resolved {
    pub status: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Reopened {
    pub status: String,
}
