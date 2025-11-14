mod admin_user_pjt_create_account_type_idx;
mod admin_user_pjt_create_status_idx;
mod admin_user_pjt_create_table;
mod contact_pjt_create_status_idx;
mod contact_pjt_create_subject_idx;
mod contact_pjt_create_table;
mod global_stat_pjt_create_table;
mod user_create_email_idx;
mod user_create_table;

use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "main",
    "m0_9",
    vec_box![],
    vec_box![
        global_stat_pjt_create_table::Operation,
        user_create_table::Operation,
        user_create_email_idx::Operation,
        admin_user_pjt_create_table::Operation,
        admin_user_pjt_create_status_idx::Operation,
        admin_user_pjt_create_account_type_idx::Operation,
        contact_pjt_create_table::Operation,
        contact_pjt_create_subject_idx::Operation,
        contact_pjt_create_status_idx::Operation,
    ]
);
