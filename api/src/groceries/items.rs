//! What there is to buy: the ingredients of every meal of a user's list,
//! each scaled to the people the meal is cooked for, the same ones merged.
//!
//! Nothing of it is kept: it is worked out from the list and from its
//! recipes as their streams leave them, so it follows whatever changes in
//! either without a moment's delay.

use std::collections::{BTreeMap, HashMap};

use evento::Executor;
use imkitchen_aggregates::list::Meal;
use imkitchen_aggregates::recipe::Ingredient;
use sha2::{Digest, Sha256};

use crate::{list, plats, recipes};

/// The smallest amount an item is of, and what amounts are rounded to: a
/// recipe says no less of an ingredient.
const GRAIN: f64 = 0.001;

/// One thing to buy.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// What it is of and in what unit, as one word: the same for the same
    /// thing whatever the list holds.
    pub id: String,
    /// As the first recipe that takes it spells it.
    pub name: String,
    pub unit: Option<String>,
    /// How much the list calls for, in `unit`; `None` when no recipe says
    /// ("salt").
    pub quantity: Option<f64>,
}

/// What a meal takes of one thing: what its recipe says, and what that is
/// multiplied by for the people the meal is cooked for.
#[derive(Debug, Clone, PartialEq)]
pub struct Portion {
    pub ingredient: Ingredient,
    pub factor: f64,
}

/// `text` as it is compared: "Olive  oil" is "olive oil".
fn folded(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    words.join(" ").to_lowercase()
}

/// The id of the item that is `name` counted in `unit`, both folded.
fn item_id(name: &str, unit: &str) -> String {
    let digest = Sha256::digest(format!("{name}\n{unit}").as_bytes());
    digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Whether two amounts of an item are the same one, to what amounts are
/// rounded to.
pub fn same_amount(one: Option<f64>, other: Option<f64>) -> bool {
    match (one, other) {
        (Some(one), Some(other)) => (one - other).abs() < GRAIN / 2.0,
        (None, None) => true,
        _ => false,
    }
}

/// `portions` as items, by name: those of the same thing in the same unit
/// are one item, of the sum of their amounts. Names and units are the same
/// whatever their case and their spacing; no unit is turned into another,
/// so "200 g" and "1 cup" of flour are two items. A portion without an
/// amount adds nothing to one that has some.
pub fn merge(portions: Vec<Portion>) -> Vec<Item> {
    let mut items: BTreeMap<(String, String), Item> = BTreeMap::new();
    for Portion { ingredient, factor } in portions {
        let unit = ingredient.unit.filter(|unit| !unit.trim().is_empty());
        let key = (
            folded(&ingredient.name),
            unit.as_deref().map(folded).unwrap_or_default(),
        );
        let item = items
            .entry(key)
            .or_insert_with_key(|(name, unit_key)| Item {
                id: item_id(name, unit_key),
                name: ingredient.name,
                unit,
                quantity: None,
            });
        if let Some(quantity) = ingredient.quantity {
            item.quantity = Some(item.quantity.unwrap_or(0.0) + quantity * factor);
        }
    }
    let mut items: Vec<Item> = items.into_values().collect();
    for item in &mut items {
        item.quantity = item
            .quantity
            .map(|quantity| ((quantity / GRAIN).round() * GRAIN).max(GRAIN));
    }
    items
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn portion(name: &str, quantity: Option<f64>, unit: Option<&str>, factor: f64) -> Portion {
        Portion {
            ingredient: Ingredient {
                name: name.to_owned(),
                quantity,
                unit: unit.map(str::to_owned),
            },
            factor,
        }
    }

    fn shown(items: &[Item]) -> Vec<(&str, Option<f64>, Option<&str>)> {
        items
            .iter()
            .map(|item| (item.name.as_str(), item.quantity, item.unit.as_deref()))
            .collect()
    }

    #[test]
    fn the_same_thing_in_the_same_unit_is_one_item_of_the_sum() {
        let items = merge(vec![
            portion("Flour", Some(250.0), Some("g"), 1.0),
            portion("salt", None, None, 1.0),
            portion("Eggs", Some(2.0), None, 1.5),
            portion("flour", Some(100.0), Some("G"), 0.5),
            portion("flour", Some(1.0), Some("cup"), 1.0),
            portion("  eggs ", Some(1.0), None, 2.0),
            portion("Salt", None, None, 3.0),
            portion("olive  oil", Some(1.0), Some("tbsp"), 1.0 / 3.0),
            portion("Olive oil", Some(1.0), Some("tbsp"), 1.0 / 3.0),
        ]);

        assert_eq!(
            shown(&items),
            [
                ("Eggs", Some(5.0), None),
                ("flour", Some(1.0), Some("cup")),
                ("Flour", Some(300.0), Some("g")),
                ("olive  oil", Some(0.667), Some("tbsp")),
                ("salt", None, None),
            ],
            "by name, as first spelled, rounded to the thousandth"
        );
    }

    #[test]
    fn an_amount_some_recipe_gives_is_not_lost_to_one_that_gives_none() {
        let items = merge(vec![
            portion("butter", None, None, 1.0),
            portion("butter", Some(20.0), None, 2.0),
            portion("saffron", Some(0.001), Some("g"), 0.01),
        ]);

        assert_eq!(
            shown(&items),
            [
                ("butter", Some(40.0), None),
                ("saffron", Some(0.001), Some("g")),
            ],
            "and what is called for is never nothing"
        );
    }

    #[test]
    fn an_item_keeps_its_id_whatever_the_list_and_however_it_is_spelled() {
        let one = merge(vec![portion("Olive oil", Some(1.0), Some("tbsp"), 1.0)]);
        let other = merge(vec![
            portion("olive  OIL", Some(9.0), Some("Tbsp"), 4.0),
            portion("olive oil", Some(9.0), Some("ml"), 4.0),
            portion("olive oil", Some(9.0), None, 4.0),
        ]);

        assert_eq!(one[0].id.len(), 32);
        let ids: Vec<&str> = other.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(
            ids.iter().filter(|id| **id == one[0].id).count(),
            1,
            "another unit is another item"
        );
        assert_eq!(other.len(), 3);
    }

    #[test]
    fn amounts_are_the_same_to_the_thousandth() {
        assert!(same_amount(None, None));
        assert!(same_amount(Some(0.1 + 0.2), Some(0.3)));
        assert!(!same_amount(Some(0.3), Some(0.301)));
        assert!(!same_amount(Some(1.0), None));
    }
}
