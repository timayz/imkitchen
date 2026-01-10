mod reset;
mod toogle;

pub use toogle::*;

use evento::{Executor, Projection, ProjectionAggregator, Snapshot, metadata::Event};
use imkitchen_shared::shopping::{self, Checked, Generated, Resetted, Unchecked};
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

#[derive(Clone)]
pub struct Command<E: Executor> {
    state: imkitchen_shared::State<E>,
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
        Self { state }
    }

    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<Shopping>> {
        create_projection(id).execute(&self.executor).await
    }
}

#[evento::projection]
pub struct Shopping {
    pub user_id: String,
    pub checked: HashMap<u64, HashSet<String>>,
    pub ingredients: HashMap<u64, HashSet<String>>,
}

impl ProjectionAggregator for Shopping {
    fn aggregator_id(&self) -> String {
        self.user_id.to_owned()
    }
}

impl Snapshot for Shopping {}

pub fn create_projection(id: impl Into<String>) -> Projection<Shopping> {
    Projection::new::<shopping::Shopping>(id)
        .handler(handle_checked())
        .handler(handle_resetted())
        .handler(handle_generated())
        .handler(handle_unchecked())
        .safety_check()
}

#[evento::handler]
async fn handle_generated(event: Event<Generated>, data: &mut Shopping) -> anyhow::Result<()> {
    data.user_id = event.metadata.user()?;

    let ingredients = event.data.ingredients.iter().map(|i| i.key()).collect();

    data.ingredients.insert(event.data.week, ingredients);
    data.checked.remove(&event.data.week);

    if data.ingredients.len() <= 5 {
        return Ok(());
    }

    let mut keys = data.ingredients.keys().cloned().collect::<Vec<_>>();
    keys.sort();

    if let Some(key) = keys.first() {
        data.ingredients.remove(key);
        data.checked.remove(key);
    }

    Ok(())
}

#[evento::handler]
async fn handle_checked(event: Event<Checked>, data: &mut Shopping) -> anyhow::Result<()> {
    let entry = data.checked.entry(event.data.week).or_default();
    entry.insert(event.data.ingredient);

    Ok(())
}

#[evento::handler]
async fn handle_unchecked(event: Event<Unchecked>, data: &mut Shopping) -> anyhow::Result<()> {
    let entry = data.checked.entry(event.data.week).or_default();
    entry.remove(&event.data.ingredient);
    if entry.is_empty() {
        data.checked.remove(&event.data.week);
    }

    Ok(())
}

#[evento::handler]
async fn handle_resetted(event: Event<Resetted>, data: &mut Shopping) -> anyhow::Result<()> {
    data.checked.remove(&event.data.week);

    Ok(())
}
