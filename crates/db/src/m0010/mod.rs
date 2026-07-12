use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0010",
    vec_box![super::m0009::Migration],
    vec_box![crate::recipe_user::m0010::RebuildFts]
);
