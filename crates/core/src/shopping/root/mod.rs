mod add;
mod generate;
mod merge;
mod remove;
mod state;
mod toogle;

use bitcode::{Decode, Encode};
pub use generate::Generate;
pub use state::ShoppingState;
pub use toogle::*;

use evento::{Executor, Projection, ProjectionAggregate, metadata::Event};
use imkitchen_types::shopping::{
    self, Checked, Generated, RecipeAdded, RecipeRemoved, RecipeSetGenerated, Unchecked,
};
use std::{collections::HashSet, ops::Deref};

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

    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<Shopping>> {
        create_projection().load(id).execute(&self.executor).await
    }
}

#[evento::projection(Encode, Decode)]
pub struct Shopping {
    pub user_id: String,
    pub checked: HashSet<String>,
    pub ingredients: HashSet<String>,
    pub recipes: HashSet<String>,
    pub from_date: u64,
    pub days: u8,
    pub generated_at: u64,
}

impl ProjectionAggregate for Shopping {
    fn aggregate_id(&self) -> String {
        self.user_id.to_owned()
    }
}

pub fn create_projection<E: Executor>() -> Projection<E, Shopping> {
    Projection::new::<shopping::Shopping>()
        // Bumped from the implicit 0 → 1 when the `recipes` field was added to
        // `Shopping`: invalidates old snapshots so they rebuild from events
        // rather than failing to bitcode-decode into the new struct shape.
        .revision(1)
        .handler(handle_checked())
        .handler(handle_generated())
        .handler(handle_unchecked())
        .handler(handle_recipe_set_generated())
        .handler(handle_recipe_added())
        .handler(handle_recipe_removed())
        .strict()
}

#[evento::handler]
async fn handle_generated(event: Event<Generated>, data: &mut Shopping) -> anyhow::Result<()> {
    data.user_id = event.metadata.requested_by()?;
    data.ingredients = event.data.ingredients.iter().map(|i| i.key()).collect();
    data.checked = HashSet::new();
    data.from_date = event.data.from_date;
    data.days = event.data.days;
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

#[evento::handler]
async fn handle_recipe_set_generated(
    event: Event<RecipeSetGenerated>,
    data: &mut Shopping,
) -> anyhow::Result<()> {
    data.recipes = event.data.recipe_ids.into_iter().collect();

    Ok(())
}

#[evento::handler]
async fn handle_recipe_added(event: Event<RecipeAdded>, data: &mut Shopping) -> anyhow::Result<()> {
    data.user_id = event.metadata.requested_by()?;
    data.recipes = event.data.recipe_ids.into_iter().collect();
    data.ingredients = event.data.ingredients.iter().map(|i| i.key()).collect();

    Ok(())
}

#[evento::handler]
async fn handle_recipe_removed(
    event: Event<RecipeRemoved>,
    data: &mut Shopping,
) -> anyhow::Result<()> {
    data.recipes = event.data.recipe_ids.into_iter().collect();
    data.ingredients = event.data.ingredients.iter().map(|i| i.key()).collect();

    Ok(())
}
