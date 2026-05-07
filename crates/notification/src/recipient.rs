use std::ops::Deref;

use bitcode::{Decode, Encode};
use evento::{Executor, Projection, metadata::Event};
use imkitchen_identity::types::user::{self, LoggedIn, Registered};

#[derive(Clone)]
pub struct Module<E: Executor>(pub(crate) imkitchen_core::State<E>);

impl<E: Executor> Deref for Module<E> {
    type Target = imkitchen_core::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Module<E> {
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<Recipient>> {
        create_projection(id).execute(&self.executor).await
    }
}

#[evento::projection(Encode, Decode)]
pub struct Recipient {
    pub user_id: String,
    pub email: String,
    pub lang: String,
    pub timezone: String,
}

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, Recipient> {
    Projection::new::<user::User>(id)
        .handler(handle_registered())
        .handler(handle_logged_in())
        .safety_check()
}

impl evento::ProjectionAggregator for Recipient {
    fn aggregator_id(&self) -> String {
        self.user_id.to_owned()
    }
}

#[evento::handler]
async fn handle_registered(event: Event<Registered>, data: &mut Recipient) -> anyhow::Result<()> {
    data.user_id = event.aggregator_id.to_owned();
    data.email = event.data.email.to_owned();
    data.lang = event.data.lang.to_owned();
    data.timezone = event.data.timezone.to_owned();

    Ok(())
}

#[evento::handler]
async fn handle_logged_in(event: Event<LoggedIn>, data: &mut Recipient) -> anyhow::Result<()> {
    data.lang = event.data.lang.to_owned();
    data.timezone = event.data.timezone.to_owned();

    Ok(())
}
