//! What app knows of the settings of each user: how their list is
//! generated, and the language they read.
//!
//! A user who changed nothing has no row: their settings are the defaults,
//! those the api answers ([`Settings::default`]). They are theirs alone:
//! nothing is read without saying whose is asked for.
//!
//! The settings are a projection of their stream, under the id of their
//! user ([`SettingsState`]), whose snapshot is their row of the `settings`
//! table, kept by a projection subscription ([`subscription`]). A page loads
//! those of who reads it from the row and the events written after it
//! ([`of`]), so it shows them as the command its user just handed the api
//! left them.

use evento::Executor;
use evento::metadata::Event;
use evento::projection::{Context, Projection, ProjectionSubscription, Snapshot};
use imkitchen_aggregates::settings::{
    DEFAULT_MEALS, DEFAULT_SERVINGS, LanguageChanged, ListChanged, ListSources,
    Settings as SettingsStream,
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

/// The settings of a user, as their stream leaves them: `None` until they
/// changed one, the defaults for those they did not change.
#[evento::projection]
#[derive(Debug)]
pub struct SettingsState {
    pub settings: Option<Settings>,
}

fn projection<E: Executor>(db: Db) -> Projection<E, SettingsState> {
    Projection::new::<SettingsStream>()
        .handler(on_list_changed())
        .handler(on_language_changed())
        .data(db)
}

/// Keeps the row of the settings of each user in `db` as their stream
/// leaves them.
pub fn subscription<E: Executor>(db: Db) -> ProjectionSubscription<E, SettingsState> {
    projection(db).subscription(SUBSCRIPTION)
}

#[evento::handler]
async fn on_list_changed(
    event: Event<ListChanged>,
    state: &mut SettingsState,
) -> anyhow::Result<()> {
    let data = &event.data;
    let settings = state.settings.get_or_insert_with(Settings::default);
    settings.servings = i64::from(data.servings);
    settings.meals = i64::from(data.meals);
    settings.sources = sources(data.sources).to_owned();
    Ok(())
}

/// A user whose first setting is their language: the rest are the
/// defaults.
#[evento::handler]
async fn on_language_changed(
    event: Event<LanguageChanged>,
    state: &mut SettingsState,
) -> anyhow::Result<()> {
    let settings = state.settings.get_or_insert_with(Settings::default);
    settings.language = event.data.language.to_owned();
    Ok(())
}

/// The row of the settings of a user, and how far their stream was folded
/// into it.
#[derive(FromRow)]
struct Row {
    servings: i64,
    meals: i64,
    sources: String,
    language: Option<String>,
    cursor: String,
    aggregate_version: i64,
}

impl<E: Executor> Snapshot<E> for SettingsState {
    async fn restore(context: &Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let db: Db = context.extract();
        let row: Option<Row> = db
            .fetch_optional(
                "SELECT servings, meals, sources, language, cursor, aggregate_version
                 FROM settings WHERE user_id = ?1",
                (context.id.as_str(),),
            )
            .await?;
        Ok(row.map(|row| SettingsState {
            settings: Some(Settings {
                servings: row.servings,
                meals: row.meals,
                sources: row.sources,
                language: row.language,
            }),
            cursor: row.cursor,
            aggregate_version: row.aggregate_version as u16,
            ..Default::default()
        }))
    }

    async fn take_snapshot(&self, context: &Context<'_, E>) -> anyhow::Result<()> {
        let db: Db = context.extract();
        let settings = self.settings.clone().unwrap_or_default();
        db.execute(
            "INSERT INTO settings (user_id, servings, meals, sources, language, cursor,
                aggregate_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT (user_id) DO UPDATE SET
                servings = excluded.servings, meals = excluded.meals, sources = excluded.sources,
                language = excluded.language, cursor = excluded.cursor,
                aggregate_version = excluded.aggregate_version",
            (
                context.id.as_str(),
                settings.servings,
                settings.meals,
                settings.sources,
                settings.language,
                self.cursor.as_str(),
                i64::from(self.aggregate_version),
            ),
        )
        .await?;
        Ok(())
    }
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

/// The settings of `user_id`, as their row holds them: a moment behind the
/// commands. The defaults for a user who changed none.
pub async fn row_of(db: &Db, user_id: &str) -> anyhow::Result<Settings> {
    let settings: Option<Settings> = db
        .fetch_optional(
            "SELECT servings, meals, sources, language FROM settings WHERE user_id = ?1",
            (user_id,),
        )
        .await?;
    Ok(settings.unwrap_or_default())
}

/// The settings of `user_id`, as their stream leaves them: their row and
/// the events written after it, read from the api. The defaults for a user
/// who changed none.
pub async fn of<E: Executor>(executor: &E, db: &Db, user_id: &str) -> anyhow::Result<Settings> {
    let state = projection(db.clone())
        .load(user_id)
        .execute(executor)
        .await?;
    Ok(state.and_then(|state| state.settings).unwrap_or_default())
}
