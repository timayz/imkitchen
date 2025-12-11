use sea_query::{ColumnDef, Index, Table, TableCreateStatement, TableDropStatement};

use crate::table::MealPlanSlot;

pub struct CreateTable;

fn create_table() -> TableCreateStatement {
    Table::create()
        .table(MealPlanSlot::Table)
        .col(
            ColumnDef::new(MealPlanSlot::UserId)
                .string()
                .not_null()
                .string_len(26),
        )
        .col(ColumnDef::new(MealPlanSlot::Day).big_integer().not_null())
        .col(ColumnDef::new(MealPlanSlot::MainCourse).blob().not_null())
        .col(ColumnDef::new(MealPlanSlot::Appetizer).blob().null())
        .col(ColumnDef::new(MealPlanSlot::Accompaniment).blob().null())
        .col(ColumnDef::new(MealPlanSlot::Dessert).blob().null())
        .primary_key(
            Index::create()
                .col(MealPlanSlot::UserId)
                .col(MealPlanSlot::Day),
        )
        .to_owned()
}

fn drop_table() -> TableDropStatement {
    Table::drop().table(MealPlanSlot::Table).to_owned()
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
