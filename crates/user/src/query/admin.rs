use evento::{
    Action, Executor, Projection, SubscriptionBuilder,
    cursor::{Args, ReadResult},
    metadata::Event,
    sql::Reader,
};
use imkitchen_db::table::UserAdmin;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::{SqlitePool, prelude::FromRow};
use std::time::{SystemTime, UNIX_EPOCH};
use strum::{AsRefStr, Display, EnumString, VariantArray};

use crate::{
    Activated, MadeAdmin, Registered, Role, State, Suspended, User,
    subscription::{LifePremiumToggled, Subscription},
};

#[derive(Default, Clone, FromRow)]
pub struct AdminView {
    pub id: String,
    pub email: String,
    pub full_name: Option<String>,
    pub username: Option<String>,
    pub role: sqlx::types::Text<Role>,
    pub state: sqlx::types::Text<State>,
    pub total_recipes_count: i64,
    pub total_active_count: i64,
    pub shared_recipes_count: i64,
    pub subscription_expire_at: u64,
    pub created_at: u64,
}

impl AdminView {
    pub fn is_admin(&self) -> bool {
        self.role.0 == Role::Admin
    }

    pub fn is_premium(&self) -> bool {
        let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else {
            return false;
        };

        self.role.0 == Role::Admin || self.subscription_expire_at > now.as_secs()
    }

    pub fn is_active(&self) -> bool {
        self.state.0 == State::Active
    }

    pub fn is_suspended(&self) -> bool {
        self.state.0 == State::Suspended
    }

    pub fn joined_at(&self) -> String {
        let Ok(created_at) = time::UtcDateTime::from_unix_timestamp(self.created_at as i64) else {
            return "".to_owned();
        };

        let Ok(format) = time::format_description::parse("[month repr:short] [day], [year]") else {
            return "".to_owned();
        };

        created_at.format(&format).unwrap_or_else(|_| "".to_owned())
    }

    pub fn short_name(&self) -> String {
        self.full_name
            .to_owned()
            .unwrap_or(self.email.to_string())
            .split(' ')
            .take(2)
            .map(|w| w.chars().next().unwrap_or('a').to_uppercase().to_string())
            .collect::<Vec<_>>()
            .join("")
    }
}

#[derive(EnumString, Display, VariantArray, Default, Debug, Deserialize, AsRefStr)]
pub enum UserSortBy {
    #[default]
    RecentlyJoined,
    Name,
    MostRecipes,
    MostActive,
}

pub struct FilterQuery {
    pub state: Option<State>,
    pub role: Option<Role>,
    pub sort_by: UserSortBy,
    pub args: Args,
}

pub async fn filter(
    pool: &SqlitePool,
    input: FilterQuery,
) -> anyhow::Result<ReadResult<AdminView>> {
    let mut statement = sea_query::Query::select()
        .columns([
            UserAdmin::Id,
            UserAdmin::Email,
            UserAdmin::FullName,
            UserAdmin::Username,
            UserAdmin::State,
            UserAdmin::Role,
            UserAdmin::SubscriptionExpireAt,
            UserAdmin::TotalRecipesCount,
            UserAdmin::SharedRecipesCount,
            UserAdmin::TotalActiveCount,
            UserAdmin::CreatedAt,
        ])
        .from(UserAdmin::Table)
        .to_owned();

    if let Some(account_type) = input.role {
        statement.and_where(Expr::col(UserAdmin::Role).eq(account_type.to_string()));
    }

    if let Some(status) = input.state {
        statement.and_where(Expr::col(UserAdmin::State).eq(status.to_string()));
    }

    match input.sort_by {
        UserSortBy::RecentlyJoined => {
            let result = Reader::new(statement)
                .desc()
                .args(input.args)
                .execute::<_, UserSortByRecentlyJoined, _>(pool)
                .await?;

            Ok(result.map(|user| user.0))
        }
        UserSortBy::Name => {
            let result = Reader::new(statement)
                .args(input.args)
                .execute::<_, UserSortByName, _>(pool)
                .await?;

            Ok(result.map(|user| user.0))
        }
        UserSortBy::MostActive => {
            let result = Reader::new(statement)
                .desc()
                .args(input.args)
                .execute::<_, UserSortByMostActive, _>(pool)
                .await?;

            Ok(result.map(|user| user.0))
        }
        UserSortBy::MostRecipes => {
            let result = Reader::new(statement)
                .desc()
                .args(input.args)
                .execute::<_, UserSortByMostRecipes, _>(pool)
                .await?;

            Ok(result.map(|user| user.0))
        }
    }
}

pub fn create_projection<E: Executor>() -> Projection<AdminView, E> {
    Projection::new("user-admin-view")
        .handler(handle_actived())
        .handler(handle_susended())
        .handler(handle_made_admin())
        .handler(handle_registered())
        .handler(handle_life_premium_toggled())
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    pool: &'a SqlitePool,
    id: impl Into<String>,
) -> Result<Option<AdminView>, anyhow::Error> {
    let id = id.into();

    Ok(create_projection()
        .no_safety_check()
        .load::<User>(&id)
        .aggregator::<Subscription>(id)
        .data(pool.clone())
        .execute(executor)
        .await?
        .map(|r| r.item))
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<AdminView, E> {
    create_projection().no_safety_check().subscription()
}

#[evento::snapshot]
async fn restore(
    context: &evento::context::RwContext,
    id: String,
) -> anyhow::Result<Option<AdminView>> {
    let pool = context.extract::<SqlitePool>();

    let statement = Query::select()
        .columns([
            UserAdmin::Id,
            UserAdmin::Email,
            UserAdmin::FullName,
            UserAdmin::Username,
            UserAdmin::State,
            UserAdmin::Role,
            UserAdmin::SubscriptionExpireAt,
            UserAdmin::TotalRecipesCount,
            UserAdmin::SharedRecipesCount,
            UserAdmin::TotalActiveCount,
            UserAdmin::CreatedAt,
        ])
        .from(UserAdmin::Table)
        .and_where(Expr::col(UserAdmin::Id).eq(id))
        .limit(1)
        .to_owned();

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    Ok(sqlx::query_as_with(&sql, values)
        .fetch_optional(&pool)
        .await?)
}

#[evento::handler]
async fn handle_registered<E: Executor>(
    event: Event<Registered>,
    action: Action<'_, AdminView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.id = event.aggregator_id.to_owned();
            data.email = event.data.email.to_owned();
            data.role.0 = Role::User;
            data.state.0 = State::Active;
            data.created_at = event.timestamp;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            let statement = Query::insert()
                .into_table(UserAdmin::Table)
                .columns([
                    UserAdmin::Id,
                    UserAdmin::Email,
                    UserAdmin::Role,
                    UserAdmin::State,
                    UserAdmin::CreatedAt,
                ])
                .values_panic([
                    event.aggregator_id.to_owned().into(),
                    event.data.email.to_owned().into(),
                    Role::User.to_string().into(),
                    State::Active.to_string().into(),
                    event.timestamp.into(),
                ])
                .on_conflict(OnConflict::column(UserAdmin::Id).do_nothing().to_owned())
                .to_owned();

            let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
            sqlx::query_with(&sql, values).execute(&pool).await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_made_admin<E: Executor>(
    event: Event<MadeAdmin>,
    action: Action<'_, AdminView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.role.0 = Role::Admin;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(
                &pool,
                UpdateInput {
                    id: event.aggregator_id.to_owned(),
                    role: Some(Role::Admin),
                    state: None,
                    subscription_expire_at: None,
                },
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_actived<E: Executor>(
    event: Event<Activated>,
    action: Action<'_, AdminView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.state.0 = State::Active;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(
                &pool,
                UpdateInput {
                    id: event.aggregator_id.to_owned(),
                    state: Some(State::Active),
                    role: None,
                    subscription_expire_at: None,
                },
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_susended<E: Executor>(
    event: Event<Suspended>,
    action: Action<'_, AdminView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.state.0 = State::Suspended;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(
                &pool,
                UpdateInput {
                    id: event.aggregator_id.to_owned(),
                    state: Some(State::Suspended),
                    role: None,
                    subscription_expire_at: None,
                },
            )
            .await?;
        }
    };

    Ok(())
}

#[evento::handler]
async fn handle_life_premium_toggled<E: Executor>(
    event: Event<LifePremiumToggled>,
    action: Action<'_, AdminView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.subscription_expire_at = event.data.expire_at;
        }
        Action::Handle(context) => {
            let pool = context.extract::<SqlitePool>();
            update(
                &pool,
                UpdateInput {
                    id: event.aggregator_id.to_owned(),
                    state: None,
                    role: None,
                    subscription_expire_at: Some(event.data.expire_at),
                },
            )
            .await?;
        }
    };

    Ok(())
}

struct UpdateInput {
    id: String,
    state: Option<State>,
    role: Option<Role>,
    subscription_expire_at: Option<u64>,
}

async fn update(pool: &SqlitePool, input: UpdateInput) -> anyhow::Result<()> {
    let mut statement = Query::update()
        .table(UserAdmin::Table)
        .and_where(Expr::col(UserAdmin::Id).eq(input.id))
        .to_owned();

    if let Some(role) = input.role {
        statement.value(UserAdmin::Role, role.to_string());
    }

    if let Some(state) = input.state {
        statement.value(UserAdmin::State, state.to_string());
    }

    if let Some(subscription_expire_at) = input.subscription_expire_at {
        statement.value(UserAdmin::SubscriptionExpireAt, subscription_expire_at);
    }

    let (sql, values) = statement.build_sqlx(SqliteQueryBuilder);
    sqlx::query_with(&sql, values).execute(pool).await?;

    Ok(())
}

evento::define_sort_for!(AdminView, UserAdmin, UserAdmin::Id => {
    UserSortByRecentlyJoined: int, UserAdmin::CreatedAt, |s| s.created_at as i64;
    UserSortByMostActive: int, UserAdmin::TotalActiveCount, |s| s.total_active_count;
    UserSortByMostRecipes: int, UserAdmin::TotalRecipesCount, |s| s.total_recipes_count;
    UserSortByName: string, UserAdmin::Email, |s| s.email.to_owned();
});
