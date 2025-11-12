use std::fmt::Display;

use bincode::{Decode, Encode};
use imkitchen_shared::Event;

use crate::{
    Activated, LoggedIn, MadeAdmin, RegistrationFailed, RegistrationRequested,
    RegistrationSucceeded, Suspended,
};

#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub enum Action {
    Registration,
}

#[derive(Default, Encode, Decode, Clone, Debug, PartialEq)]
pub enum Role {
    #[default]
    User,
    Suspend,
    Admin,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub premium_expire_at: u64,
}

#[evento::aggregator]
impl User {
    async fn handle_register_requested(
        &mut self,
        event: Event<RegistrationRequested>,
    ) -> anyhow::Result<()> {
        self.status = Status::Processing(Action::Registration);
        self.email = event.data.email;
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

    async fn handle_made_admin(&mut self, _event: Event<MadeAdmin>) -> anyhow::Result<()> {
        self.role = Role::Admin;

        Ok(())
    }

    async fn handle_suspended(&mut self, _event: Event<Suspended>) -> anyhow::Result<()> {
        self.role = Role::Suspend;

        Ok(())
    }

    async fn handle_activated(&mut self, _event: Event<Activated>) -> anyhow::Result<()> {
        self.role = Role::User;

        Ok(())
    }
}
