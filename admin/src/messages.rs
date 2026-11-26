//! What admin knows of the messages written to imkitchen: each one, as it
//! was written.
//!
//! admin keeps one row per message, which is one event: written once
//! however often the event is delivered (the subscription is
//! at-least-once).

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::contact::Sent;
use timada_core::db::FromRow;

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-admin/contact-messages";

/// Follows the messages into `db`.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_sent())
}

#[evento::subscription]
async fn on_sent<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Sent>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let data = &event.data;
    db.execute(
        "INSERT INTO contact_messages (id, name, email, message, language, sent_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT (id) DO NOTHING",
        (
            event.aggregate_id.as_str(),
            data.name.as_str(),
            data.email.as_str(),
            data.message.as_str(),
            data.language.as_str(),
            event.timestamp as i64,
        ),
    )
    .await?;
    Ok(())
}

/// A message written to imkitchen.
#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct MessageRow {
    pub id: String,
    /// Who writes, as they name themselves.
    pub name: String,
    /// Where they are answered. Nothing proves it is theirs.
    pub email: String,
    pub message: String,
    /// The language they read, as its tag: the one to answer them in.
    pub language: String,
    /// Unix seconds.
    pub sent_at: i64,
}

/// How many messages were written, as far as admin has followed.
pub async fn count(db: &Db) -> anyhow::Result<u64> {
    let count: i64 = db
        .fetch_scalar("SELECT count(*) FROM contact_messages", ())
        .await?;
    Ok(count.max(0) as u64)
}

/// The messages, the most recent first: `limit` of them from the
/// `offset`th.
pub async fn list(db: &Db, offset: i64, limit: i64) -> anyhow::Result<Vec<MessageRow>> {
    db.fetch_all(
        "SELECT id, name, email, message, language, sent_at FROM contact_messages
         ORDER BY sent_at DESC, id DESC LIMIT ?1 OFFSET ?2",
        (limit, offset),
    )
    .await
}

/// The message `id`, if there is one.
pub async fn get(db: &Db, id: &str) -> anyhow::Result<Option<MessageRow>> {
    db.fetch_optional(
        "SELECT id, name, email, message, language, sent_at FROM contact_messages WHERE id = ?1",
        (id,),
    )
    .await
}
