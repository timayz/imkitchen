use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum RecipeUserStat {
    Table,
    UserId,
    Total,
    Shared,
    Favorite,
    FromCommunity,
    CreatedAt,
}

pub(crate) mod m0001 {
    use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

    use super::RecipeUserStat;

    pub struct CreateTable;

    fn create_table() -> TableCreateStatement {
        Table::create()
            .table(RecipeUserStat::Table)
            .col(
                ColumnDef::new(RecipeUserStat::UserId)
                    .string()
                    .string_len(26)
                    .null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(RecipeUserStat::Total)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(RecipeUserStat::Shared)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(RecipeUserStat::Favorite)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .col(
                ColumnDef::new(RecipeUserStat::FromCommunity)
                    .integer()
                    .not_null()
                    .default(0),
            )
            .to_owned()
    }

    fn drop_table() -> TableDropStatement {
        Table::drop().table(RecipeUserStat::Table).to_owned()
    }

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateTable {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = create_table().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }

        async fn down(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            let statement = drop_table().to_string(sea_query::SqliteQueryBuilder);
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(connection)
                .await?;

            Ok(())
        }
    }
}

pub(crate) mod m0005 {
    /// Rebuilds `recipe_user_stat` from each recipe's own events. The
    /// recipe-user-stat-view subscription no longer handles the bulk
    /// `AllSharedToCommunity` / `AllMadePrivate` events — the recipe-saga-share
    /// saga now turns those into per-recipe `SharedToCommunity` / `MadePrivate`
    /// events — so the `shared` count is recomputed purely from per-recipe
    /// events to avoid double-counting the saga's historical backfill.
    pub struct Rebuild;

    #[async_trait::async_trait]
    impl sqlx_migrator::Operation<sqlx::Sqlite> for Rebuild {
        async fn up(
            &self,
            connection: &mut sqlx::SqliteConnection,
        ) -> Result<(), sqlx_migrator::Error> {
            sqlx::query("DELETE FROM recipe_user_stat")
                .execute(&mut *connection)
                .await?;

            sqlx::query("UPDATE subscriber SET cursor = NULL WHERE key = 'recipe-user-stat-view'")
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
