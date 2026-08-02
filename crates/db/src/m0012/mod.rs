use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0012",
    vec_box![super::m0011::Migration],
    vec_box![crate::user_login::m0012::AddAdConsentAt]
);
