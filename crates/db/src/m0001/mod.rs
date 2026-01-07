mod contact_admin;
mod contact_global_stat;
mod mealplan_last_week;
mod mealplan_recipe;
mod mealplan_slot;
mod mealplan_week;
mod recipe_command;
mod recipe_rating_command;
mod recipe_user;
mod recipe_user_stat;
mod shopping_list;
mod user;
mod user_admin;
mod user_global_stat;
mod user_login;

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
        user_global_stat::CreateTable,
        user_admin::CreateTable,
        user_admin::CreateIdx1,
        user_admin::CreateIdx2,
        contact_admin::CreateTable,
        contact_admin::CreateIdx1,
        contact_admin::CreateIdx2,
        contact_global_stat::CreateTable,
        recipe_command::CreateTable,
        recipe_user::CreateTable,
        recipe_user::CreateIdx1,
        recipe_user::CreateIdx2,
        recipe_user::CreateIdx3,
        recipe_user::CreateIdx4,
        recipe_user::CreateIdx5,
        recipe_user::CreateIdx6,
        recipe_user_stat::CreateTable,
        recipe_rating_command::CreateTable,
        recipe_rating_command::CreateIdx1,
        mealplan_recipe::CreateTable,
        mealplan_recipe::CreateIdx1,
        mealplan_recipe::CreateIdx2,
        mealplan_week::CreateTable,
        mealplan_last_week::CreateTable,
        mealplan_slot::CreateTable,
        shopping_list::CreateTable
    ]
);
