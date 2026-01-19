use bitcode::{Decode, Encode};
use evento::{Executor, Projection, Snapshot, metadata::Event};
use imkitchen_db::table::UserLogin;
use sea_query::{Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::{SqlitePool, prelude::FromRow};

use imkitchen_shared::user::{
    Activated, LoggedIn, Logout, MadeAdmin, Role, State, Suspended, User, UsernameChanged,
    password::ResetCompleted,
    subscription::{LifePremiumToggled, Subscription},
};

impl<E: Executor> super::Query<E> {
    pub async fn login(&self, id: impl Into<String>) -> Result<Option<LoginView>, anyhow::Error> {
        let id = id.into();

        create_projection(&id)
            .data((self.read_db.clone(), self.write_db.clone()))
            .execute(&self.executor)
            .await
    }
}

#[derive(Default, Clone, Debug, Encode, Decode)]
pub struct Login {
    pub id: String,
    pub user_agent: String,
    pub role: Role,
    pub state: State,
    pub username: Option<String>,
    pub subscription_expire_at: u64,
    pub tz: String,
}

#[evento::projection(Debug, FromRow)]
pub struct LoginView {
    pub id: String,
    pub role: sqlx::types::Text<Role>,
    pub state: sqlx::types::Text<State>,
    pub username: Option<String>,
    pub subscription_expire_at: u64,
    pub logins: evento::sql_types::Bitcode<Vec<Login>>,
}

impl Login {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }

    pub fn is_premium(&self) -> bool {
        let Ok(now): Result<u64, _> = time::UtcDateTime::now().unix_timestamp().try_into() else {
            return false;
        };

        self.subscription_expire_at > now
    }

    pub fn username(&self) -> String {
        self.username.to_owned().unwrap_or("john_doe".to_owned())
    }
}

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, LoginView> {
    let id = id.into();

    Projection::new::<User>(&id)
        .aggregator::<Subscription>(id)
        .handler(handle_logged_in())
        .handler(handle_logout())
        .handler(handle_actived())
        .handler(handle_susended())
        .handler(handle_made_admin())
        .handler(handle_reset_completed())
        .handler(handle_username_changed())
        .handler(handle_life_premium_toggled())
}

impl<E: Executor> Snapshot<E> for LoginView {
    async fn restore(context: &evento::projection::Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let (read_db, _) = context.extract::<(SqlitePool, SqlitePool)>();
        let statement = sea_query::Query::select()
            .columns([
                UserLogin::Id,
                UserLogin::Cursor,
                UserLogin::Username,
                UserLogin::State,
                UserLogin::Role,
                UserLogin::SubscriptionExpireAt,
                UserLogin::Logins,
            ])
            .from(UserLogin::Table)
            .and_where(Expr::col(UserLogin::Id).eq(&context.id))
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
        let logins = bitcode::encode(&self.logins.0);

        let statement = Query::insert()
            .into_table(UserLogin::Table)
            .columns([
                UserLogin::Id,
                UserLogin::Cursor,
                UserLogin::Username,
                UserLogin::State,
                UserLogin::Role,
                UserLogin::SubscriptionExpireAt,
                UserLogin::Logins,
            ])
            .values([
                self.id.to_owned().into(),
                self.cursor.to_owned().into(),
                self.username.to_owned().into(),
                self.state.to_string().into(),
                self.role.to_string().into(),
                self.subscription_expire_at.into(),
                logins.into(),
            ])?
            .on_conflict(
                OnConflict::column(UserLogin::Id)
                    .update_columns([
                        UserLogin::Cursor,
                        UserLogin::Username,
                        UserLogin::State,
                        UserLogin::Role,
                        UserLogin::SubscriptionExpireAt,
                        UserLogin::Logins,
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
async fn handle_username_changed(
    event: Event<UsernameChanged>,
    data: &mut LoginView,
) -> anyhow::Result<()> {
    data.username = Some(event.data.value.to_owned());

    for login in data.logins.iter_mut() {
        login.username = Some(event.data.value.to_owned());
    }

    Ok(())
}

#[evento::handler]
async fn handle_logged_in(event: Event<LoggedIn>, data: &mut LoginView) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.logins
        .retain(|r| r.user_agent != event.data.user_agent);
    data.logins.push(Login {
        id: event.data.access_id,
        role: data.role.0.to_owned(),
        state: data.state.0.to_owned(),
        subscription_expire_at: data.subscription_expire_at,
        username: data.username.to_owned(),
        user_agent: event.data.user_agent,
        tz: event.data.timezone,
    });

    Ok(())
}

#[evento::handler]
async fn handle_logout(event: Event<Logout>, data: &mut LoginView) -> anyhow::Result<()> {
    data.logins.retain(|r| r.id != event.data.access_id);

    Ok(())
}

#[evento::handler]
async fn handle_reset_completed(
    event: Event<ResetCompleted>,
    data: &mut LoginView,
) -> anyhow::Result<()> {
    if data.id == event.metadata.requested_by()? {
        data.logins = vec![].into();
    }

    Ok(())
}

#[evento::handler]
async fn handle_made_admin(_event: Event<MadeAdmin>, data: &mut LoginView) -> anyhow::Result<()> {
    data.role.0 = Role::Admin;
    for login in data.logins.iter_mut() {
        login.role = Role::Admin;
    }

    Ok(())
}

#[evento::handler]
async fn handle_actived(_event: Event<Activated>, data: &mut LoginView) -> anyhow::Result<()> {
    data.state.0 = State::Active;
    for login in data.logins.iter_mut() {
        login.state = State::Active;
    }

    Ok(())
}

#[evento::handler]
async fn handle_susended(_event: Event<Suspended>, data: &mut LoginView) -> anyhow::Result<()> {
    data.state.0 = State::Suspended;
    for login in data.logins.iter_mut() {
        login.state = State::Suspended;
    }

    Ok(())
}

#[evento::handler]
async fn handle_life_premium_toggled(
    event: Event<LifePremiumToggled>,
    data: &mut LoginView,
) -> anyhow::Result<()> {
    data.subscription_expire_at = event.data.expire_at;
    for login in data.logins.iter_mut() {
        login.subscription_expire_at = event.data.expire_at;
    }

    Ok(())
}
