//! What app knows of the tutorial of each user: the steps of it they did,
//! and whether they skipped it.
//!
//! The api keeps it, so it is the same on every device, and notes each step
//! a moment after the command that does it: app follows what it writes. A
//! user who did nothing yet has no row, and every step ahead of them
//! ([`Tutorial::default`]). A tutorial is its user's alone: nothing is read
//! from the table without saying whose is asked for.
//!
//! The handlers leave the table as it should be when an event is delivered
//! twice (the subscription is at-least-once): a step is done however often
//! it is, the tutorial skipped however often it is, and started over by
//! the last event that says so, whatever came before.

use evento::metadata::Event;
use evento::subscription::{Context, SubscriptionBuilder};
use imkitchen_aggregates::tutorial::{Restarted, Skipped, Step, StepCompleted};
use timada_core::db::{FromRow, Json};

use crate::db::Db;

/// The cursor identity of the subscription, in the store.
pub const SUBSCRIPTION: &str = "imkitchen-app/tutorial";

/// Follows the tutorials into `db`.
pub fn subscription<E: evento::Executor>(db: Db) -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new(SUBSCRIPTION)
        .data(db)
        .handler(on_step_completed())
        .handler(on_skipped())
        .handler(on_restarted())
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

#[derive(FromRow)]
struct Row {
    done: Json<Vec<String>>,
    skipped: i64,
}

/// The tutorial of `user_id`, as far as app has followed: every step ahead
/// for a user who did nothing yet.
pub async fn of(db: &Db, user_id: &str) -> anyhow::Result<Tutorial> {
    let row: Option<Row> = db
        .fetch_optional(
            "SELECT done, skipped FROM tutorials WHERE user_id = ?1",
            (user_id,),
        )
        .await?;
    let Some(row) = row else {
        return Ok(Tutorial::default());
    };
    let named = |done: &String| {
        let mut steps = Step::ALL.into_iter();
        steps.find(|step| word(*step) == done.as_str())
    };
    Ok(Tutorial {
        done: row.done.0.iter().filter_map(named).collect(),
        skipped: row.skipped != 0,
    })
}

async fn write(db: &Db, user_id: &str, tutorial: &Tutorial) -> anyhow::Result<()> {
    let done: Vec<&str> = tutorial.done.iter().map(|step| word(*step)).collect();
    db.execute(
        "INSERT INTO tutorials (user_id, done, skipped) VALUES (?1, ?2, ?3)
         ON CONFLICT (user_id) DO UPDATE SET done = excluded.done, skipped = excluded.skipped",
        (user_id, Json(done).to_text()?, i64::from(tutorial.skipped)),
    )
    .await?;
    Ok(())
}

#[evento::subscription]
async fn on_step_completed<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<StepCompleted>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let mut tutorial = of(&db, &event.aggregate_id).await?;
    if tutorial.did(event.data.step) {
        return Ok(());
    }
    tutorial.done.push(event.data.step);
    write(&db, &event.aggregate_id, &tutorial).await
}

#[evento::subscription]
async fn on_skipped<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Skipped>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    let tutorial = of(&db, &event.aggregate_id).await?;
    let skipped = Tutorial {
        skipped: true,
        ..tutorial
    };
    write(&db, &event.aggregate_id, &skipped).await
}

/// No step is done, and it is not skipped.
#[evento::subscription]
async fn on_restarted<E: evento::Executor>(
    context: &Context<'_, E>,
    event: Event<Restarted>,
) -> anyhow::Result<()> {
    let db: Db = context.extract();
    write(&db, &event.aggregate_id, &Tutorial::default()).await
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
