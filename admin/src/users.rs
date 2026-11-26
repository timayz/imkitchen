//! What admin knows of the users: who they are, how they sign in and the
//! sessions they have.
//!
//! admin keeps one row per user, deleted or not (a deleted account says
//! so), and one per session. Whatever the event, the handler reads the
//! user or the session again from its events, through the api, and leaves
//! the row as it stands: an event delivered twice (the subscription is
//! at-least-once) changes nothing.

use std::collections::HashMap;

use evento::metadata::RawEvent;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::identity::{Session, User};
use timada_core::db::{FromRow, execute, params_from_iter, placeholders, search_key};
use timada_core::like_contains;
use timada_identity::{UserState, session_projection, user_projection};

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-admin/users";

/// Follows the users and their sessions into `db`.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_user_event())
        .handler(on_session_event())
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
    upsert(&db, &user).await
}

/// The row of a user as their events leave it, and the sessions a "sign
/// out everywhere" ended, in one transaction.
async fn upsert(db: &Db, user: &UserState) -> anyhow::Result<()> {
    db.write(user, |tx, user| {
        Box::pin(async move {
            execute(
                tx,
                "INSERT INTO users (id, account, label, is_admin, deleted, registered_at,
                    last_sign_in_at, sign_in_count, oauth2_count, passkey_count, has_password,
                    search)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT (id) DO UPDATE SET
                    account = excluded.account, label = excluded.label,
                    is_admin = excluded.is_admin, deleted = excluded.deleted,
                    registered_at = excluded.registered_at,
                    last_sign_in_at = excluded.last_sign_in_at,
                    sign_in_count = excluded.sign_in_count, oauth2_count = excluded.oauth2_count,
                    passkey_count = excluded.passkey_count,
                    has_password = excluded.has_password, search = excluded.search",
                (
                    user.id.as_str(),
                    user.account.as_str(),
                    user.label.as_str(),
                    user.is_admin,
                    user.deleted,
                    user.registered_at as i64,
                    user.last_sign_in().map(|sign_in| sign_in.at as i64),
                    user.sign_ins.len() as i64,
                    user.oauth2_accounts.len() as i64,
                    user.passkeys.len() as i64,
                    user.password.is_some(),
                    search_key(&format!("{}\n{}", user.account, user.label)),
                ),
            )
            .await?;
            if let Some(revoked) = &user.sessions_revoked {
                execute(
                    tx,
                    "UPDATE user_sessions SET ended = 1
                     WHERE user_id = ?1 AND started_at_ms <= ?2 AND (?3 IS NULL OR id <> ?3)",
                    (
                        user.id.as_str(),
                        revoked.at_ms as i64,
                        revoked.except_session.as_deref(),
                    ),
                )
                .await?;
            }
            Ok(())
        })
    })
    .await
}

#[evento::subscription_all]
async fn on_session_event<E: evento::Executor>(
    context: &Context<'_, E>,
    event: RawEvent<Session>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let Some(session) = session_projection()
        .load(&event.aggregate_id)
        .execute(context.executor)
        .await?
    else {
        return Ok(());
    };
    // Once ended, ended: what a "sign out everywhere" ended stays so.
    db.execute(
        "INSERT INTO user_sessions (id, user_id, started_at, started_at_ms, expires_at, ended,
            user_agent)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT (id) DO UPDATE SET
            expires_at = excluded.expires_at,
            ended = user_sessions.ended OR excluded.ended",
        (
            session.id.as_str(),
            session.user_id.as_str(),
            session.started_at as i64,
            session.started_at_ms as i64,
            session.expires_at as i64,
            session.ended,
            session.user_agent.as_deref(),
        ),
    )
    .await?;
    Ok(())
}

/// A user.
#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct UserRow {
    pub id: String,
    /// The address they sign in with.
    pub account: String,
    /// The name they show.
    pub label: String,
    pub is_admin: bool,
    /// Their account is deleted.
    pub deleted: bool,
    /// Unix seconds.
    pub registered_at: i64,
    /// Unix seconds.
    pub last_sign_in_at: Option<i64>,
    pub sign_in_count: i64,
    /// How many accounts of other sites they sign in with.
    pub oauth2_count: i64,
    pub passkey_count: i64,
    pub has_password: bool,
}

const ROW: &str = "SELECT id, account, label, is_admin, deleted, registered_at, last_sign_in_at,
        sign_in_count, oauth2_count, passkey_count, has_password
    FROM users";

/// A session of a user.
#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct SessionRow {
    /// Unix seconds.
    pub started_at: i64,
    /// Unix seconds.
    pub expires_at: i64,
    pub ended: bool,
    /// The browser it was opened from, as it names itself.
    pub user_agent: Option<String>,
}

impl SessionRow {
    /// Whether it still signs its user in at `now` (unix seconds).
    pub fn is_active_at(&self, now: i64) -> bool {
        !self.ended && self.expires_at > now
    }
}

/// How many users there are whose account is not deleted, as far as admin
/// has followed.
pub async fn count(db: &Db) -> anyhow::Result<u64> {
    let count: i64 = db
        .fetch_scalar("SELECT count(*) FROM users WHERE deleted = 0", ())
        .await?;
    Ok(count.max(0) as u64)
}

/// How many of them are administrators.
pub async fn administrators(db: &Db) -> anyhow::Result<u64> {
    let count: i64 = db
        .fetch_scalar(
            "SELECT count(*) FROM users WHERE deleted = 0 AND is_admin = 1",
            (),
        )
        .await?;
    Ok(count.max(0) as u64)
}

fn pattern(q: Option<&str>) -> Option<String> {
    q.map(|q| like_contains(&search_key(q)))
}

/// How many users `q` is found in, deleted or not: everyone without one.
pub async fn found(db: &Db, q: Option<&str>) -> anyhow::Result<u64> {
    let count: i64 = db
        .fetch_scalar(
            "SELECT count(*) FROM users WHERE ?1 IS NULL OR search LIKE ?1 ESCAPE '\\'",
            (pattern(q),),
        )
        .await?;
    Ok(count.max(0) as u64)
}

/// The users `q` is found in (their address or their name, whatever the
/// case and the accents), by name, those whose account is deleted last:
/// `limit` of them from the `offset`th.
pub async fn list(
    db: &Db,
    q: Option<&str>,
    offset: i64,
    limit: i64,
) -> anyhow::Result<Vec<UserRow>> {
    db.fetch_all(
        &format!(
            "{ROW} WHERE ?1 IS NULL OR search LIKE ?1 ESCAPE '\\'
             ORDER BY deleted, label COLLATE NOCASE, id LIMIT ?2 OFFSET ?3"
        ),
        (pattern(q), limit, offset),
    )
    .await
}

/// The user `id`, if admin knows of one.
pub async fn get(db: &Db, id: &str) -> anyhow::Result<Option<UserRow>> {
    db.fetch_optional(&format!("{ROW} WHERE id = ?1"), (id,))
        .await
}

/// Every session admin knows of the user `id`, the most recent first.
pub async fn sessions(db: &Db, id: &str) -> anyhow::Result<Vec<SessionRow>> {
    db.fetch_all(
        "SELECT started_at, expires_at, ended, user_agent FROM user_sessions
         WHERE user_id = ?1 ORDER BY started_at DESC, id",
        (id,),
    )
    .await
}

#[derive(FromRow)]
struct Name {
    id: String,
    label: String,
}

/// The names of the users `ids`, by id: those admin knows of.
pub async fn names(
    db: &Db,
    ids: impl IntoIterator<Item = &str>,
) -> anyhow::Result<HashMap<String, String>> {
    let mut ids: Vec<&str> = ids.into_iter().collect();
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let names: Vec<Name> = db
        .fetch_all(
            &format!(
                "SELECT id, label FROM users WHERE id IN ({})",
                placeholders(ids.len())
            ),
            params_from_iter(ids),
        )
        .await?;
    Ok(names
        .into_iter()
        .map(|name| (name.id, name.label))
        .collect())
}
