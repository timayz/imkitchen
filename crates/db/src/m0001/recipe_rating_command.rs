use sea_query::{
    ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
    TableDropStatement,
};

use crate::table::RecipeRatingCommand;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(RecipeRatingCommand::Table)
        .col(
            ColumnDef::new(RecipeRatingCommand::RecipeId)
                .string()
                .string_len(26)
                .null(),
        )
        .col(
            ColumnDef::new(RecipeRatingCommand::UserId)
                .string()
                .string_len(26)
                .null(),
        )
        .col(
            ColumnDef::new(RecipeRatingCommand::Viewed)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(RecipeRatingCommand::Liked)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(RecipeRatingCommand::Unliked)
                .boolean()
                .not_null()
                .default(false),
        )
        .primary_key(
            Index::create()
                .col(RecipeRatingCommand::RecipeId)
                .col(RecipeRatingCommand::UserId),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(RecipeRatingCommand::Table).to_owned()
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
        .name("idx_recipe_rating_P4CTqO")
        .table(RecipeRatingCommand::Table)
        .col(RecipeRatingCommand::RecipeId)
        .to_owned()
}

fn drop_idx_1() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_rating_P4CTqO")
        .table(RecipeRatingCommand::Table)
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
