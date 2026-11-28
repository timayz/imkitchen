//! What app knows of the tutorial of each user: the steps of it they did,
//! and whether they skipped it.
//!
//! The api keeps it, so it is the same on every device, and notes each step
//! a moment after the command that does it: app follows what it writes. A
//! user who did nothing yet has no row, and every step ahead of them
//! ([`Tutorial::default`]). A tutorial is its user's alone: nothing is read
//! without saying whose is asked for.
//!
//! A tutorial is a projection of its stream, under the id of its user
//! ([`TutorialState`]), whose snapshot is its row of the `tutorials` table,
//! kept by a projection subscription ([`subscription`]). A page loads the
//! tutorial of who reads it from the row and the events written after it
//! ([`of`]).

use evento::Executor;
use evento::metadata::Event;
use evento::projection::{Context, Projection, ProjectionSubscription, Snapshot};
use imkitchen_aggregates::tutorial::{
    Restarted, Skipped, Step, StepCompleted, Tutorial as TutorialStream,
};
use timada_core::db::{FromRow, Json};

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-app/tutorial";

/// A tutorial as its stream leaves it.
#[evento::projection]
#[derive(Debug)]
pub struct TutorialState {
    /// The steps the user did, in the order they did them.
    pub done: Vec<Step>,
    /// The user does not want to be walked through.
    pub skipped: bool,
}

fn projection<E: Executor>(db: Db) -> Projection<E, TutorialState> {
    Projection::new::<TutorialStream>()
        .handler(on_step_completed())
        .handler(on_skipped())
        .handler(on_restarted())
        .data(db)
}

/// Keeps the row of the tutorial of each user in `db` as its stream leaves
/// it.
pub fn subscription<E: Executor>(db: Db) -> ProjectionSubscription<E, TutorialState> {
    projection(db).subscription(SUBSCRIPTION)
}

#[evento::handler]
async fn on_step_completed(
    event: Event<StepCompleted>,
    tutorial: &mut TutorialState,
) -> anyhow::Result<()> {
    if !tutorial.done.contains(&event.data.step) {
        tutorial.done.push(event.data.step);
    }
    Ok(())
}

#[evento::handler]
async fn on_skipped(_event: Event<Skipped>, tutorial: &mut TutorialState) -> anyhow::Result<()> {
    tutorial.skipped = true;
    Ok(())
}

/// No step is done, and it is not skipped.
#[evento::handler]
async fn on_restarted(
    _event: Event<Restarted>,
    tutorial: &mut TutorialState,
) -> anyhow::Result<()> {
    tutorial.done.clear();
    tutorial.skipped = false;
    Ok(())
}

/// The word `step` is named by, as the api words it and as the table keeps
/// it.
pub fn word(step: Step) -> &'static str {
    match step {
        Step::Recipe => "recipe",
        Step::Plat => "plat",
        Step::Settings => "settings",
        Step::List => "list",
        Step::Groceries => "groceries",
    }
}

/// The tutorial of a user.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tutorial {
    /// The steps the user did, in the order they did them.
    pub done: Vec<Step>,
    /// The user does not want to be walked through.
    pub skipped: bool,
}

impl Tutorial {
    /// Whether the user did `step`.
    pub fn did(&self, step: Step) -> bool {
        self.done.contains(&step)
    }

    /// Whether the user did every step.
    pub fn completed(&self) -> bool {
        Step::ALL.iter().all(|step| self.did(*step))
    }

    /// The step the user is at: the first of those ahead of them, in the
    /// order the tutorial goes through them. None once they skipped it, or
    /// did every step.
    pub fn current(&self) -> Option<Step> {
        if self.skipped {
            return None;
        }
        Step::ALL.into_iter().find(|step| !self.did(*step))
    }
}

/// The step `word` names, as the api words it and as the table keeps it.
fn named(word: &str) -> Option<Step> {
    Step::ALL.into_iter().find(|step| self::word(*step) == word)
}

/// The row of a tutorial, and how far its stream was folded into it.
#[derive(FromRow)]
struct Row {
    done: Json<Vec<String>>,
    skipped: i64,
    cursor: String,
    aggregate_version: i64,
}

impl<E: Executor> Snapshot<E> for TutorialState {
    async fn restore(context: &Context<'_, E>) -> anyhow::Result<Option<Self>> {
        let db: Db = context.extract();
        let row: Option<Row> = db
            .fetch_optional(
                "SELECT done, skipped, cursor, aggregate_version FROM tutorials WHERE user_id = ?1",
                (context.id.as_str(),),
            )
            .await?;
        Ok(row.map(|row| TutorialState {
            done: row.done.0.iter().filter_map(|done| named(done)).collect(),
            skipped: row.skipped != 0,
            cursor: row.cursor,
            aggregate_version: row.aggregate_version as u16,
            ..Default::default()
        }))
    }

    async fn take_snapshot(&self, context: &Context<'_, E>) -> anyhow::Result<()> {
        let db: Db = context.extract();
        let done: Vec<&str> = self.done.iter().map(|step| word(*step)).collect();
        db.execute(
            "INSERT INTO tutorials (user_id, done, skipped, cursor, aggregate_version)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT (user_id) DO UPDATE SET done = excluded.done,
                skipped = excluded.skipped, cursor = excluded.cursor,
                aggregate_version = excluded.aggregate_version",
            (
                context.id.as_str(),
                Json(done).to_text()?,
                i64::from(self.skipped),
                self.cursor.as_str(),
                i64::from(self.aggregate_version),
            ),
        )
        .await?;
        Ok(())
    }
}

#[derive(FromRow)]
struct Kept {
    done: Json<Vec<String>>,
    skipped: i64,
}

/// The tutorial of `user_id`, as its row holds it: a moment behind the
/// commands. Every step ahead for a user who did nothing yet.
pub async fn row_of(db: &Db, user_id: &str) -> anyhow::Result<Tutorial> {
    let row: Option<Kept> = db
        .fetch_optional(
            "SELECT done, skipped FROM tutorials WHERE user_id = ?1",
            (user_id,),
        )
        .await?;
    Ok(row.map_or_else(Tutorial::default, |row| Tutorial {
        done: row.done.0.iter().filter_map(|done| named(done)).collect(),
        skipped: row.skipped != 0,
    }))
}

/// The tutorial of `user_id`, as its stream leaves it: its row and the
/// events written after it, read from the api. Every step ahead for a user
/// who did nothing yet.
pub async fn of<E: Executor>(executor: &E, db: &Db, user_id: &str) -> anyhow::Result<Tutorial> {
    let state = projection(db.clone())
        .load(user_id)
        .execute(executor)
        .await?;
    Ok(state.map_or_else(Tutorial::default, |state| Tutorial {
        done: state.done,
        skipped: state.skipped,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_current_step_is_the_first_ahead_in_the_order_of_the_tutorial() {
        let mut tutorial = Tutorial::default();
        assert_eq!(tutorial.current(), Some(Step::Recipe));
        // In any order: the first ahead is the one the user is at.
        tutorial.done = vec![Step::Settings, Step::Recipe];
        assert_eq!(tutorial.current(), Some(Step::Plat));
        assert!(!tutorial.completed());
        tutorial.done = Step::ALL.into_iter().rev().collect();
        assert_eq!(tutorial.current(), None);
        assert!(tutorial.completed());
        // Skipped, no step is ahead: the steps done stay done.
        let skipped = Tutorial {
            done: vec![Step::Recipe],
            skipped: true,
        };
        assert_eq!(skipped.current(), None);
        assert!(skipped.did(Step::Recipe));
    }

    #[test]
    fn each_step_is_named_as_the_api_names_it() {
        let words: Vec<&str> = Step::ALL.into_iter().map(word).collect();
        assert_eq!(words, ["recipe", "plat", "settings", "list", "groceries"]);
    }
}
