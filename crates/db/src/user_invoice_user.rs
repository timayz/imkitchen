use sea_query::Iden;

#[derive(Iden, Clone)]
pub enum UserInvoiceUser {
    Table,
    Id,
    InvoiceNumber,
    Cursor,
    UserId,
    IssuedAt,
    DueAt,
    From,
    To,
    Plan,
    ExpireAt,
    Price,
    Tax,
    TaxRate,
    TotalIncTax,
}

pub(crate) mod m0001 {
    use sea_query::{
        ColumnDef, Index, IndexCreateStatement, IndexDropStatement, Table, TableCreateStatement,
        TableDropStatement,
    };

    use super::UserInvoiceUser;

    pub struct CreateTable;

    fn create_table() -> TableCreateStatement {
        Table::create()
            .table(UserInvoiceUser::Table)
            .col(
                ColumnDef::new(UserInvoiceUser::Id)
                    .string()
                    .not_null()
                    .string_len(26)
                    .primary_key(),
            )
            .col(
                ColumnDef::new(UserInvoiceUser::InvoiceNumber)
                    .string()
                    .not_null()
                    .string_len(100)
                    .unique_key(),
            )
            .col(ColumnDef::new(UserInvoiceUser::Cursor).string().not_null())
            .col(
                ColumnDef::new(UserInvoiceUser::UserId)
                    .string()
                    .not_null()
                    .string_len(26),
            )
            .col(ColumnDef::new(UserInvoiceUser::From).blob().not_null())
            .col(ColumnDef::new(UserInvoiceUser::To).blob().not_null())
            .col(
                ColumnDef::new(UserInvoiceUser::Plan)
                    .string()
                    .not_null()
                    .string_len(100),
            )
            .col(ColumnDef::new(UserInvoiceUser::Price).integer().not_null())
            .col(ColumnDef::new(UserInvoiceUser::Tax).integer().not_null())
            .col(ColumnDef::new(UserInvoiceUser::TaxRate).float().not_null())
            .col(
                ColumnDef::new(UserInvoiceUser::TotalIncTax)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserInvoiceUser::DueAt)
                    .big_integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserInvoiceUser::IssuedAt)
                    .big_integer()
                    .null(),
            )
            .col(
                ColumnDef::new(UserInvoiceUser::ExpireAt)
                    .big_integer()
                    .null(),
            )
            .to_owned()
    }

    fn drop_table() -> TableDropStatement {
        Table::drop().table(UserInvoiceUser::Table).to_owned()
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
            .name("idx_user_invoice_user_6jagKS")
            .table(UserInvoiceUser::Table)
            .col(UserInvoiceUser::UserId)
            .to_owned()
    }

    fn drop_idx_1() -> IndexDropStatement {
        Index::drop()
            .name("idx_user_invoice_user_6jagKS")
            .table(UserInvoiceUser::Table)
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
}
