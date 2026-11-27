//! What app knows of the settings of each user: how their list is
//! generated, and the language they read.
//!
//! A user who changed nothing has no row: their settings are the defaults,
//! those the api answers ([`Settings::default`]). They are theirs alone:
//! nothing is read from the table without saying whose is asked for.
//!
//! The handlers leave the table as it should be when an event is delivered
//! twice (the subscription is at-least-once): each sets its part of the
//! row as the event says, whatever the row held.

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::settings::{
    DEFAULT_MEALS, DEFAULT_SERVINGS, LanguageChanged, ListChanged, ListSources,
};
use timada_core::db::FromRow;

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-app/settings";

/// What the meals of a generated list are drawn from, as the api words
/// them and as the table keeps them: the first is the default.
pub const SOURCES: [&str; 3] = ["plats_and_recipes", "plats", "recipes"];

fn sources(sources: ListSources) -> &'static str {
    match sources {
        ListSources::PlatsAndRecipes => SOURCES[0],
        ListSources::Plats => SOURCES[1],
        ListSources::Recipes => SOURCES[2],
    }
}

/// Follows the settings into `db`.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_list_changed())
        .handler(on_language_changed())
}

#[evento::subscription]
async fn on_list_changed<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<ListChanged>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let data = &event.data;
    db.execute(
        "INSERT INTO settings (user_id, servings, meals, sources) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (user_id) DO UPDATE SET
            servings = excluded.servings, meals = excluded.meals, sources = excluded.sources",
        (
            event.aggregate_id.as_str(),
            data.servings,
            data.meals,
            sources(data.sources),
        ),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_language_changed<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<LanguageChanged>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let defaults = Settings::default();
    // A user whose first setting is their language: the rest of the row is
    // the defaults.
    db.execute(
        "INSERT INTO settings (user_id, servings, meals, sources, language)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (user_id) DO UPDATE SET language = excluded.language",
        (
            event.aggregate_id.as_str(),
            defaults.servings,
            defaults.meals,
            defaults.sources.as_str(),
            event.data.language.as_deref(),
        ),
    )
    .await?;
    Ok(())
}

/// The settings of a user.
#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct Settings {
    /// How many people the list is cooked for.
    pub servings: i64,
    /// How many meals a generated list holds.
    pub meals: i64,
    /// What its meals are drawn from: one of [`SOURCES`].
    pub sources: String,
    /// The tag of the language the user reads (`fr`); `None` for the one
    /// of the device they are on.
    pub language: Option<String>,
}

impl Default for Settings {
    /// Those of a user who said nothing, as the api has them.
    fn default() -> Self {
        Self {
            servings: i64::from(DEFAULT_SERVINGS),
            meals: i64::from(DEFAULT_MEALS),
            sources: sources(ListSources::default()).to_owned(),
            language: None,
        }
    }
}

/// The settings of `user_id`, as far as app has followed: the defaults for
/// a user who changed none.
pub async fn of(db: &Db, user_id: &str) -> anyhow::Result<Settings> {
    let settings: Option<Settings> = db
        .fetch_optional(
            "SELECT servings, meals, sources, language FROM settings WHERE user_id = ?1",
            (user_id,),
        )
        .await?;
    Ok(settings.unwrap_or_default())
}
