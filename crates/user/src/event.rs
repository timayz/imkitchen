use bincode::{Decode, Encode};
use evento::AggregatorName;

#[derive(AggregatorName, Encode, Decode)]
pub struct RegistrationRequested {
    pub email: String,
    pub password_hash: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RegistrationSucceeded {
    pub email: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RegistrationFailed {
    pub reason: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct LoggedIn {
    pub lang: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct MadeAdmin {
    pub role: String,
}
