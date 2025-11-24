use sea_query::{
    ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
    TableDropStatement,
};

use crate::table::MealPlanRecipe;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(MealPlanRecipe::Table)
        .col(
            ColumnDef::new(MealPlanRecipe::Id)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(MealPlanRecipe::UserId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(MealPlanRecipe::RecipeType)
                .string()
                .not_null()
                .string_len(25),
        )
        .col(
            ColumnDef::new(MealPlanRecipe::Name)
                .string()
                .not_null()
                .string_len(30),
        )
        .col(
            ColumnDef::new(MealPlanRecipe::PrepTime)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(MealPlanRecipe::CookTime)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(MealPlanRecipe::Ingredients)
                .blob()
                .not_null(),
        )
        .col(
            ColumnDef::new(MealPlanRecipe::Instructions)
                .blob()
                .not_null(),
        )
        .col(
            ColumnDef::new(MealPlanRecipe::AdvancePrep)
                .string()
                .not_null()
                .string_len(2000)
                .default(""),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(MealPlanRecipe::Table).to_owned()
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

pub struct CreatePk;

fn create_pk() -> IndexCreateStatement {
    Index::create()
        .name("pk_mealplan_recipe")
        .table(MealPlanRecipe::Table)
        .col(MealPlanRecipe::UserId)
        .col(MealPlanRecipe::Id)
        .to_owned()
}

fn drop_pk() -> IndexDropStatement {
    Index::drop()
        .name("pk_mealplan_recipe")
        .table(MealPlanRecipe::Table)
        .to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for CreatePk {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = create_pk().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = drop_pk().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }
}

pub struct CreateIdx1;

fn create_idx_1() -> IndexCreateStatement {
    Index::create()
        .name("idk_mealplan_recipe_wrA7kG")
        .table(MealPlanRecipe::Table)
        .col(MealPlanRecipe::Id)
        .to_owned()
}

fn drop_idx_1() -> IndexDropStatement {
    Index::drop()
        .name("idk_mealplan_recipe_wrA7kG")
        .table(MealPlanRecipe::Table)
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
        .name("idk_mealplan_recipe_lxvLay")
        .table(MealPlanRecipe::Table)
        .col(MealPlanRecipe::UserId)
        .to_owned()
}

fn drop_idx_2() -> IndexDropStatement {
    Index::drop()
        .name("idk_mealplan_recipe_lxvLay")
        .table(MealPlanRecipe::Table)
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
        .name("idk_mealplan_recipe_GffMLT")
        .table(MealPlanRecipe::Table)
        .col(MealPlanRecipe::UserId)
        .col(MealPlanRecipe::RecipeType)
        .to_owned()
}

fn drop_idx_3() -> IndexDropStatement {
    Index::drop()
        .name("idk_mealplan_recipe_GffMLT")
        .table(MealPlanRecipe::Table)
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
