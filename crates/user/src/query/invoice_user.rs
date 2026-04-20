use evento::{
    Cursor, Executor, Projection, Snapshot,
    cursor::{Args, ReadResult},
    metadata::Event,
    sql::Reader,
};
use imkitchen_db::table::UserInvoiceUser;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::{SqlitePool, prelude::FromRow};

use imkitchen_shared::user::invoice::{Created, Invoice, InvoiceAddress};

impl<E: Executor> super::Query<E> {
    pub async fn invoice(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<InvoiceUserView>, anyhow::Error> {
        load(&self.executor, &self.read_db, &self.write_db, id).await
    }
}

pub(crate) async fn load<E: Executor>(
    executor: &E,
    read_db: &SqlitePool,
    write_db: &SqlitePool,
    id: impl Into<String>,
) -> Result<Option<InvoiceUserView>, anyhow::Error> {
    let id = id.into();

    create_projection(&id)
        .aggregator::<Invoice>(id)
        .data((read_db.clone(), write_db.clone()))
        .execute(executor)
        .await
}

#[evento::projection(FromRow, Cursor, Debug)]
pub struct InvoiceUserView {
    #[cursor(UserInvoiceUser::Id, 1)]
    pub id: String,
    pub invoice_number: String,
    pub user_id: String,
    pub due_at: u64,
    #[cursor(UserInvoiceUser::IssuedAt, 2)]
    pub issued_at: u64,
    pub from: evento::sql_types::Bitcode<InvoiceAddress>,
    pub to: evento::sql_types::Bitcode<InvoiceAddress>,
    pub plan: String,
    pub expire_at: u64,
    pub price: u32,
    pub tax: u32,
    pub tax_rate: f64,
    pub total_inc_tax: u32,
}

pub struct FilterQuery {
    pub user_id: String,
    pub args: Args,
}

impl<E: Executor> super::Query<E> {
    pub async fn filter_invoice(
        &self,
        input: FilterQuery,
    ) -> anyhow::Result<ReadResult<InvoiceUserView>> {
        let statement = sea_query::Query::select()
            .columns([
                UserInvoiceUser::Id,
                UserInvoiceUser::InvoiceNumber,
                UserInvoiceUser::Cursor,
                UserInvoiceUser::UserId,
                UserInvoiceUser::IssuedAt,
                UserInvoiceUser::DueAt,
                UserInvoiceUser::ExpireAt,
                UserInvoiceUser::From,
                UserInvoiceUser::To,
                UserInvoiceUser::Plan,
                UserInvoiceUser::Price,
                UserInvoiceUser::Tax,
                UserInvoiceUser::TaxRate,
                UserInvoiceUser::TotalIncTax,
            ])
            .from(UserInvoiceUser::Table)
            .and_where(Expr::col(UserInvoiceUser::UserId).eq(&input.user_id))
            .to_owned();

        Reader::new(statement)
            .desc()
            .args(input.args)
            .execute::<_, InvoiceUserView, _>(&self.read_db)
            .await
    }
}

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, InvoiceUserView> {
    Projection::new::<Invoice>(id).handler(handle_created())
}

impl<E: Executor> Snapshot<E> for InvoiceUserView {
    async fn restore(context: &evento::projection::Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let (read_db, _) = context.extract::<(SqlitePool, SqlitePool)>();
        let statement = sea_query::Query::select()
            .columns([
                UserInvoiceUser::Id,
                UserInvoiceUser::InvoiceNumber,
                UserInvoiceUser::Cursor,
                UserInvoiceUser::UserId,
                UserInvoiceUser::IssuedAt,
                UserInvoiceUser::DueAt,
                UserInvoiceUser::ExpireAt,
                UserInvoiceUser::From,
                UserInvoiceUser::To,
                UserInvoiceUser::Plan,
                UserInvoiceUser::Price,
                UserInvoiceUser::Tax,
                UserInvoiceUser::TaxRate,
                UserInvoiceUser::TotalIncTax,
            ])
            .from(UserInvoiceUser::Table)
            .and_where(Expr::col(UserInvoiceUser::InvoiceNumber).eq(&context.id))
            .limit(1)
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

        Ok(sqlx::query_as_with(&sql, values)
            .fetch_optional(&read_db)
            .await?)
    }

    async fn take_snapshot(
        &self,
        context: &evento::projection::Context<'_, E>,
    ) -> anyhow::Result<()> {
        let (_, write_db) = context.extract::<(SqlitePool, SqlitePool)>();

        let from = bitcode::encode(&self.from.0);
        let to = bitcode::encode(&self.to.0);

        let statement = Query::insert()
            .into_table(UserInvoiceUser::Table)
            .columns([
                UserInvoiceUser::Id,
                UserInvoiceUser::InvoiceNumber,
                UserInvoiceUser::Cursor,
                UserInvoiceUser::UserId,
                UserInvoiceUser::IssuedAt,
                UserInvoiceUser::DueAt,
                UserInvoiceUser::ExpireAt,
                UserInvoiceUser::From,
                UserInvoiceUser::To,
                UserInvoiceUser::Plan,
                UserInvoiceUser::Price,
                UserInvoiceUser::Tax,
                UserInvoiceUser::TaxRate,
                UserInvoiceUser::TotalIncTax,
            ])
            .values([
                self.id.to_owned().into(),
                self.invoice_number.to_owned().into(),
                self.cursor.to_owned().into(),
                self.user_id.to_owned().into(),
                self.issued_at.into(),
                self.due_at.into(),
                self.expire_at.into(),
                from.into(),
                to.into(),
                self.plan.to_owned().into(),
                self.price.into(),
                self.tax.into(),
                self.tax_rate.into(),
                self.total_inc_tax.into(),
            ])?
            .on_conflict(
                OnConflict::column(UserInvoiceUser::Id)
                    .update_columns([
                        UserInvoiceUser::InvoiceNumber,
                        UserInvoiceUser::Cursor,
                        UserInvoiceUser::UserId,
                        UserInvoiceUser::IssuedAt,
                        UserInvoiceUser::DueAt,
                        UserInvoiceUser::ExpireAt,
                        UserInvoiceUser::From,
                        UserInvoiceUser::To,
                        UserInvoiceUser::Plan,
                        UserInvoiceUser::Price,
                        UserInvoiceUser::Tax,
                        UserInvoiceUser::TaxRate,
                        UserInvoiceUser::TotalIncTax,
                    ])
                    .to_owned(),
            )
            .to_owned();

        let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
        sqlx::query_with(&sql, values).execute(&write_db).await?;

        Ok(())
    }
}

#[evento::handler]
async fn handle_created(event: Event<Created>, data: &mut InvoiceUserView) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.invoice_number = format!("{}-{:02}", event.data.key, event.data.number);
    data.user_id = event.metadata.requested_by()?;
    data.due_at = event.data.paid_at;
    data.issued_at = event.data.paid_at;
    data.from.0 = event.data.from;
    data.to.0 = event.data.to;
    data.plan = event.data.details.plan.to_owned();
    data.expire_at = event.data.expire_at;
    data.price = event.data.details.price;
    data.tax = event.data.details.tax;
    data.tax_rate = event.data.details.tax_rate.unwrap_or_default();
    data.total_inc_tax = event.data.details.price + event.data.details.tax;

    Ok(())
}
