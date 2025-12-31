use sea_query::{
    ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
    TableDropStatement,
};

use crate::table::RecipeUser;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(RecipeUser::Table)
        .col(
            ColumnDef::new(RecipeUser::Id)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(RecipeUser::OwnerId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(RecipeUser::OwnerName)
                .string()
                .string_len(15),
        )
        .col(
            ColumnDef::new(RecipeUser::RecipeType)
                .string()
                .not_null()
                .string_len(25),
        )
        .col(
            ColumnDef::new(RecipeUser::CuisineType)
                .string()
                .not_null()
                .string_len(25),
        )
        .col(
            ColumnDef::new(RecipeUser::Name)
                .string()
                .not_null()
                .string_len(30),
        )
        .col(
            ColumnDef::new(RecipeUser::Description)
                .string()
                .not_null()
                .string_len(2000)
                .default(""),
        )
        .col(
            ColumnDef::new(RecipeUser::HouseholdSize)
                .integer()
                .not_null()
                .default(4),
        )
        .col(
            ColumnDef::new(RecipeUser::PrepTime)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(RecipeUser::CookTime)
                .integer()
                .not_null()
                .default(0),
        )
        .col(ColumnDef::new(RecipeUser::Ingredients).blob().not_null())
        .col(ColumnDef::new(RecipeUser::Instructions).blob().not_null())
        .col(
            ColumnDef::new(RecipeUser::DietaryRestrictions)
                .json_binary()
                .not_null(),
        )
        .col(
            ColumnDef::new(RecipeUser::AcceptsAccompaniment)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(RecipeUser::AdvancePrep)
                .string()
                .not_null()
                .string_len(2000)
                .default(""),
        )
        .col(
            ColumnDef::new(RecipeUser::IsShared)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(RecipeUser::TotalViews)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(RecipeUser::TotalLikes)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(RecipeUser::TotalComments)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(RecipeUser::CreatedAt)
                .big_integer()
                .not_null(),
        )
        .col(ColumnDef::new(RecipeUser::UpdatedAt).big_integer().null())
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(RecipeUser::Table).to_owned()
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
        .name("idx_recipe_list_6jagKS")
        .table(RecipeUser::Table)
        .col(RecipeUser::OwnerId)
        .col(RecipeUser::CuisineType)
        .to_owned()
}

fn drop_idx_1() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_list_6jagKS")
        .table(RecipeUser::Table)
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
        .name("idx_recipe_list_FctFNN")
        .table(RecipeUser::Table)
        .col(RecipeUser::OwnerId)
        .col(RecipeUser::RecipeType)
        .to_owned()
}

fn drop_idx_2() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_list_FctFNN")
        .table(RecipeUser::Table)
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
        .name("idx_recipe_list_KSDt5k")
        .table(RecipeUser::Table)
        .col(RecipeUser::OwnerId)
        .to_owned()
}

fn drop_idx_3() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_list_KSDt5k")
        .table(RecipeUser::Table)
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

pub struct CreateIdx4;

fn create_idx_4() -> IndexCreateStatement {
    Index::create()
        .name("idx_recipe_list_QJBhvl")
        .table(RecipeUser::Table)
        .col(RecipeUser::IsShared)
        .col(RecipeUser::CuisineType)
        .to_owned()
}

fn drop_idx_4() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_list_QJBhvl")
        .table(RecipeUser::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx4 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = create_idx_4().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = drop_idx_4().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }
}

pub struct CreateIdx5;

fn create_idx_5() -> IndexCreateStatement {
    Index::create()
        .name("idx_recipe_list_kXJfAR")
        .table(RecipeUser::Table)
        .col(RecipeUser::IsShared)
        .col(RecipeUser::RecipeType)
        .to_owned()
}

fn drop_idx_5() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_list_kXJfAR")
        .table(RecipeUser::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx5 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = create_idx_5().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = drop_idx_5().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }
}

pub struct CreateIdx6;

fn create_idx_6() -> IndexCreateStatement {
    Index::create()
        .name("idx_recipe_list_P4CTqO")
        .table(RecipeUser::Table)
        .col(RecipeUser::IsShared)
        .to_owned()
}

fn drop_idx_6() -> IndexDropStatement {
    Index::drop()
        .name("idx_recipe_list_P4CTqO")
        .table(RecipeUser::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreateIdx6 {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = create_idx_6().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statement = drop_idx_6().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statement).execute(connection).await?;

        Ok(())
    }
}
