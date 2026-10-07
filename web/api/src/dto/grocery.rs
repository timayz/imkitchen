use imkitchen_web_shared::services::grocery::{AisleSection, Groceries as GroceriesView};
use serde::{Deserialize, Serialize};

use super::recipe::IngredientDto;

#[derive(Serialize, Debug)]
pub struct Aisle {
    /// `shopping_<Category>`, the i18n key of the aisle label.
    pub key: String,
    pub items: Vec<Item>,
    pub checked: usize,
    pub total: usize,
    pub done: bool,
    pub pct: usize,
}

#[derive(Serialize, Debug)]
pub struct Item {
    #[serde(flatten)]
    pub ingredient: IngredientDto,
    pub checked: bool,
}

#[derive(Serialize, Debug)]
pub struct Groceries {
    pub recipe_count: usize,
    pub total_items: usize,
    pub checked_items: usize,
    pub progress_pct: usize,
    pub aisles: Vec<Aisle>,
}

impl From<GroceriesView> for Groceries {
    fn from(groceries: GroceriesView) -> Self {
        let view = groceries.view;
        let aisle = |a: &AisleSection| Aisle {
            key: a.name.to_owned(),
            items: a
                .items
                .iter()
                .map(|i| Item {
                    checked: view.checked.contains(&i.key()),
                    ingredient: IngredientDto::from(i),
                })
                .collect(),
            checked: a.checked,
            total: a.total,
            done: a.done,
            pct: a.pct,
        };
        Self {
            recipe_count: groceries.recipe_count,
            total_items: view.total_items,
            checked_items: view.checked_items,
            progress_pct: view.progress_pct,
            aisles: view.aisles.iter().map(aisle).collect(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ToggleRequest {
    /// The ingredient `key` from `GET /groceries`.
    pub key: String,
}
