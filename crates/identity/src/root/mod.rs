use crate::types::user::{
    self, Activated, LoggedIn, Logout, MadeAdmin, Registered, Role, RoleChanged, State, Suspended,
    UsernameChanged,
};
use bitcode::{Decode, Encode};
use evento::{Executor, Projection, ProjectionAggregate, metadata::Event};
use std::ops::Deref;

use crate::repository::{self};

mod activate;
mod change_role;
mod login;
mod made_admin;
mod register;
mod set_username;
mod suspend;

pub use login::LoginInput;
pub use register::RegisterInput;
pub use set_username::SetUsernameInput;

#[derive(Clone)]
pub struct Module<E: Executor> {
    state: imkitchen_core::State<E>,
    pub meal_preferences: crate::meal_preferences::Module<E>,
    pub password: crate::password::Module<E>,
    pub user_profile: crate::user_profile::Module<E>,
}

impl<E: Executor> Deref for Module<E> {
    type Target = imkitchen_core::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<E: Executor> Module<E> {
    pub fn new(state: imkitchen_core::State<E>) -> Self
    where
        imkitchen_core::State<E>: Clone,
    {
        Self {
            meal_preferences: crate::meal_preferences::Module(state.clone()),
            password: crate::password::Module(state.clone()),
            user_profile: crate::user_profile::Module(state.clone()),
            state,
        }
    }
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<User>> {
        create_projection().load(id).execute(&self.executor).await
    }

    pub async fn find_email(
        &self,
        id: impl Into<String>,
    ) -> imkitchen_core::Result<Option<String>> {
        Ok(
            repository::find(&self.read_db, repository::FindType::Id(id.into()))
                .await?
                .map(|row| row.email),
        )
    }
}

#[evento::projection(Encode, Decode)]
pub struct User {
    pub id: String,
    pub role: Role,
    pub state: State,
}

pub fn create_projection<E: Executor>() -> Projection<E, User> {
    Projection::new::<user::User>()
        .handler(handle_registered())
        .handler(handle_actived())
        .handler(handle_susended())
        .handler(handle_made_admin())
        .handler(handle_role_changed())
        .skip::<LoggedIn>()
        .skip::<Logout>()
        .skip::<UsernameChanged>()
        .strict()
}

impl ProjectionAggregate for User {
    fn aggregate_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_registered(event: Event<Registered>, data: &mut User) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.state = State::Active;
    data.role = Role::User;

    Ok(())
}

#[evento::handler]
async fn handle_made_admin(_event: Event<MadeAdmin>, data: &mut User) -> anyhow::Result<()> {
    data.role = Role::Admin;

    Ok(())
}

#[evento::handler]
async fn handle_role_changed(event: Event<RoleChanged>, data: &mut User) -> anyhow::Result<()> {
    data.role = event.data.role.to_owned();

    Ok(())
}

#[evento::handler]
async fn handle_actived(_event: Event<Activated>, data: &mut User) -> anyhow::Result<()> {
    data.state = State::Active;

    Ok(())
}

#[evento::handler]
async fn handle_susended(_event: Event<Suspended>, data: &mut User) -> anyhow::Result<()> {
    data.state = State::Suspended;

    Ok(())
}
