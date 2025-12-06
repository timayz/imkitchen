use sea_query::{ColumnDef, Index, Table, TableCreateStatement, TableDropStatement};

use crate::table::ShoppingList;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(ShoppingList::Table)
        .col(
            ColumnDef::new(ShoppingList::UserId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(ColumnDef::new(ShoppingList::Week).big_integer().not_null())
        .col(ColumnDef::new(ShoppingList::Ingredients).blob().not_null())
        .primary_key(
            Index::create()
                .col(ShoppingList::UserId)
                .col(ShoppingList::Week),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(ShoppingList::Table).to_owned()
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
