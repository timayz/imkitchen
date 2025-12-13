use bincode::{Decode, Encode};
use imkitchen_shared::Event;

use crate::{
    Activated, LoggedIn, MadeAdmin, RegistrationFailed, RegistrationRequested,
    RegistrationSucceeded, Role, State, Status, Suspended, reset_password::Resetted,
};

#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct User {
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub state: State,
    pub premium_expire_at: u64,
    pub status: Status,
    pub failed_reason: Option<String>,
}

#[evento::aggregator]
impl User {
    async fn handle_register_requested(
        &mut self,
        event: Event<RegistrationRequested>,
    ) -> anyhow::Result<()> {
        self.status = event.data.status;
        self.email = event.data.email;
        self.password_hash = event.data.password_hash;

        Ok(())
    }

    async fn handle_registered(
        &mut self,
        event: Event<RegistrationSucceeded>,
    ) -> anyhow::Result<()> {
        self.status = event.data.status;

        Ok(())
    }

    async fn handle_register_failed(
        &mut self,
        event: Event<RegistrationFailed>,
    ) -> anyhow::Result<()> {
        self.status = event.data.status;
        self.failed_reason = Some(event.data.reason);

        Ok(())
    }

    async fn handle_logged_in(&mut self, _event: Event<LoggedIn>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn handle_password_resetted(&mut self, event: Event<Resetted>) -> anyhow::Result<()> {
        self.password_hash = event.data.password_hash;

        Ok(())
    }

    async fn handle_made_admin(&mut self, event: Event<MadeAdmin>) -> anyhow::Result<()> {
        self.role = event.data.role;

        Ok(())
    }

    async fn handle_suspended(&mut self, event: Event<Suspended>) -> anyhow::Result<()> {
        self.state = event.data.state;

        Ok(())
    }

    async fn handle_activated(&mut self, event: Event<Activated>) -> anyhow::Result<()> {
        self.state = event.data.state;

        Ok(())
    }
}
