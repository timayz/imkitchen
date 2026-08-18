use sqlx_migrator::vec_box;

pub struct Migration;

sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0013",
    vec_box![super::m0012::Migration],
    vec_box![
        crate::user_login::m0013::AddAggregateVersion,
        crate::user_admin::m0013::AddAggregateVersion,
        crate::contact_admin::m0013::AddAggregateVersion,
        crate::recipe_user::m0013::AddAggregateVersion,
        crate::user_invoice_user::m0013::AddAggregateVersion,
        crate::notification_recipient::m0013::AddAggregateVersion,
    ]
);
