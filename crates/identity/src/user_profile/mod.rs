mod update;

use std::ops::Deref;
pub use update::*;

use evento::{Executor, Projection, metadata::Event};
use imkitchen_types::user_profile::{self, Changed};

#[derive(Clone)]
pub struct Module<E: Executor>(pub(crate) imkitchen_core::State<E>);

impl<E: Executor> Deref for Module<E> {
    type Target = imkitchen_core::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Module<E> {
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<UserProfile> {
        let id = id.into();

        create_projection::<E>()
            .load(&id)
            .execute(&self.executor)
            .await
            .map(|r| {
                r.unwrap_or_else(|| UserProfile {
                    id,
                    description: String::new(),
                    cursor: Default::default(),
                })
            })
    }
}

#[evento::projection(bitcode::Encode, bitcode::Decode)]
pub struct UserProfile {
    pub id: String,
    pub description: String,
}

fn create_projection<E: Executor>() -> Projection<E, UserProfile> {
    Projection::new::<user_profile::UserProfile>()
        .handler(handle_changed())
        .strict()
}

impl evento::ProjectionAggregate for UserProfile {
    fn aggregate_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_changed(event: Event<Changed>, data: &mut UserProfile) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.description = event.data.description;

    Ok(())
}
