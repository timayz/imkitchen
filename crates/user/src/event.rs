use bincode::{Decode, Encode};
use evento::{AggregatorName, EventDetails};
use ulid::Ulid;

#[derive(AggregatorName, Encode, Decode)]
pub struct RegistrationRequested {
    pub email: String,
    pub password_hash: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RegistrationSucceeded {
    pub email: String,
    pub password_hash: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct RegistrationFailed {
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

impl Metadata {
    pub fn new(
        trigger_by: impl Into<Option<String>>,
        trigger_as: impl Into<Option<String>>,
    ) -> Self {
        let trigger_by = trigger_by.into();
        let trigger_as = trigger_as.into();

        Self {
            id: Ulid::new().to_string(),
            trigger_by,
            trigger_as,
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new(None, None)
    }
}

pub type UserEvent<D> = EventDetails<D, Metadata>;
