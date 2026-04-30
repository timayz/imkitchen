mod generate;
mod toogle;

use bitcode::{Decode, Encode};
pub use generate::Generate;
pub use toogle::*;

use evento::{Executor, Projection, ProjectionAggregator, metadata::Event};
use imkitchen_shared::shopping::{self, Checked, Generated, Unchecked};
use std::{collections::HashSet, ops::Deref};

#[derive(Clone)]
pub struct Module<E: Executor> {
    state: imkitchen_shared::State<E>,
}

impl<E: Executor> Deref for Module<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<E: Executor> Module<E> {
    pub fn new(state: imkitchen_shared::State<E>) -> Self
    where
        imkitchen_shared::State<E>: Clone,
    {
        Self { state }
    }

    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<Shopping>> {
        create_projection(id).execute(&self.executor).await
    }
}

#[evento::projection(Encode, Decode)]
pub struct Shopping {
    pub user_id: String,
    pub checked: HashSet<String>,
    pub ingredients: HashSet<String>,
    pub generated_at: u64,
}

impl ProjectionAggregator for Shopping {
    fn aggregator_id(&self) -> String {
        self.user_id.to_owned()
    }
}

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, Shopping> {
    Projection::new::<shopping::Shopping>(id)
        .handler(handle_checked())
        .handler(handle_generated())
        .handler(handle_unchecked())
        .safety_check()
}

#[evento::handler]
async fn handle_generated(event: Event<Generated>, data: &mut Shopping) -> anyhow::Result<()> {
    data.user_id = event.metadata.requested_by()?;
    data.ingredients = event.data.ingredients.iter().map(|i| i.key()).collect();
    data.checked = HashSet::new();
    data.generated_at = event.timestamp;

    Ok(())
}

#[evento::handler]
async fn handle_checked(event: Event<Checked>, data: &mut Shopping) -> anyhow::Result<()> {
    data.checked.insert(event.data.ingredient);

    Ok(())
}

#[evento::handler]
async fn handle_unchecked(event: Event<Unchecked>, data: &mut Shopping) -> anyhow::Result<()> {
    data.checked.remove(&event.data.ingredient);

    Ok(())
}
