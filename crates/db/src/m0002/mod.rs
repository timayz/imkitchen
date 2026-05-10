use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0002",
    vec_box![super::m0001::Migration],
    vec_box![crate::recipe_user::m0002::AddDifficultyScore]
);
