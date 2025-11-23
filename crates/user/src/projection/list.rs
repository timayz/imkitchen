use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_db::table::UserList;
use imkitchen_shared::Event;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

use crate::{
    Activated, MadeAdmin, RegistrationSucceeded, Role, State, Suspended, User,
    subscription::{LifePremiumToggled, UserSubscription},
};

pub fn subscribe_list<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("user-list")
        .handler(handle_registration_succeeded())
        .handler(handle_suspended())
        .handler(handle_activated())
        .handler(handle_made_admin())
        .handler(handle_toggle_life_premium())
        .handler_check_off()
}

#[evento::handler(User)]
async fn handle_registration_succeeded<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<RegistrationSucceeded>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::insert()
        .into_table(UserList::Table)
        .columns([
            UserList::Id,
            UserList::Email,
            UserList::State,
            UserList::Role,
            UserList::CreatedAt,
        ])
        .values_panic([
            event.aggregator_id.to_owned().into(),
            event.data.email.to_owned().into(),
            State::Active.to_string().into(),
            Role::User.to_string().into(),
            event.timestamp.into(),
        ])
        .to_owned();
    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(User)]
async fn handle_suspended<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Suspended>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(UserList::Table)
        .values([(UserList::State, State::Suspended.to_string().into())])
        .and_where(Expr::col(UserList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(User)]
async fn handle_activated<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<Activated>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(UserList::Table)
        .values([(UserList::State, State::Active.to_string().into())])
        .and_where(Expr::col(UserList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(User)]
async fn handle_made_admin<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<MadeAdmin>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    let statment = Query::update()
        .table(UserList::Table)
        .values([(UserList::Role, Role::Admin.to_string().into())])
        .and_where(Expr::col(UserList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}

#[evento::handler(UserSubscription)]
async fn handle_toggle_life_premium<E: Executor>(
    context: &evento::Context<'_, E>,
    event: Event<LifePremiumToggled>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();

    let statment = Query::update()
        .table(UserList::Table)
        .values([(UserList::SubscriptionExpireAt, event.data.expire_at.into())])
        .and_where(Expr::col(UserList::Id).eq(&event.aggregator_id))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
