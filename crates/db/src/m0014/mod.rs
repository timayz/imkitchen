use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0014",
    vec_box![super::m0013::Migration],
    vec_box![crate::recipe_user::m0014::AddSitemapIndexes]
);
