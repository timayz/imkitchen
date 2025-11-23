use sea_query::{
    ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
    TableDropStatement,
};

use crate::table::RecipeList;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(RecipeList::Table)
        .col(
            ColumnDef::new(RecipeList::Id)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(RecipeList::UserId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(RecipeList::RecipeType)
                .string()
                .not_null()
                .string_len(25),
        )
        .col(
            ColumnDef::new(RecipeList::CuisineType)
                .string()
                .not_null()
                .string_len(25),
        )
        .col(
            ColumnDef::new(RecipeList::Name)
                .string()
                .not_null()
                .string_len(30),
        )
        .col(
            ColumnDef::new(RecipeList::Description)
                .string()
                .not_null()
                .string_len(2000)
                .default(""),
        )
        .col(
            ColumnDef::new(RecipeList::PrepTime)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(RecipeList::CookTime)
                .integer()
                .not_null()
                .default(0),
        )
        .col(ColumnDef::new(RecipeList::Ingredients).blob().not_null())
        .col(ColumnDef::new(RecipeList::Instructions).blob().not_null())
        .col(
            ColumnDef::new(RecipeList::DietaryRestrictions)
                .json_binary()
                .not_null(),
        )
        .col(
            ColumnDef::new(RecipeList::AcceptsAccompaniment)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(RecipeList::PreferredAccompanimentTypes)
                .json_binary()
                .not_null(),
        )
        .col(
            ColumnDef::new(RecipeList::AdvancePrep)
                .string()
                .not_null()
                .string_len(2000)
                .default(""),
        )
        .col(
            ColumnDef::new(RecipeList::IsShared)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(RecipeList::CreatedAt)
                .big_integer()
                .not_null(),
        )
        .col(ColumnDef::new(RecipeList::UpdatedAt).big_integer().null())
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(RecipeList::Table).to_owned()
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

pub struct CreateIdx1;

fn create_idx_1() -> IndexCreateStatement {
    Index::create()
        .name("idx_recipe_list_6jagKS")
        .table(RecipeList::Table)
        .col(RecipeList::UserId)
        .col(RecipeList::CuisineType)
        .to_owned()
}

fn drop_idx_1() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_list_6jagKS")
        .table(RecipeList::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx1 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = create_idx_1().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = drop_idx_1().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }
}

pub struct CreateIdx2;

fn create_idx_2() -> IndexCreateStatement {
    Index::create()
        .name("idx_recipe_list_FctFNN")
        .table(RecipeList::Table)
        .col(RecipeList::UserId)
        .col(RecipeList::RecipeType)
        .to_owned()
}

fn drop_idx_2() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_list_FctFNN")
        .table(RecipeList::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx2 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = create_idx_2().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = drop_idx_2().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }
}
pub struct CreateIdx3;

fn create_idx_3() -> IndexCreateStatement {
    Index::create()
        .name("idx_recipe_list_KSDt5k")
        .table(RecipeList::Table)
        .col(RecipeList::UserId)
        .to_owned()
}

fn drop_idx_3() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_list_KSDt5k")
        .table(RecipeList::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx3 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = create_idx_3().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = drop_idx_3().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }
}
