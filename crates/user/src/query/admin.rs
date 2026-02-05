use evento::{
    Cursor, Executor, Projection, Snapshot,
    cursor::{Args, ReadResult},
    metadata::Event,
    sql::Reader,
};
use imkitchen_db::table::{UserAdmin, UserAdminFts};
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Deserialize;
use sqlx::{SqlitePool, prelude::FromRow};
use std::time::{SystemTime, UNIX_EPOCH};
use strum::{AsRefStr, Display, EnumString, VariantArray};

use imkitchen_shared::user::{
    Activated, MadeAdmin, Registered, Role, State, Suspended, User, UsernameChanged,
    subscription::{LifePremiumToggled, Subscription},
};

impl<E: Executor> super::Query<E> {
    pub async fn admin(&self, id: impl Into<String>) -> Result<Option<AdminView>, anyhow::Error> {
        load(&self.executor, &self.read_db, &self.write_db, id).await
    }
}

pub(crate) async fn load<E: Executor>(
    executor: &E,
    read_db: &SqlitePool,
    write_db: &SqlitePool,
    id: impl Into<String>,
) -> Result<Option<AdminView>, anyhow::Error> {
    let id = id.into();

    create_projection(&id)
        .aggregator::<Subscription>(id)
        .data((read_db.clone(), write_db.clone()))
        .execute(executor)
        .await
}

#[evento::projection(FromRow, Cursor, Debug)]
pub struct AdminView {
    #[cursor(by_recently_joined, UserAdmin::Id, 1)]
    #[cursor(by_most_active, UserAdmin::Id, 1)]
    #[cursor(by_most_recipes, UserAdmin::Id, 1)]
    #[cursor(by_name, UserAdmin::Id, 1)]
    pub id: String,
    #[cursor(by_name, UserAdmin::Email, 2)]
    pub email: String,
    pub full_name: Option<String>,
    pub username: Option<String>,
    pub role: sqlx::types::Text<Role>,
    pub state: sqlx::types::Text<State>,
    #[cursor(by_most_recipes, UserAdmin::TotalRecipesCount, 2)]
    pub total_recipes_count: i64,
    #[cursor(by_most_active, UserAdmin::TotalActiveCount, 2)]
    pub total_active_count: i64,
    pub shared_recipes_count: i64,
    pub subscription_expire_at: u64,
    #[cursor(by_recently_joined, UserAdmin::CreatedAt, 2)]
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
    pub search: Option<String>,
    pub sort_by: UserSortBy,
    pub args: Args,
}

impl<E: Executor> super::Query<E> {
    pub async fn filter_admin(&self, input: FilterQuery) -> anyhow::Result<ReadResult<AdminView>> {
        let mut statement = sea_query::Query::select()
            .columns([
                UserAdmin::Id,
                UserAdmin::Cursor,
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

        if let Some(search) = input.search {
            statement.and_where(
                Expr::col(UserAdmin::Id).in_subquery(
                    Query::select()
                        .column(UserAdminFts::Id)
                        .from(UserAdminFts::Table)
                        .and_where(Expr::cust(format!("user_admin_fts MATCH '{search}*'")))
                        .order_by(UserAdminFts::Rank, sea_query::Order::Asc)
                        .limit(20)
                        .take(),
                ),
            );
        }

        match input.sort_by {
            UserSortBy::RecentlyJoined => {
                let result = Reader::new(statement)
                    .desc()
                    .args(input.args)
                    .execute::<_, AdminViewByRecentlyJoined, _>(&self.read_db)
                    .await?;

                Ok(result.map(|user| user.0))
            }
            UserSortBy::Name => {
                let result = Reader::new(statement)
                    .args(input.args)
                    .execute::<_, AdminViewByName, _>(&self.read_db)
                    .await?;

                Ok(result.map(|user| user.0))
            }
            UserSortBy::MostActive => {
                let result = Reader::new(statement)
                    .desc()
                    .args(input.args)
                    .execute::<_, AdminViewByMostActive, _>(&self.read_db)
                    .await?;

                Ok(result.map(|user| user.0))
            }
            UserSortBy::MostRecipes => {
                let result = Reader::new(statement)
                    .desc()
                    .args(input.args)
                    .execute::<_, AdminViewByMostRecipes, _>(&self.read_db)
                    .await?;

                Ok(result.map(|user| user.0))
            }
        }
    }
}

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, AdminView> {
    Projection::new::<User>(id)
        .handler(handle_actived())
        .handler(handle_susended())
        .handler(handle_made_admin())
        .handler(handle_registered())
        .handler(handle_life_premium_toggled())
        .handler(handle_username_changed())
}

impl<E: Executor> Snapshot<E> for AdminView {
    async fn restore(context: &evento::projection::Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let (read_db, _) = context.extract::<(SqlitePool, SqlitePool)>();
        let statement = sea_query::Query::select()
            .columns([
                UserAdmin::Id,
                UserAdmin::Cursor,
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
            .and_where(Expr::col(UserAdmin::Id).eq(&context.id))
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

        let statement = Query::insert()
            .into_table(UserAdmin::Table)
            .columns([
                UserAdmin::Id,
                UserAdmin::Cursor,
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
            .values([
                self.id.to_owned().into(),
                self.cursor.to_owned().into(),
                self.email.to_owned().into(),
                self.full_name.to_owned().into(),
                self.username.to_owned().into(),
                self.state.to_string().into(),
                self.role.to_string().into(),
                self.subscription_expire_at.into(),
                self.total_recipes_count.into(),
                self.shared_recipes_count.into(),
                self.total_active_count.into(),
                self.created_at.into(),
            ])?
            .on_conflict(
                OnConflict::column(UserAdmin::Id)
                    .update_columns([
                        UserAdmin::Cursor,
                        UserAdmin::Email,
                        UserAdmin::FullName,
                        UserAdmin::Username,
                        UserAdmin::State,
                        UserAdmin::Role,
                        UserAdmin::SubscriptionExpireAt,
                        UserAdmin::TotalRecipesCount,
                        UserAdmin::SharedRecipesCount,
                        UserAdmin::TotalActiveCount,
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
async fn handle_registered(event: Event<Registered>, data: &mut AdminView) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.email = event.data.email.to_owned();
    data.role.0 = Role::User;
    data.state.0 = State::Active;
    data.created_at = event.timestamp;

    Ok(())
}

#[evento::handler]
async fn handle_made_admin(_event: Event<MadeAdmin>, data: &mut AdminView) -> anyhow::Result<()> {
    data.role.0 = Role::Admin;

    Ok(())
}

#[evento::handler]
async fn handle_actived(_event: Event<Activated>, data: &mut AdminView) -> anyhow::Result<()> {
    data.state.0 = State::Active;

    Ok(())
}

#[evento::handler]
async fn handle_susended(_event: Event<Suspended>, data: &mut AdminView) -> anyhow::Result<()> {
    data.state.0 = State::Suspended;

    Ok(())
}

#[evento::handler]
async fn handle_username_changed(
    event: Event<UsernameChanged>,
    data: &mut AdminView,
) -> anyhow::Result<()> {
    data.username = Some(event.data.value);

    Ok(())
}

#[evento::handler]
async fn handle_life_premium_toggled(
    event: Event<LifePremiumToggled>,
    data: &mut AdminView,
) -> anyhow::Result<()> {
    data.subscription_expire_at = event.data.expire_at;

    Ok(())
}
