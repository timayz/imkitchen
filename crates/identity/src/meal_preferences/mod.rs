mod set_aisle_order;
mod update;

use bitcode::{Decode, Encode};
use std::ops::Deref;
pub use update::*;

use evento::{Executor, Projection, metadata::Event};
use imkitchen_types::meal_preferences::{self, AisleOrderChanged, Changed, RecipeTypesChanged};
use imkitchen_types::recipe::{DietaryRestriction, IngredientCategory, RecipeType};

#[derive(Clone)]
pub struct Module<E: Executor>(pub(crate) imkitchen_core::State<E>);

impl<E: Executor> Deref for Module<E> {
    type Target = imkitchen_core::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Module<E> {
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<MealPreferences> {
        let id = id.into();

        create_projection::<E>()
            .load(&id)
            .execute(&self.executor)
            .await
            .map(|r| match r {
                Some(mut row) => {
                    // Rows rebuilt from streams older than `aisle_order` hold an
                    // empty vec, and a stored order predates any variant added
                    // since: always hand out every aisle.
                    row.aisle_order = IngredientCategory::complete_aisle_order(&row.aisle_order);
                    row
                }
                None => MealPreferences {
                    id,
                    household_size: 4,
                    dietary_restrictions: vec![],
                    cuisine_variety_weight: 1.0,
                    recipe_types: RecipeType::default_meal_plan_types(),
                    aisle_order: IngredientCategory::DEFAULT_AISLE_ORDER.to_vec(),
                    cursor: Default::default(),
                    aggregate_version: Default::default(),
                },
            })
    }
}

#[evento::projection(
    name = "imkitchen-identity/meal_preferences/MealPreferences",
    Encode,
    Decode
)]
pub struct MealPreferences {
    pub id: String,
    pub household_size: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_variety_weight: f32,
    pub recipe_types: Vec<RecipeType>,
    pub aisle_order: Vec<IngredientCategory>,
}

fn create_projection<E: Executor>() -> Projection<E, MealPreferences> {
    Projection::new::<meal_preferences::MealPreferences>()
        // Bumped from the implicit 0 → 1 when the `recipe_types` field was added
        // to `MealPreferences`: invalidates old snapshots so they rebuild from
        // events rather than failing to bitcode-decode into the new struct shape.
        // 1 → 2: evento's `#[projection]` macro grew the `aggregate_version`
        // field, changing the bitcode layout again.
        // 2 → 3: `aisle_order` added.
        .revision(3)
        .handler(handle_updated())
        .handler(handle_recipe_types_changed())
        .handler(handle_aisle_order_changed())
        .strict()
}

impl evento::ProjectionAggregate for MealPreferences {
    fn aggregate_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_updated(event: Event<Changed>, data: &mut MealPreferences) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.household_size = event.data.household_size;
    data.dietary_restrictions = event.data.dietary_restrictions;
    data.cuisine_variety_weight = event.data.cuisine_variety_weight;
    // Streams written before `RecipeTypesChanged` existed carry only `Changed`,
    // so seed the historical behaviour here rather than leaving an empty vec
    // (which would mean "generate no optional courses"). `update` always commits
    // `RecipeTypesChanged` right after `Changed` — higher version, same
    // timestamp, so replayed second — and the user's real selection wins.
    data.recipe_types = RecipeType::default_meal_plan_types();
    // Unlike `recipe_types`, nothing re-commits the aisle order on each save:
    // only seed it, never reset a custom one on replay.
    if data.aisle_order.is_empty() {
        data.aisle_order = IngredientCategory::DEFAULT_AISLE_ORDER.to_vec();
    }

    Ok(())
}

#[evento::handler]
async fn handle_recipe_types_changed(
    event: Event<RecipeTypesChanged>,
    data: &mut MealPreferences,
) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.recipe_types = event.data.recipe_types;

    Ok(())
}

#[evento::handler]
async fn handle_aisle_order_changed(
    event: Event<AisleOrderChanged>,
    data: &mut MealPreferences,
) -> anyhow::Result<()> {
    data.id = event.aggregate_id.to_owned();
    data.aisle_order = event.data.aisles;

    Ok(())
}
