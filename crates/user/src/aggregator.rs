use bincode::{Decode, Encode};
use imkitchen_shared::Event;

use crate::{LoggedIn, RegistrationFailed, RegistrationRequested, RegistrationSucceeded};

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub enum Action {
    Registration,
}

#[derive(Default, Encode, Decode, Clone, Debug, PartialEq)]
pub enum Status {
    #[default]
    Idle,
    Processing(Action),
    Failed(String),
}

#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct User {
    pub status: Status,
    pub password_hash: String,
}

#[evento::aggregator]
impl User {
    async fn handle_register_requested(
        &mut self,
        event: Event<RegistrationRequested>,
    ) -> anyhow::Result<()> {
        self.status = Status::Processing(Action::Registration);
        self.password_hash = event.data.password_hash;

        Ok(())
    }

    async fn handle_registered(
        &mut self,
        _event: Event<RegistrationSucceeded>,
    ) -> anyhow::Result<()> {
        self.status = Status::Idle;

        Ok(())
    }

    async fn handle_register_failed(
        &mut self,
        event: Event<RegistrationFailed>,
    ) -> anyhow::Result<()> {
        self.status = Status::Failed(event.data.reason);

        Ok(())
    }

    async fn handle_logged_in(&mut self, _event: Event<LoggedIn>) -> anyhow::Result<()> {
        Ok(())
    }
}
