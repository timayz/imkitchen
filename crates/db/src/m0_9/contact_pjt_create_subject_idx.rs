use sea_query::{Index, IndexCreateStatement, IndexDropStatement};

use crate::table::ContactPjt;

pub struct Operation;

fn up_statement() -> IndexCreateStatement {
    Index::create()
        .name("idx_contact_subject")
        .table(ContactPjt::Table)
        .col(ContactPjt::Subject)
        .to_owned()
}

fn down_statement() -> IndexDropStatement {
    Index::drop()
        .name("idx_contact_subject")
        .table(ContactPjt::Table)
        .to_owned()
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
