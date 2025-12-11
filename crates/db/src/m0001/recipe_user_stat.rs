use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

use crate::table::RecipeUserStat;

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
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = drop_table().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }
}
