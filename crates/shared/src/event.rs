use bincode::{Decode, Encode};
use evento::EventDetails;
use ulid::Ulid;

#[derive(Encode, Decode, Clone)]
pub struct Metadata {
    pub id: String,
    pub trigger_by: Option<String>,
    pub trigger_as: Option<String>,
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

    pub fn by(trigger_by: impl Into<Option<String>>) -> Self {
        let trigger_by = trigger_by.into();
        Self {
            trigger_by,
            ..Default::default()
        }
    }

    pub fn by_as(
        trigger_by: impl Into<Option<String>>,
        trigger_as: impl Into<Option<String>>,
    ) -> Self {
        let trigger_by = trigger_by.into();
        let trigger_as = trigger_as.into();
        Self {
            trigger_by,
            trigger_as,
            ..Default::default()
        }
    }

    pub fn trigger_by(&self) -> crate::Result<String> {
        match self.trigger_as.to_owned().or(self.trigger_by.to_owned()) {
            Some(id) => Ok(id),
            _ => crate::bail!("User not found in metadata"),
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new(None, None)
    }
}

pub type Event<D> = EventDetails<D, Metadata>;
