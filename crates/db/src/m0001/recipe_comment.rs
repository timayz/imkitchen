use sea_query::{
    ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
    TableDropStatement,
};

use crate::table::RecipeComment;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(RecipeComment::Table)
        .col(
            ColumnDef::new(RecipeComment::Id)
                .string()
                .not_null()
                .string_len(64)
                .primary_key(),
        )
        .col(ColumnDef::new(RecipeComment::Cursor).string().not_null())
        .col(
            ColumnDef::new(RecipeComment::RecipeId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(RecipeComment::ReplyTo)
                .string()
                .null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(RecipeComment::OwnerId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(RecipeComment::OwnerName)
                .string()
                .string_len(15),
        )
        .col(
            ColumnDef::new(RecipeComment::Body)
                .string()
                .not_null()
                .string_len(2000),
        )
        .col(
            ColumnDef::new(RecipeComment::TotalLikes)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(RecipeComment::CreatedAt)
                .big_integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(RecipeComment::UpdatedAt)
                .big_integer()
                .null(),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(RecipeComment::Table).to_owned()
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
        .name("idx_recipe_comment_6jagKS")
        .table(RecipeComment::Table)
        .col(RecipeComment::RecipeId)
        .to_owned()
}

fn drop_idx_1() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_comment_6jagKS")
        .table(RecipeComment::Table)
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

pub struct CreateIdx2;

fn create_idx_2() -> IndexCreateStatement {
    Index::create()
        .name("idx_recipe_comment_KSDt5k")
        .table(RecipeComment::Table)
        .col(RecipeComment::OwnerId)
        .to_owned()
}

fn drop_idx_2() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_comment_KSDt5k")
        .table(RecipeComment::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx2 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = create_idx_2().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = drop_idx_2().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }
}

pub struct CreateIdx3;

fn create_idx_3() -> IndexCreateStatement {
    Index::create()
        .name("idx_recipe_comment_cBSg0k")
        .table(RecipeComment::Table)
        .col(RecipeComment::ReplyTo)
        .to_owned()
}

fn drop_idx_3() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_comment_cBSg0k")
        .table(RecipeComment::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx3 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = create_idx_3().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = drop_idx_3().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }
}
