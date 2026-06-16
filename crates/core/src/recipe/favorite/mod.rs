mod save;
mod unsave;

use bitcode::{Decode, Encode};
use evento::{Executor, Projection, ProjectionAggregate, metadata::Event};
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

        create_projection::<E>()
            .load_ids(vec![id.clone(), user_id.clone()])
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

pub fn create_projection<E: Executor>() -> Projection<E, Favorite> {
    Projection::new::<favorite::Favorite>()
        .handler(handle_saved())
        .handler(handle_unsaved())
        .strict()
}

impl ProjectionAggregate for Favorite {
    fn aggregate_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_saved(event: Event<favorite::Saved>, data: &mut Favorite) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.saved = true;

    Ok(())
}

#[evento::handler]
async fn handle_unsaved(
    event: Event<favorite::Unsaved>,
    data: &mut Favorite,
) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.saved = false;

    Ok(())
}
