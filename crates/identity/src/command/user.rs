use bitcode::{Decode, Encode};
use evento::{Executor, Projection, ProjectionAggregator, metadata::Event};
use imkitchen_shared::user::{
    self, Activated, LoggedIn, Logout, MadeAdmin, Registered, Role, State, Suspended,
    UsernameChanged,
};
use std::ops::Deref;

use crate::repository::{self};

mod activate;
mod login;
mod made_admin;
mod register;
mod set_username;
mod suspend;

pub use login::LoginInput;
pub use register::RegisterInput;
pub use set_username::SetUsernameInput;

static RE_ALPHA_NUM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9_]+$").unwrap());

#[derive(Clone)]
pub struct Command<E: Executor> {
    state: imkitchen_shared::State<E>,
    pub subscription: crate::subscription::Command<E>,
    pub meal_preferences: crate::meal_preferences::Command<E>,
    pub password: crate::password::Command<E>,
}

impl<E: Executor> Deref for Command<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<E: Executor> Command<E> {
    pub fn new(state: imkitchen_shared::State<E>) -> Self
    where
        imkitchen_shared::State<E>: Clone,
    {
        Self {
            subscription: crate::subscription::Command(state.clone()),
            meal_preferences: crate::meal_preferences::Command(state.clone()),
            password: crate::password::Command(state.clone()),
            state,
        }
    }
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Option<User>> {
        create_projection(id).execute(&self.executor).await
    }

    pub async fn suspend(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let Some(user) = self.load(id).await? else {
            imkitchen_shared::not_found!("user");
        };

        if user.state == State::Suspended {
            return Ok(());
        }

        user.aggregator()?
            .event(&Suspended)
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }

    pub async fn activate(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let Some(user) = self.load(id).await? else {
            imkitchen_shared::not_found!("user");
        };

        if user.state == State::Active {
            return Ok(());
        }

        user.aggregator()?
            .event(&Activated)
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;

        Ok(())
    }

    pub async fn logout(
        &self,
        id: impl Into<String>,
        access_id: String,
    ) -> imkitchen_shared::Result<String> {
        let Some(user) = self.load(id).await? else {
            imkitchen_shared::not_found!("user in logout");
        };

        user.aggregator()?
            .event(&Logout {
                access_id: access_id.to_owned(),
            })
            .commit(&self.executor)
            .await?;

        Ok(access_id)
    }

    pub async fn made_admin(&self, id: impl Into<String>) -> imkitchen_shared::Result<()> {
        let Some(user) = self.load(id).await? else {
            imkitchen_shared::not_found!("user");
        };

        if user.role == Role::Admin {
            return Ok(());
        }

        user.aggregator()?
            .event(&MadeAdmin)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}

#[derive(Validate)]
pub struct LoginInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub lang: String,
    pub timezone: String,
    pub user_agent: String,
}

impl<E: Executor> super::Command<E> {
    pub async fn login(&self, input: LoginInput) -> imkitchen_shared::Result<(String, String)> {
        input.validate()?;

        let Some(user_row) =
            repository::find(&self.read_db, repository::FindType::Email(input.email)).await?
        else {
            imkitchen_shared::user!("Invalid email or password. Please try again.");
        };

        let Some(user) = self.load(&user_row.id).await? else {
            imkitchen_shared::server!("User not found in login");
        };

        let parsed_hash = PasswordHash::new(&user_row.password)?;
        let argon2 = Argon2::default();

        if argon2
            .verify_password(input.password.as_bytes(), &parsed_hash)
            .is_err()
        {
            imkitchen_shared::user!("Invalid email or password. Please try again.");
        }

        if user.state == State::Suspended {
            imkitchen_shared::user!("Account suspended");
        }

        let access_id = Ulid::new().to_string();

        user.aggregator()?
            .event(&LoggedIn {
                lang: input.lang,
                timezone: input.timezone,
                user_agent: input.user_agent,
                access_id: access_id.to_owned(),
            })
            .commit(&self.executor)
            .await?;

        Ok((user_row.id, access_id))
    }
}

#[derive(Validate)]
pub struct RegisterInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 20))]
    pub password: String,
    pub lang: String,
    pub timezone: String,
}

impl<E: Executor> super::Command<E> {
    pub async fn register(&self, input: RegisterInput) -> imkitchen_shared::Result<String> {
        input.validate()?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(input.password.as_bytes(), &salt)?
            .to_string();

        if repository::find(
            &self.read_db,
            repository::FindType::Email(input.email.to_owned()),
        )
        .await?
        .is_some()
        {
            imkitchen_shared::user!("Email already exists");
        }

        let id = evento::create()
            .event(&Registered {
                email: input.email.to_owned(),
                lang: input.lang,
                timezone: input.timezone,
            })
            .commit(&self.executor)
            .await?;

        repository::create(&self.write_db, id.to_owned(), input.email, password_hash).await?;

        Ok(id)
    }
}

#[derive(Validate)]
pub struct SetUsernameInput {
    #[validate(length(min = 3, max = 15), regex(path = *RE_ALPHA_NUM, message = "Only letters (A-Z, a-z) and numbers (0-9) are allowed."))]
    pub username: String,
}

impl<E: Executor> super::Command<E> {
    pub async fn set_username(
        &self,
        id: impl Into<String>,
        username: String,
    ) -> imkitchen_shared::Result<()> {
        let input = SetUsernameInput { username };
        input.validate()?;

        let Some(user) =
            repository::find(&self.read_db, repository::FindType::Id(id.into())).await?
        else {
            imkitchen_shared::not_found!("user not found");
        };

        if user.username.is_some() {
            imkitchen_shared::user!("Username has already been set");
        }

        if repository::is_username_exists(&self.read_db, &input.username).await? {
            imkitchen_shared::user!("Username already used");
        }

        repository::update(
            &self.write_db,
            repository::UpdateInput {
                id: user.id.to_owned(),
                username: Some(input.username.to_owned()),
                password: None,
                role: None,
                state: None,
            },
        )
        .await?;

        let Some(user) = self.load(&user.id).await? else {
            imkitchen_shared::server!("user in set_username");
        };

        user.aggregator()?
            .event(&UsernameChanged {
                value: input.username,
            })
            .requested_by(user.id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}

#[evento::projection(Encode, Decode)]
pub struct User {
    pub id: String,
    pub role: Role,
    pub state: State,
}

pub fn create_projection<E: Executor>(id: impl Into<String>) -> Projection<E, User> {
    Projection::new::<user::User>(id)
        .handler(handle_registered())
        .handler(handle_actived())
        .handler(handle_susended())
        .handler(handle_made_admin())
        .skip::<LoggedIn>()
        .skip::<Logout>()
        .skip::<UsernameChanged>()
        .safety_check()
}

impl ProjectionAggregator for User {
    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}

#[evento::handler]
async fn handle_registered(event: Event<Registered>, data: &mut User) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.state = State::Active;
    data.role = Role::User;

    Ok(())
}

#[evento::handler]
async fn handle_made_admin(_event: Event<MadeAdmin>, data: &mut User) -> anyhow::Result<()> {
    data.role = Role::Admin;

    Ok(())
}

#[evento::handler]
async fn handle_actived(_event: Event<Activated>, data: &mut User) -> anyhow::Result<()> {
    data.state = State::Active;

    Ok(())
}

#[evento::handler]
async fn handle_susended(_event: Event<Suspended>, data: &mut User) -> anyhow::Result<()> {
    data.state = State::Suspended;

    Ok(())
}
