//! `recipe_owner` — a minimal `recipe_id → owner_id` index maintained by the
//! recipe-saga-share saga so it can fan a bulk "share all" out to per-recipe
//! events without reading the recipe-query read model (which may be mid-rebuild).

use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum RecipeOwner {
    Table,
    RecipeId,
    OwnerId,
}

pub(crate) mod m0005 {
    use sea_query::{ColumnDef, Index, SqliteQueryBuilder, Table};

    use super::RecipeOwner;

    pub struct CreateTable;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateTable {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let table = Table::create()
                .table(RecipeOwner::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(RecipeOwner::RecipeId)
                        .string()
                        .not_null()
                        .string_len(26)
                        .primary_key(),
                )
                .col(
                    ColumnDef::new(RecipeOwner::OwnerId)
                        .string()
                        .not_null()
                        .string_len(26),
                )
                .to_owned();

            sqlx::query(sqlx::AssertSqlSafe(table.to_string(SqliteQueryBuilder)))
                .execute(&mut *connection)
                .await?;

            let index = Index::create()
                .if_not_exists()
                .name("idx_recipe_owner_owner")
                .table(RecipeOwner::Table)
                .col(RecipeOwner::OwnerId)
                .to_owned();

            sqlx::query(sqlx::AssertSqlSafe(index.to_string(SqliteQueryBuilder)))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let table = Table::drop()
                .table(RecipeOwner::Table)
                .if_exists()
                .to_owned();

            sqlx::query(sqlx::AssertSqlSafe(table.to_string(SqliteQueryBuilder)))
                .execute(connection)
                .await?;

            Ok(())
        }
    }

    /// Drops the cursor row of the retired `recipe-query-share` subscription. It
    /// was replaced by the `recipe-saga-share` saga, which starts fresh (cursor
    /// NULL) and replays history to emit the per-recipe share events.
    ///
    /// `subscriber` is evento's internal table with no `Iden`, so the delete is
    /// raw SQL — the same way m0002/m0004 touch it.
    pub struct RemoveLegacyShareSubscriber;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for RemoveLegacyShareSubscriber {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query("DELETE FROM subscriber WHERE key = 'recipe-query-share'")
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
}
