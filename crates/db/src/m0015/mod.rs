use sqlx_migrator::vec_box;

pub struct Migration;

// The calendar meal plan was replaced by the per-user recipe list: drop its
// read models and forget the cursors of the subscriptions that fed them.
sqlx_migrator::sqlite_migration!(
    Migration,
    "imkitchen",
    "m0015",
    vec_box![super::m0014::Migration],
    vec_box![
        crate::mealplan_slot::m0015::DropTable,
        crate::shopping_slot::m0015::DropTable,
        crate::shopping_list::m0015::DropTable,
        ForgetSubscribers,
    ]
);

/// Remove the `mealplan-slot` and `shopping-list` subscriber rows so a
/// re-introduced key would start from scratch rather than resume mid-stream.
pub struct ForgetSubscribers;

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for ForgetSubscribers {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        sqlx::query("DELETE FROM subscriber WHERE key IN ('mealplan-slot', 'shopping-list')")
            .execute(connection)
            .await?;

        Ok(())
    }

    async fn down(
        &self,
        _connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        Ok(())
    }
}
