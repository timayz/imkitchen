mod add;
mod add_dish;
mod check;
mod generate;
mod meal;
mod merge;
mod pick;
mod remove;
mod remove_dish;
mod state;
mod status;
mod toogle;

use bitcode::{Decode, Encode};
pub use check::SetCheckedInput;
pub use generate::GenerateList;
pub use merge::Servings;
pub use pick::{PoolRecipe, Randomize};
pub use state::ShoppingState;
pub use status::ChangeRecipeStatus;
pub use toogle::*;

use evento::{Executor, Projection, ProjectionAggregate, metadata::Event};
use imkitchen_types::shopping::{
    self, Checked, DishAdded, DishPlaced, DishRemoved, Generated, ListGenerated, RecipeAdded,
    RecipeRemoved, RecipeSetGenerated, RecipeStatus, RecipeStatusChanged, ServingsSet, Unchecked,
};
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

/// Largest household a list entry can be scaled to; matches the settings
/// form's upper bound.
pub const MAX_HOUSEHOLD_SIZE: u16 = 1000;

/// Reject a household size the settings form would not accept either.
pub(crate) fn check_household_size(household_size: u16) -> crate::Result<()> {
    if household_size == 0 || household_size > MAX_HOUSEHOLD_SIZE {
        crate::user!("household size must be between 1 and {MAX_HOUSEHOLD_SIZE}");
    }
    Ok(())
}

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

/// A composed dish's recipes that are in the list (its own recipes only, not
/// the courses generation filled in around it), in meal order. The kitchen
/// shows them as one row.
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq)]
pub struct ListDish {
    pub dish_id: String,
    pub recipe_ids: Vec<String>,
}

/// The user's recipe list: the recipes in it (in list order), the merged
/// ingredient keys they need, which of those are checked off, each
/// recipe's cooking status, the household size each recipe is scaled to and
/// which recipes form a composed dish.
#[evento::projection(name = "imkitchen-core/shopping/Shopping", Encode, Decode)]
pub struct Shopping {
    pub user_id: String,
    pub checked: HashSet<String>,
    pub ingredients: HashSet<String>,
    pub recipes: Vec<String>,
    /// Cooking status per recipe id; a missing entry means `Idle`.
    pub statuses: HashMap<String, RecipeStatus>,
    pub generated_at: u64,
    /// Household size chosen per recipe id when it was generated or added. A
    /// missing entry means the recipe predates per-recipe servings: callers
    /// fall back to the user's settings value.
    pub servings: HashMap<String, u16>,
    /// Dish groups in placement order; a recipe is in at most one.
    pub dishes: Vec<ListDish>,
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
            servings: Default::default(),
            dishes: Default::default(),
            cursor: Default::default(),
            aggregate_version: Default::default(),
            tenant: Default::default(),
        }
    }

    pub fn status(&self, recipe_id: &str) -> RecipeStatus {
        self.statuses.get(recipe_id).cloned().unwrap_or_default()
    }

    /// Household size the recipe is scaled to, or `default` when it was added
    /// before servings were recorded.
    pub fn servings(&self, recipe_id: &str, default: u16) -> u16 {
        self.servings.get(recipe_id).copied().unwrap_or(default)
    }

    /// The dish group placed under `dish_id`, if it is in the list.
    pub fn dish(&self, dish_id: &str) -> Option<&ListDish> {
        self.dishes.iter().find(|d| d.dish_id == dish_id)
    }

    /// The dish group `recipe_id` belongs to, if any.
    pub fn dish_of(&self, recipe_id: &str) -> Option<&ListDish> {
        self.dishes
            .iter()
            .find(|d| d.recipe_ids.iter().any(|id| id == recipe_id))
    }

    /// Drop group members that left the list, and groups left empty.
    fn prune_dishes(&mut self) {
        let recipes = &self.recipes;
        for dish in self.dishes.iter_mut() {
            dish.recipe_ids.retain(|id| recipes.contains(id));
        }
        self.dishes.retain(|d| !d.recipe_ids.is_empty());
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
        // 3 → 4: `RecipeRemoved` now drops checks for ingredients that left
        // the list; rebuild so lists emptied before the fix lose their stale
        // checks instead of carrying them onto the next recipes added.
        // 4 → 5 (2026-11-01): `servings` added when the list started
        // remembering the household size chosen per recipe.
        // 5 → 6 (2026-11-01): `dishes` added so the kitchen shows a composed
        // dish as one row.
        .revision(6)
        .handler(handle_checked())
        .handler(handle_generated())
        .handler(handle_unchecked())
        .handler(handle_recipe_set_generated())
        .handler(handle_recipe_added())
        .handler(handle_recipe_removed())
        .handler(handle_list_generated())
        .handler(handle_recipe_status_changed())
        .handler(handle_dish_added())
        .handler(handle_servings_set())
        .handler(handle_dish_placed())
        .handler(handle_dish_removed())
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
    data.servings.clear();
    data.dishes.clear();

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
    data.dishes.clear();

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
    data.servings.remove(&event.data.recipe_id);
    // A check belongs to the ingredient as it is on the list now; once the
    // recipe that needed it is gone, a later recipe using the same ingredient
    // must start unchecked.
    let ingredients = &data.ingredients;
    data.checked.retain(|key| ingredients.contains(key));
    data.prune_dishes();

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
    // `ServingsSet` and the `DishPlaced` groups follow in the same commit.
    data.servings.clear();
    data.dishes.clear();

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

#[evento::handler]
async fn handle_dish_added(event: Event<DishAdded>, data: &mut Shopping) -> anyhow::Result<()> {
    data.user_id = event.metadata.requested_by()?;
    data.recipes = event.data.recipe_ids;
    data.ingredients = event.data.ingredients.iter().map(|i| i.key()).collect();

    Ok(())
}

#[evento::handler]
async fn handle_servings_set(event: Event<ServingsSet>, data: &mut Shopping) -> anyhow::Result<()> {
    for recipe_id in event.data.recipe_ids {
        data.servings.insert(recipe_id, event.data.household_size);
    }

    Ok(())
}

/// A recipe belongs to one group: placing it under a dish pulls it out of any
/// other, and placing a dish again replaces its earlier group.
#[evento::handler]
async fn handle_dish_placed(event: Event<DishPlaced>, data: &mut Shopping) -> anyhow::Result<()> {
    let DishPlaced {
        dish_id,
        recipe_ids,
    } = event.data;
    data.dishes.retain(|d| d.dish_id != dish_id);
    for dish in data.dishes.iter_mut() {
        dish.recipe_ids.retain(|id| !recipe_ids.contains(id));
    }
    if !recipe_ids.is_empty() {
        data.dishes.push(ListDish {
            dish_id,
            recipe_ids,
        });
    }
    data.prune_dishes();

    Ok(())
}

#[evento::handler]
async fn handle_dish_removed(event: Event<DishRemoved>, data: &mut Shopping) -> anyhow::Result<()> {
    let removed: Vec<String> = data
        .recipes
        .iter()
        .filter(|id| !event.data.recipe_ids.contains(id))
        .cloned()
        .collect();
    data.recipes = event.data.recipe_ids;
    data.ingredients = event.data.ingredients.iter().map(|i| i.key()).collect();
    for id in &removed {
        data.statuses.remove(id);
        data.servings.remove(id);
    }
    let ingredients = &data.ingredients;
    data.checked.retain(|key| ingredients.contains(key));
    data.dishes.retain(|d| d.dish_id != event.data.dish_id);
    data.prune_dishes();

    Ok(())
}
