use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0008",
    vec_box![super::m0007::Migration],
    vec_box![crate::user_admin::m0008::SyncEmailFts]
);
