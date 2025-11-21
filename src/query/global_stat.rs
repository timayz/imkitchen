use std::collections::HashMap;

use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_db::table::GlobalStatPjt;
use imkitchen_shared::Event;
use imkitchen_user::{RegistrationSucceeded, User};
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;

#[derive(Default)]
pub struct AdminUserGlobalStats {
    pub total: u32,
    pub total_percent: u8,
    pub premium: u32,
    pub premium_percent: i8,
    pub active_today: u32,
    pub active_today_percent: u8,
    pub suspended: u32,
    pub suspended_percent: u8,
}

pub async fn query_admin_users_global_stats(
    pool: &sqlx::SqlitePool,
) -> anyhow::Result<AdminUserGlobalStats> {
    let stats = query_global_stats(
        pool,
        vec![
            "total_users",
            "premium_users",
            "active_today_users",
            "suspended_users",
        ],
    )
    .await?;

    Ok(AdminUserGlobalStats {
        total: stats.get("total_users").unwrap_or(&0).to_owned(),
        total_percent: 0,
        premium: stats.get("premium_users").unwrap_or(&0).to_owned(),
        premium_percent: 0,
        active_today: stats.get("active_today_users").unwrap_or(&0).to_owned(),
        active_today_percent: 0,
        suspended: stats.get("suspended_users").unwrap_or(&0).to_owned(),
        suspended_percent: 0,
    })
}

#[derive(Default)]
pub struct UserRecipeGlobalStats {
    pub total: u32,
    pub favorite: u32,
    pub shared: u32,
    pub from_community: u32,
}

pub async fn query_user_recipe_global_stats(
    pool: &sqlx::SqlitePool,
    id: impl Into<String>,
) -> anyhow::Result<UserRecipeGlobalStats> {
    let id = id.into();
    let stats = query_global_stats(
        pool,
        vec![
            format!("total_recipes_{id}"),
            format!("shared_recipes_{id}"),
        ],
    )
    .await?;

    Ok(UserRecipeGlobalStats {
        total: stats
            .get(&format!("total_recipes_{id}"))
            .unwrap_or(&0)
            .to_owned(),
        shared: stats
            .get(&format!("shared_recipes_{id}"))
            .unwrap_or(&0)
            .to_owned(),
        favorite: 0,
        from_community: 0,
    })
}

async fn query_global_stats<I, V>(
    pool: &sqlx::SqlitePool,
    keys: I,
) -> anyhow::Result<HashMap<String, u32>>
where
    V: Into<Expr>,
    I: IntoIterator<Item = V>,
{
    let statment = Query::select()
        .columns([GlobalStatPjt::Key, GlobalStatPjt::Value])
        .from(GlobalStatPjt::Table)
        .and_where(Expr::col(GlobalStatPjt::Key).is_in::<V, I>(keys))
        .to_owned();

    let (sql, values) = statment.build_sqlx(SqliteQueryBuilder);
    let rows = sqlx::query_as_with::<_, (String, u32), _>(&sql, values)
        .fetch_all(pool)
        .await?;

    let mut items = HashMap::new();

    for row in rows {
        items.insert(row.0, row.1);
    }

    Ok(items)
}

pub fn subscribe_global_stat<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("global-stat-query")
        .handler(handle_registration_succeeded())
        .handler_check_off()
}

#[evento::handler(User)]
async fn handle_registration_succeeded<E: Executor>(
    context: &evento::Context<'_, E>,
    _event: Event<RegistrationSucceeded>,
) -> anyhow::Result<()> {
    // let event_date = time::UtcDateTime::from_unix_timestamp(event.timestamp).unwrap();
    //
    // let total_users_count_key = format!(
    //     "total-users-{}-{}",
    //     event_date.month(),
    //     event_date.year()
    // );
    //
    // let pool = context.extract::<sqlx::SqlitePool>();
    // let statement = Query::insert()
    //     .into_table(GlobalStatPjt::Table)
    //     .columns([GlobalStatPjt::Key, GlobalStatPjt::Value])
    //     // @TODO: fixme, need to be prev month total users count + 1 and not just instead of just 1
    //     .values_panic([total_users_count_key.into(), 1.into()])
    //     .on_conflict(
    //         OnConflict::column(GlobalStatPjt::Key)
    //             .value(GlobalStatPjt::Value, Expr::col(GlobalStatPjt::Value).add(1))
    //             .to_owned(),
    //     )
    //     .to_owned();
    //
    // let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    //
    // sqlx::query_with(&sql, values).execute(&pool).await?;

    let pool = context.extract::<sqlx::SqlitePool>();
    let statement = Query::insert()
        .into_table(GlobalStatPjt::Table)
        .columns([GlobalStatPjt::Key, GlobalStatPjt::Value])
        .values_panic(["total_users".into(), 1.into()])
        .on_conflict(
            OnConflict::column(GlobalStatPjt::Key)
                .value(GlobalStatPjt::Value, Expr::col(GlobalStatPjt::Value).add(1))
                .to_owned(),
        )
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values).execute(&pool).await?;

    Ok(())
}
