use clap::ValueEnum;
use imkitchen_shared::Metadata;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Role {
    User,
    Admin,
    Suspend,
}

pub async fn set_role(
    config: crate::config::Config,
    email: String,
    role: Role,
) -> anyhow::Result<()> {
    // Set up database connection pool with optimized PRAGMAs
    let pool = imkitchen::create_pool(&config.database.url, 1).await?;
    let evento: evento::Sqlite = pool.clone().into();
    let command = imkitchen_user::Command(evento, pool.clone());

    let Some(user) = command.get_user_by_email(&email).await? else {
        tracing::error!("user {email} not found");
        return Ok(());
    };

    match role {
        Role::User => command.activate(user.id, Metadata::default()).await?,
        Role::Suspend => command.suspend(user.id, Metadata::default()).await?,
        Role::Admin => command.made_admin(user.id, Metadata::default()).await?,
    }

    tracing::info!("{email} now have admin access");

    Ok(())
}
