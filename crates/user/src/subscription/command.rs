use evento::{
    Executor, Projection, Snapshot,
    metadata::{Event, Metadata},
};
use time::UtcDateTime;
use validator::Validate;

use imkitchen_shared::user::subscription::{LifePremiumToggled, Subscription};

#[evento::command]
pub struct Command {
    pub expire_at: u64,
}

#[derive(Validate)]
pub struct UpdateInput {
    pub user_id: String,
}

impl<'a, E: Executor + Clone> Command<'a, E> {
    pub async fn toggle_life_premium(
        &self,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let now = UtcDateTime::now();
        let expire_at = if self.expire_at > now.unix_timestamp().try_into()? {
            0
        } else {
            (now + time::Duration::weeks(10 * 53)).unix_timestamp()
        };

        self.aggregator()
            .event(&LifePremiumToggled {
                expire_at: expire_at.try_into()?,
            })
            .metadata(&Metadata::new(request_by))
            .commit(self.executor)
            .await?;

        Ok(())
    }
}

fn create_projection(id: impl Into<String>) -> Projection<CommandData> {
    Projection::new::<Subscription>(id)
        .handler(handle_life_premium_toggled())
        .safety_check()
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    id: impl Into<String>,
) -> Result<Command<'a, E>, anyhow::Error> {
    let id = id.into();

    let result = create_projection(&id).execute(executor).await?;

    let cmd = match result {
        Some(data) => Command::new(id, data.get_cursor_version()?, data, executor),
        _ => Command::new(id, 0, Default::default(), executor),
    };

    Ok(cmd)
}

impl evento::Snapshot for CommandData {}

#[evento::handler]
async fn handle_life_premium_toggled(
    event: Event<LifePremiumToggled>,
    data: &mut CommandData,
) -> anyhow::Result<()> {
    data.expire_at = event.data.expire_at;

    Ok(())
}
