use bincode::{Decode, Encode};
use evento::{AggregatorName, EventDetails};

#[derive(AggregatorName, Encode, Decode)]
pub struct RegisterRequested {
    pub email: String,
    pub password_hash: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Registered {
    pub email: String,
    pub password_hash: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RegisterFailed {
    pub reason: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct LoggedIn {
    pub email: String,
    pub lang: String,
}

#[derive(Encode, Decode)]
pub struct Metadata {
    id: String,
    trigger_by: Option<String>,
    trigger_as: Option<String>,
}

pub type UserEvent<D> = EventDetails<D, Metadata>;
