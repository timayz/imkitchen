use bincode::{Decode, Encode};
use evento::AggregatorName;

#[derive(AggregatorName, Encode, Decode)]
pub struct FormSubmitted {
    pub name: String,
    pub email: String,
    pub subject: Subject,
    pub message: String,
    pub status: Status,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct MarkedReadAndReply {
    pub status: Status,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Resolved {
    pub status: Status,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Reopened {
    pub status: Status,
}
