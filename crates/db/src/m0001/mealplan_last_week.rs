use sea_query::{ColumnDef, Table, TableCreateStatement, TableDropStatement};

use crate::table::MealPlanLastWeek;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(MealPlanLastWeek::Table)
        .col(
            ColumnDef::new(MealPlanLastWeek::UserId)
                .string()
                .not_null()
                .string_len(26)
                .primary_key(),
        )
        .col(
            ColumnDef::new(MealPlanLastWeek::Start)
                .big_integer()
                .not_null(),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(MealPlanLastWeek::Table).to_owned()
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
