//! What admin knows of the users: how many there are.
//!
//! admin keeps one row per user whose account is not deleted, holding its
//! id and nothing else, for the dashboard to say how many imkitchen has.
//!
//! Whatever the event, the handler reads the user again from its events,
//! through the api, and leaves the row as the user stands: an event
//! delivered twice (the subscription is at-least-once) changes nothing.

use evento::metadata::RawEvent;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::identity::User;
use timada_identity::user_projection;

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-admin/users";

/// Follows the users into `db`.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_user_event())
}

#[evento::subscription_all]
async fn on_user_event<E: evento::Executor>(
    context: &Context<'_, E>,
    event: RawEvent<User>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let Some(user) = user_projection()
        .load(&event.aggregate_id)
        .execute(context.executor)
        .await?
    else {
        return Ok(());
    };
    if user.deleted {
        db.execute("DELETE FROM users WHERE id = ?1", (user.id.as_str(),))
            .await?;
    } else {
        db.execute(
            "INSERT INTO users (id) VALUES (?1) ON CONFLICT (id) DO NOTHING",
            (user.id.as_str(),),
        )
        .await?;
    }
    Ok(())
}

/// How many users there are, as far as admin has followed.
pub async fn count(db: &Db) -> anyhow::Result<u64> {
    let count: i64 = db.fetch_scalar("SELECT count(*) FROM users", ()).await?;
    Ok(count.max(0) as u64)
}
