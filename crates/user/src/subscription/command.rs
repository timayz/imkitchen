use evento::{
    Action, Executor, Projection,
    metadata::{Event, Metadata},
};
use time::UtcDateTime;
use validator::Validate;

use crate::subscription::{LifePremiumToggled, Subscription};

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

fn create_projection<E: Executor>() -> Projection<CommandData, E> {
    Projection::new("user-subscription-command").handler(handle_life_premium_toggled())
}

pub async fn load<'a, E: Executor>(
    executor: &'a E,
    id: impl Into<String>,
) -> Result<Command<'a, E>, anyhow::Error> {
    let id = id.into();

    Ok(create_projection()
        .no_safety_check()
        .load::<Subscription>(&id)
        .execute_all(executor)
        .await?
        .map(|loaded| Command::new(id.to_owned(), loaded, executor))
        .unwrap_or_else(|| Command::new(id, Default::default(), executor)))
}

impl evento::Snapshot for CommandData {}

#[evento::handler]
async fn handle_life_premium_toggled<E: Executor>(
    event: Event<LifePremiumToggled>,
    action: Action<'_, CommandData, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(data) => {
            data.expire_at = event.data.expire_at;
        }
        Action::Handle(_context) => {}
    };

    Ok(())
}
