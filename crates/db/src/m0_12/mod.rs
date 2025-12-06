mod mealplan_slot;
mod shopping_list;

use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0_12",
    vec_box![crate::m0_11::Migration],
    vec_box![mealplan_slot::CreateTable, shopping_list::CreateTable]
);
