use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

use crate::table::RecipePjt;

pub struct Operation;

fn up_statement() -> TableCreateStatement {
    Table::create()
        .table(RecipePjt::Table)
        .col(
            ColumnDef::new(RecipePjt::Id)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(RecipePjt::UserId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(
            ColumnDef::new(RecipePjt::RecipeType)
                .string()
                .not_null()
                .string_len(25),
        )
        .col(
            ColumnDef::new(RecipePjt::CuisineType)
                .string()
                .not_null()
                .string_len(25),
        )
        .col(
            ColumnDef::new(RecipePjt::Name)
                .string()
                .not_null()
                .string_len(30),
        )
        .col(
            ColumnDef::new(RecipePjt::Description)
                .string()
                .not_null()
                .string_len(2000)
                .default(""),
        )
        .col(
            ColumnDef::new(RecipePjt::PrepTime)
                .integer()
                .not_null()
                .default(0),
        )
        .col(
            ColumnDef::new(RecipePjt::CookTime)
                .integer()
                .not_null()
                .default(0),
        )
        .col(ColumnDef::new(RecipePjt::Ingredients).blob().not_null())
        .col(ColumnDef::new(RecipePjt::Instructions).blob().not_null())
        .col(
            ColumnDef::new(RecipePjt::DietaryRestrictions)
                .json_binary()
                .not_null(),
        )
        .col(
            ColumnDef::new(RecipePjt::AcceptAccompaniments)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(RecipePjt::PreferredAccompanimentTypes)
                .json_binary()
                .not_null(),
        )
        .col(
            ColumnDef::new(RecipePjt::AdvancePreparation)
                .string()
                .not_null()
                .string_len(2000)
                .default(""),
        )
        .col(
            ColumnDef::new(RecipePjt::IsShared)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(RecipePjt::CreatedAt)
                .big_integer()
                .not_null(),
        )
        .col(ColumnDef::new(RecipePjt::UpdatedAt).big_integer().null())
        .to_owned()
}

fn down_statement() -> TableDropStatement {
    Table::drop().table(RecipePjt::Table).to_owned()
}

#[async_trait::async_trait]
impl sqlx_migrator::Operation<sqlx::Sqlite> for Operation {
    async fn up(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = up_statement().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }

    async fn down(
        &self,
        connection: &mut sqlx::SqliteConnection,
    ) -> Result<(), sqlx_migrator::Error> {
        let statment = down_statement().to_string(sea_query::SqliteQueryBuilder);
        sqlx::query(&statment).execute(connection).await?;

        Ok(())
    }
}
