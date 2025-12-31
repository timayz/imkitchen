use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

use crate::table::RecipeCommand;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(RecipeCommand::Table)
        .col(
            ColumnDef::new(RecipeCommand::Id)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(RecipeCommand::OwnerId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(RecipeCommand::RecipeType)
                .string()
                .not_null()
                .string_len(25),
        )
        .col(
            ColumnDef::new(RecipeCommand::CuisineType)
                .string()
                .not_null()
                .string_len(25),
        )
        .col(
            ColumnDef::new(RecipeCommand::BasicInformationHash)
                .string()
                .not_null()
                .default(""),
        )
        .col(
            ColumnDef::new(RecipeCommand::IngredientsHash)
                .string()
                .not_null()
                .default(""),
        )
        .col(
            ColumnDef::new(RecipeCommand::InstructionsHash)
                .string()
                .not_null()
                .default(""),
        )
        .col(
            ColumnDef::new(RecipeCommand::DietaryRestrictionsHash)
                .string()
                .not_null()
                .default(""),
        )
        .col(
            ColumnDef::new(RecipeCommand::AcceptsAccompaniment)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(RecipeCommand::AdvancePrepHash)
                .string()
                .not_null()
                .default(""),
        )
        .col(
            ColumnDef::new(RecipeCommand::IsShared)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(RecipeCommand::IsDeleted)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(ColumnDef::new(RecipeCommand::Version).integer().not_null())
        .col(
            ColumnDef::new(RecipeCommand::RoutingKey)
                .string()
                .string_len(50),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(RecipeCommand::Table).to_owned()
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
