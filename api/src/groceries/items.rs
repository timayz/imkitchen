//! What there is to buy: the ingredients of every meal of a user's list,
//! each scaled to the people the meal is cooked for, the same ones merged
//! (`imkitchen_aggregates::groceries::items`, which app works its own out
//! with).
//!
//! Nothing of it is kept: it is worked out from the list and from its
//! recipes as their streams leave them, so it follows whatever changes in
//! either without a moment's delay.

use std::collections::HashMap;

use evento::Executor;
pub use imkitchen_aggregates::groceries::items::{Item, same_amount};
use imkitchen_aggregates::groceries::items::{Portion, merge};
use imkitchen_aggregates::list::Meal;
use imkitchen_aggregates::recipe::Ingredient;

use crate::{list, plats, recipes};

/// What `owner_id` has to buy to cook their list as it stands. A plat is
/// each of its recipes; a meal that was deleted and is not off the list
/// yet takes nothing.
pub async fn of<E: Executor>(executor: &E, owner_id: &str) -> anyhow::Result<Vec<Item>> {
    // A recipe is read once, however many meals it is part of.
    let mut recipes: HashMap<String, Option<(Vec<Ingredient>, u32)>> = HashMap::new();
    let mut portions = Vec::new();
    for entry in list::entries_of(executor, owner_id).await? {
        let recipe_ids = match entry.meal {
            Meal::Recipe(id) => vec![id],
            Meal::Plat(id) => plats::recipe_ids_of(executor, owner_id, &id)
                .await?
                .unwrap_or_default(),
        };
        for id in recipe_ids {
            if !recipes.contains_key(&id) {
                let recipe = recipes::ingredients_of(executor, owner_id, &id).await?;
                recipes.insert(id.clone(), recipe);
            }
            let Some((ingredients, servings)) = &recipes[&id] else {
                continue;
            };
            let factor = f64::from(entry.servings) / f64::from((*servings).max(1));
            portions.extend(ingredients.iter().map(|ingredient| Portion {
                ingredient: ingredient.clone(),
                factor,
            }));
        }
    }
    Ok(merge(portions))
}
