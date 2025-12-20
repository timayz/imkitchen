mod contact_list;
mod contact_stat;
mod mealplan_last_week;
mod mealplan_recipe;
mod mealplan_slot;
mod mealplan_week;
mod recipe_list;
mod recipe_rating;
mod recipe_user_stat;
mod shopping_list;
mod user;
mod user_list;
mod user_login;
mod user_stat;

use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0001",
    vec_box![],
    vec_box![
        user::CreateTable,
        user::CreateUk1,
        user::CreateUk2,
        user_login::CreateTable,
        user_login::CreateIdx1,
        user_login::CreateUk1,
        user_stat::CreateTable,
        user_list::CreateTable,
        user_list::CreateIdx1,
        user_list::CreateIdx2,
        contact_list::CreateTable,
        contact_list::CreateIdx1,
        contact_list::CreateIdx2,
        contact_stat::CreateTable,
        recipe_list::CreateTable,
        recipe_list::CreateIdx1,
        recipe_list::CreateIdx2,
        recipe_list::CreateIdx3,
        recipe_list::CreateIdx4,
        recipe_list::CreateIdx5,
        recipe_list::CreateIdx6,
        recipe_user_stat::CreateTable,
        recipe_rating::CreateTable,
        recipe_rating::CreateIdx1,
        mealplan_recipe::CreateTable,
        mealplan_recipe::CreateIdx1,
        mealplan_recipe::CreateIdx2,
        mealplan_recipe::CreateIdx3,
        mealplan_week::CreateTable,
        mealplan_last_week::CreateTable,
        mealplan_slot::CreateTable,
        shopping_list::CreateTable
    ]
);
