use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum ShoppingSlot {
    Table,
    UserId,
    Date,
    RecipeIds,
}

pub(crate) mod m0001 {
    use sea_query::{ColumnDef, Index, Table, TableCreateStatement, TableDropStatement};

    use super::ShoppingSlot;

    pub struct CreateTable;

    fn create_table() -> TableCreateStatement {
        Table::create()
            .table(ShoppingSlot::Table)
            .col(
                ColumnDef::new(ShoppingSlot::UserId)
                    .string()
                    .not_null()
                    .string_len(26),
            )
            .col(ColumnDef::new(ShoppingSlot::Date).big_integer().not_null())
            .col(ColumnDef::new(ShoppingSlot::RecipeIds).blob().not_null())
            .primary_key(
                Index::create()
                    .col(ShoppingSlot::UserId)
                    .col(ShoppingSlot::Date),
            )
            .to_owned()
    }

    fn drop_table() -> TableDropStatement {
        Table::drop().table(ShoppingSlot::Table).to_owned()
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
}
