use sea_query::{ColumnDef, Index, Table, TableCreateStatement, TableDropStatement};

use crate::table::MealPlanWeek;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(MealPlanWeek::Table)
        .col(
            ColumnDef::new(MealPlanWeek::UserId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(ColumnDef::new(MealPlanWeek::Start).big_integer().not_null())
        .col(ColumnDef::new(MealPlanWeek::End).big_integer().not_null())
        .col(
            ColumnDef::new(MealPlanWeek::Status)
                .string()
                .not_null()
                .string_len(25),
        )
        .col(ColumnDef::new(MealPlanWeek::Slots).blob().not_null())
        .primary_key(
            Index::create()
                .col(MealPlanWeek::UserId)
                .col(MealPlanWeek::Start),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(MealPlanWeek::Table).to_owned()
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
