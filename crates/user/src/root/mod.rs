use std::ops::Deref;

use evento::{Executor, Projection, ProjectionCursor, Snapshot, cursor, metadata::Event};
use imkitchen_shared::user::{
    self, Activated, LoggedIn, Logout, MadeAdmin, Registered, Role, State, Suspended,
    UsernameChanged,
};

use crate::repository::{self};

mod activate;
mod login;
mod made_admin;
mod register;
mod set_username;
mod suspend;

pub use login::LoginInput;
pub use register::RegisterInput;
pub use set_username::SetUsernameInput;

pub struct Command<E: Executor> {
    state: imkitchen_shared::State<E>,
    pub subscription: crate::subscription::Command<E>,
    pub meal_preferences: crate::meal_preferences::Command<E>,
    pub password: crate::password::Command<E>,
}

impl<E: Executor> Deref for Command<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<E: Executor> Command<E> {
    pub fn new(state: imkitchen_shared::State<E>) -> Self
    where
        imkitchen_shared::State<E>: Clone,
    {
        Self {
            subscription: crate::subscription::Command(state.clone()),
            meal_preferences: crate::meal_preferences::Command(state.clone()),
            password: crate::password::Command(state.clone()),
            state,
        }
    }
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<User>> {
        create_projection(id).execute(&self.executor).await
    }
}

#[derive(Default)]
pub struct User {
    pub id: String,
    pub role: Role,
    pub state: State,
    pub cursor: cursor::Value,
}

pub fn create_projection(id: impl Into<String>) -> Projection<User> {
    Projection::new::<user::User>(id)
        .handler(handle_registered())
        .handler(handle_actived())
        .handler(handle_susended())
        .handler(handle_made_admin())
        .skip::<LoggedIn>()
        .skip::<Logout>()
        .skip::<UsernameChanged>()
        .safety_check()
}

impl ProjectionCursor for User {
    fn set_cursor(&mut self, v: &cursor::Value) {
        self.cursor = v.to_owned();
    }

    fn get_cursor(&self) -> cursor::Value {
        self.cursor.to_owned()
    }

    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}

impl Snapshot for User {}

#[evento::handler]
async fn handle_registered(event: Event<Registered>, data: &mut User) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
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
async fn handle_actived(_event: Event<Activated>, data: &mut User) -> anyhow::Result<()> {
    data.state = State::Active;

    Ok(())
}

#[evento::handler]
async fn handle_susended(_event: Event<Suspended>, data: &mut User) -> anyhow::Result<()> {
    data.state = State::Suspended;

    Ok(())
}
