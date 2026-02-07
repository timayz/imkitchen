use sea_query::{
    ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
    TableDropStatement,
};

use crate::table::RecipeThumbnail;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(RecipeThumbnail::Table)
        .col(
            ColumnDef::new(RecipeThumbnail::Id)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(RecipeThumbnail::Device)
                .string()
                .not_null()
                .string_len(10),
        )
        .col(
            ColumnDef::new(RecipeThumbnail::Data)
                .blob()
                .not_null()
                .string_len(30),
        )
        .primary_key(
            Index::create()
                .col(RecipeThumbnail::Id)
                .col(RecipeThumbnail::Device),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(RecipeThumbnail::Table).to_owned()
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
        .name("idx_recipe_thumbnail_6jagKS")
        .table(RecipeThumbnail::Table)
        .col(RecipeThumbnail::Id)
        .to_owned()
}

fn drop_idx_1() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_thumbnail_6jagKS")
        .table(RecipeThumbnail::Table)
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
