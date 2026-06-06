use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0005",
    vec_box![super::m0004::Migration],
    vec_box![crate::user_login::m0005::AddEmail]
);
