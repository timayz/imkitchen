mod add;
mod generate;
mod merge;
mod pick;
mod remove;
mod state;
mod status;
mod toogle;

use bitcode::{Decode, Encode};
pub use generate::GenerateList;
pub use pick::{PoolRecipe, Randomize};
pub use state::ShoppingState;
pub use status::ChangeRecipeStatus;
pub use toogle::*;

use evento::{Executor, Projection, ProjectionAggregate, metadata::Event};
use imkitchen_types::shopping::{
    self, Checked, Generated, ListGenerated, RecipeAdded, RecipeRemoved, RecipeSetGenerated,
    RecipeStatus, RecipeStatusChanged, Unchecked,
};
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

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

    /// Load the user's list, or an empty one for a user who has none yet.
    pub(crate) async fn load_or_empty(&self, user_id: &str) -> anyhow::Result<Shopping> {
        Ok(self
            .load(user_id)
            .await?
            .unwrap_or_else(|| Shopping::empty(user_id)))
    }
}

/// The user's recipe list: the recipes in it (in list order), the merged
/// ingredient keys they need, which of those are checked off, and each
/// recipe's cooking status.
#[evento::projection(name = "imkitchen-core/shopping/Shopping", Encode, Decode)]
pub struct Shopping {
    pub user_id: String,
    pub checked: HashSet<String>,
    pub ingredients: HashSet<String>,
    pub recipes: Vec<String>,
    /// Cooking status per recipe id; a missing entry means `Idle`.
    pub statuses: HashMap<String, RecipeStatus>,
    pub generated_at: u64,
}

impl Shopping {
    fn empty(user_id: &str) -> Self {
        Shopping {
            user_id: user_id.to_owned(),
            checked: Default::default(),
            ingredients: Default::default(),
            recipes: Default::default(),
            statuses: Default::default(),
            generated_at: 0,
            cursor: Default::default(),
            aggregate_version: Default::default(),
        }
    }

    pub fn status(&self, recipe_id: &str) -> RecipeStatus {
        self.statuses.get(recipe_id).cloned().unwrap_or_default()
    }
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
        // 1 → 2: evento's `#[projection]` macro grew the `aggregate_version`
        // field, changing the bitcode layout again.
        // 2 → 3: the meal-plan window (`from_date`/`days`) was dropped,
        // `recipes` became an ordered `Vec` and `statuses` was added when the
        // list replaced the calendar meal plan.
        .revision(3)
        .handler(handle_checked())
        .handler(handle_generated())
        .handler(handle_unchecked())
        .handler(handle_recipe_set_generated())
        .handler(handle_recipe_added())
        .handler(handle_recipe_removed())
        .handler(handle_list_generated())
        .handler(handle_recipe_status_changed())
        .strict()
}

/// Legacy date-ranged generation: still replayed for lists generated before
/// the calendar was removed.
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

/// Legacy companion of `Generated`.
#[evento::handler]
async fn handle_recipe_set_generated(
    event: Event<RecipeSetGenerated>,
    data: &mut Shopping,
) -> anyhow::Result<()> {
    data.recipes = event.data.recipe_ids;
    data.statuses.clear();

    Ok(())
}

#[evento::handler]
async fn handle_recipe_added(event: Event<RecipeAdded>, data: &mut Shopping) -> anyhow::Result<()> {
    data.user_id = event.metadata.requested_by()?;
    data.recipes = event.data.recipe_ids;
    data.ingredients = event.data.ingredients.iter().map(|i| i.key()).collect();

    Ok(())
}

#[evento::handler]
async fn handle_recipe_removed(
    event: Event<RecipeRemoved>,
    data: &mut Shopping,
) -> anyhow::Result<()> {
    data.recipes = event.data.recipe_ids;
    data.ingredients = event.data.ingredients.iter().map(|i| i.key()).collect();
    data.statuses.remove(&event.data.recipe_id);

    Ok(())
}

#[evento::handler]
async fn handle_list_generated(
    event: Event<ListGenerated>,
    data: &mut Shopping,
) -> anyhow::Result<()> {
    data.user_id = event.metadata.requested_by()?;
    data.generated_at = event.timestamp;
    data.recipes = event.data.recipe_ids;
    data.ingredients = event.data.ingredients.iter().map(|i| i.key()).collect();
    data.checked.clear();
    data.statuses.clear();

    Ok(())
}

#[evento::handler]
async fn handle_recipe_status_changed(
    event: Event<RecipeStatusChanged>,
    data: &mut Shopping,
) -> anyhow::Result<()> {
    data.statuses
        .insert(event.data.recipe_id, event.data.status);

    Ok(())
}
