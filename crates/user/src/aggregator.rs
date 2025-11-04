use bincode::{Decode, Encode};

use crate::{LoggedIn, RegisterFailed, RegisterRequested, Registered, UserEvent};

#[derive(Encode, Decode, Clone, Debug)]
pub enum Action {
    Register,
}

#[derive(Default, Encode, Decode, Clone, Debug)]
pub enum Status {
    #[default]
    Idle,
    Processing(Action),
    Failed(String),
}

#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct User {
    status: Status,
}

#[evento::aggregator]
impl User {
    async fn handle_register_requested(
        &mut self,
        _event: UserEvent<RegisterRequested>,
    ) -> anyhow::Result<()> {
        self.status = Status::Processing(Action::Register);

        Ok(())
    }

    async fn handle_registered(&mut self, _event: UserEvent<Registered>) -> anyhow::Result<()> {
        self.status = Status::Idle;

        Ok(())
    }

    async fn handle_register_failed(
        &mut self,
        event: UserEvent<RegisterFailed>,
    ) -> anyhow::Result<()> {
        self.status = Status::Failed(event.data.reason);

        Ok(())
    }

    async fn handle_logged_in(&mut self, _event: UserEvent<LoggedIn>) -> anyhow::Result<()> {
        Ok(())
    }
}
