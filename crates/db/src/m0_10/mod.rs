mod recipe_list;
mod recipe_user_stat;

use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0_10",
    vec_box![crate::m0_9::Migration],
    vec_box![
        recipe_list::CreateTable,
        recipe_list::CreateIdx1,
        recipe_list::CreateIdx2,
        recipe_list::CreateIdx3,
        recipe_user_stat::CreateTable,
    ]
);
