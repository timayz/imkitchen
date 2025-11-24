mod mealplan_recipe;
mod mealplan_week;

use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0_11",
    vec_box![],
    vec_box![
        mealplan_recipe::CreateTable,
        mealplan_recipe::CreateIdx1,
        mealplan_recipe::CreateIdx2,
        mealplan_recipe::CreateIdx3,
        mealplan_week::CreateTable,
    ]
);
