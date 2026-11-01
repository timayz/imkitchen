mod commands;

use bitcode::{Decode, Encode};
use evento::{Executor, Projection, ProjectionAggregate, metadata::Event};
use imkitchen_types::chef_follow;
use std::ops::Deref;

#[derive(Clone)]
pub struct Module<E: Executor>(pub(crate) crate::State<E>);

impl<E: Executor> Deref for Module<E> {
    type Target = crate::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Module<E> {
    pub async fn load(
        &self,
        chef_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> anyhow::Result<ChefFollow> {
        let chef_id = chef_id.into();
        let user_id = user_id.into();

        create_projection::<E>()
            .load_ids(vec![chef_id.clone(), user_id.clone()])
            .execute(&self.executor)
            .await
            .map(|r| {
                r.unwrap_or_else(|| ChefFollow {
                    id: evento::hash_ids(vec![chef_id, user_id]),
                    following: false,
                    cursor: Default::default(),
                    aggregate_version: Default::default(),
                    tenant: Default::default(),
                })
            })
    }
}

#[evento::projection(name = "imkitchen-core/recipe/follow/ChefFollow", Encode, Decode)]
pub struct ChefFollow {
    pub id: String,
    pub following: bool,
}

pub fn create_projection<E: Executor>() -> Projection<E, ChefFollow> {
    Projection::new::<chef_follow::ChefFollow>()
        .handler(handle_followed())
        .handler(handle_unfollowed())
        .strict()
}

impl ProjectionAggregate for ChefFollow {
    fn aggregate_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_followed(
    event: Event<chef_follow::Followed>,
    data: &mut ChefFollow,
) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.following = true;

    Ok(())
}

#[evento::handler]
async fn handle_unfollowed(
    event: Event<chef_follow::Unfollowed>,
    data: &mut ChefFollow,
) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.following = false;

    Ok(())
}
