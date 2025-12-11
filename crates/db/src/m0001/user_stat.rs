use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

use crate::table::UserStat;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(UserStat::Table)
        .col(ColumnDef::new(UserStat::Day).big_integer().primary_key())
        .col(
            ColumnDef::new(UserStat::Total)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(UserStat::Premium)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(UserStat::Suspended)
                .integer()
                .not_null()
                .default(0),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(UserStat::Table).to_owned()
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
