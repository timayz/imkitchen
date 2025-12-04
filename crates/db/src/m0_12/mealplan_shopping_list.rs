use sea_query::{ColumnDef, Index, Table, TableCreateStatement, TableDropStatement};

use crate::table::MealPlanShoppingList;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(MealPlanShoppingList::Table)
        .col(
            ColumnDef::new(MealPlanShoppingList::UserId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(MealPlanShoppingList::Week)
                .big_integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(MealPlanShoppingList::Ingredients)
                .blob()
                .not_null(),
        )
        .primary_key(
            Index::create()
                .col(MealPlanShoppingList::UserId)
                .col(MealPlanShoppingList::Week),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(MealPlanShoppingList::Table).to_owned()
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
