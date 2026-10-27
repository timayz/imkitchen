use bitcode::{Decode, Encode};
use evento::{Executor, Projection, ProjectionAggregate, metadata::Event};
use imkitchen_types::dish::{self, Changed, Created, Deleted};
use std::ops::Deref;

mod compose;
mod delete;

pub use compose::{ComposeInput, Composition, compose};

#[derive(Clone)]
pub struct Module<E: Executor> {
    state: crate::State<E>,
}

impl<E: Executor> Deref for Module<E> {
    type Target = crate::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<E: Executor> Module<E> {
    pub fn new(state: crate::State<E>) -> Self
    where
        crate::State<E>: Clone,
    {
        Self { state }
    }

    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<Dish>> {
        load(&self.executor, id).await
    }
}

/// The dish aggregate as commands see it: owner, name and recipe ids in
/// meal order. Deleted dishes are tombstoned and load as `None`.
#[evento::projection(name = "imkitchen-core/dish/Dish", Encode, Decode)]
pub struct Dish {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub recipe_ids: Vec<String>,
}

impl ProjectionAggregate for Dish {
    fn aggregate_id(&self) -> String {
        self.id.to_owned()
    }
}

pub fn create_projection<E: Executor>() -> Projection<E, Dish> {
    Projection::new::<dish::Dish>()
        .tombstone::<Deleted>()
        .handler(handle_created())
        .handler(handle_changed())
        .strict()
}

pub(crate) async fn load<E: Executor>(
    executor: &E,
    id: impl Into<String>,
) -> anyhow::Result<Option<Dish>> {
    create_projection().load(id).execute(executor).await
}

#[evento::handler]
async fn handle_created(event: Event<Created>, data: &mut Dish) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.owner_id = event.metadata.requested_by()?;
    data.name = event.data.name;
    data.recipe_ids = event.data.recipe_ids;

    Ok(())
}

#[evento::handler]
async fn handle_changed(event: Event<Changed>, data: &mut Dish) -> anyhow::Result<()> {
    data.name = event.data.name;
    data.recipe_ids = event.data.recipe_ids;

    Ok(())
}
