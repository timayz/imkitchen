mod save;
mod unsave;

use bitcode::{Decode, Encode};
use evento::{Executor, Projection, ProjectionAggregator, metadata::Event};
use imkitchen_types::favorite::{self};
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
        id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> anyhow::Result<Favorite> {
        let id = id.into();
        let user_id = user_id.into();

        create_projection::<E>(&id, &user_id)
            .execute(&self.executor)
            .await
            .map(|r| {
                r.unwrap_or_else(|| Favorite {
                    id: evento::hash_ids(vec![id, user_id]),
                    saved: false,
                    cursor: Default::default(),
                })
            })
    }
}

#[evento::projection(Encode, Decode)]
pub struct Favorite {
    pub id: String,
    pub saved: bool,
}

pub fn create_projection<E: Executor>(
    id: impl Into<String>,
    user_id: impl Into<String>,
) -> Projection<E, Favorite> {
    Projection::ids::<favorite::Favorite>(vec![id.into(), user_id.into()])
        .handler(handle_saved())
        .handler(handle_unsaved())
        .safety_check()
}

impl ProjectionAggregator for Favorite {
    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_saved(event: Event<favorite::Saved>, data: &mut Favorite) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.saved = true;

    Ok(())
}

#[evento::handler]
async fn handle_unsaved(
    event: Event<favorite::Unsaved>,
    data: &mut Favorite,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.saved = false;

    Ok(())
}
