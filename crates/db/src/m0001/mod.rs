mod contact_admin;
mod contact_global_stat;
mod mealplan_recipe;
mod mealplan_slot;
mod mealplan_week;
mod recipe_comment;
mod recipe_user;
mod recipe_user_stat;
mod shopping_list;
mod shopping_recipe;
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
        user_global_stat::CreateTable,
        user_admin::CreateTable,
        user_admin::CreateIdx1,
        user_admin::CreateIdx2,
        user_admin::CreateFTSTable,
        contact_admin::CreateTable,
        contact_admin::CreateIdx1,
        contact_admin::CreateIdx2,
        contact_global_stat::CreateTable,
        recipe_user::CreateTable,
        recipe_user::CreateIdx1,
        recipe_user::CreateIdx2,
        recipe_user::CreateIdx3,
        recipe_user::CreateIdx4,
        recipe_user::CreateIdx5,
        recipe_user::CreateIdx6,
        recipe_comment::CreateTable,
        recipe_comment::CreateIdx1,
        recipe_comment::CreateIdx2,
        recipe_comment::CreateIdx3,
        recipe_user_stat::CreateTable,
        mealplan_recipe::CreateTable,
        mealplan_recipe::CreateIdx1,
        mealplan_recipe::CreateIdx2,
        mealplan_recipe::CreateIdx3,
        mealplan_week::CreateTable,
        mealplan_slot::CreateTable,
        shopping_list::CreateTable,
        shopping_recipe::CreateTable,
        shopping_recipe::CreateIdx1
    ]
);
