use bincode::{Decode, Encode};

use crate::{
    LoggedIn, RegistrationFailed, RegistrationRequested, RegistrationSucceeded, UserEvent,
};

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
}

#[evento::aggregator]
impl User {
    async fn handle_register_requested(
        &mut self,
        _event: UserEvent<RegistrationRequested>,
    ) -> anyhow::Result<()> {
        self.status = Status::Processing(Action::Registration);

        Ok(())
    }

    async fn handle_registered(
        &mut self,
        _event: UserEvent<RegistrationSucceeded>,
    ) -> anyhow::Result<()> {
        self.status = Status::Idle;

        Ok(())
    }

    async fn handle_register_failed(
        &mut self,
        event: UserEvent<RegistrationFailed>,
    ) -> anyhow::Result<()> {
        self.status = Status::Failed(event.data.reason);

        Ok(())
    }

    async fn handle_logged_in(&mut self, _event: UserEvent<LoggedIn>) -> anyhow::Result<()> {
        Ok(())
    }
}
