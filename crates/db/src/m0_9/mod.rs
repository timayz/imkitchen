mod contact_list;
mod contact_stat;
mod user_auth;
mod user_list;
mod user_stat;

use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0_9",
    vec_box![],
    vec_box![
        user_stat::CreateTable,
        user_auth::CreateTable,
        user_auth::CreateUk1,
        user_list::CreateTable,
        user_list::CreateIdx1,
        user_list::CreateIdx2,
        contact_list::CreateTable,
        contact_list::CreateIdx1,
        contact_list::CreateIdx2,
        contact_stat::CreateTable,
    ]
);
