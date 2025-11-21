use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

use crate::table::ContactStat;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(ContactStat::Table)
        .col(
            ColumnDef::new(ContactStat::Today)
                .big_integer()
                .null()
                .primary_key(),
        )
        .col(
            ColumnDef::new(ContactStat::Total)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(ContactStat::Unread)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(ContactStat::AvgResponseTime)
                .integer()
                .not_null()
                .default(0),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(ContactStat::Table).to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateTable {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = create_table().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = drop_table().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }
}
