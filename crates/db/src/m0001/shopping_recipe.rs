use sea_query::{
    ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
    TableDropStatement,
};

use crate::table::ShoppingRecipe;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(ShoppingRecipe::Table)
        .col(
            ColumnDef::new(ShoppingRecipe::Id)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(ShoppingRecipe::UserId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(ShoppingRecipe::HouseholdSize)
                .integer()
                .not_null()
                .default(4),
        )
        .col(
            ColumnDef::new(ShoppingRecipe::Ingredients)
                .blob()
                .not_null(),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(ShoppingRecipe::Table).to_owned()
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

pub struct CreateIdx1;

fn create_idx_1() -> IndexCreateStatement {
    Index::create()
        .name("idx_shopping_recipe_KSDt5k")
        .table(ShoppingRecipe::Table)
        .col(ShoppingRecipe::UserId)
        .to_owned()
}

fn drop_idx_1() -> IndexDropStatement {
    Index::drop()
        .name("idx_shopping_recipe_KSDt5k")
        .table(ShoppingRecipe::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx1 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = create_idx_1().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = drop_idx_1().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }
}
