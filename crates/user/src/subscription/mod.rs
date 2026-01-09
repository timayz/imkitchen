mod toogle_life_premium;

use evento::{Executor, Projection, cursor, metadata::Event};
use imkitchen_shared::user::subscription;
use std::ops::Deref;

pub struct Command<E: Executor>(pub(crate) imkitchen_shared::State<E>);

impl<E: Executor> Deref for Command<E> {
    type Target = imkitchen_shared::State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Command<E> {
    pub async fn load(&self, id: impl Into<String>) -> anyhow::Result<Subscription> {
        let id = id.into();
        create_projection(&id)
            .execute(&self.executor)
            .await
            .map(|r| {
                r.unwrap_or_else(|| Subscription {
                    id,
                    expire_at: 0,
                    cursor: Default::default(),
                })
            })
    }
}

#[derive(Default)]
pub struct Subscription {
    pub id: String,
    pub expire_at: u64,
    pub cursor: cursor::Value,
}

fn create_projection(id: impl Into<String>) -> Projection<Subscription> {
    Projection::new::<subscription::Subscription>(id)
        .handler(handle_life_premium_toggled())
        .safety_check()
}

impl evento::ProjectionCursor for Subscription {
    fn set_cursor(&mut self, v: &cursor::Value) {
        self.cursor = v.to_owned();
    }

    fn get_cursor(&self) -> cursor::Value {
        self.cursor.to_owned()
    }

    fn aggregator_id(&self) -> String {
        self.id.to_owned()
    }
}

impl evento::Snapshot for Subscription {}

#[evento::handler]
async fn handle_life_premium_toggled(
    event: Event<subscription::LifePremiumToggled>,
    data: &mut Subscription,
) -> anyhow::Result<()> {
    data.id = event.aggregator_id.to_owned();
    data.expire_at = event.data.expire_at;

    Ok(())
}
