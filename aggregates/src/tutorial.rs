//! The tutorial: how far a new user is through what the app walks them
//! through. Like everything of a user, it is theirs alone, and the same on
//! every device they sign in on.

use bitcode::{Decode, Encode};

/// The tutorial of one user, under the id of the user. A user who did
/// nothing yet has no stream, and every step ahead of them.
#[evento::aggregate(name = "imkitchen/Tutorial")]
pub enum Tutorial {
    /// The user did what `step` asks: written by the api when it sees the
    /// command go by, never by the user.
    StepCompleted { step: Step },
    /// The user does not want to be walked through: the steps left are
    /// left.
    Skipped,
    /// The user starts over: no step is done, and it is not skipped.
    Restarted,
}

/// What the tutorial walks a user through, each step done by the command
/// it names. Persisted in events: its variants are frozen as theirs are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Encode, Decode)]
pub enum Step {
    /// Write a recipe, or import one.
    #[default]
    Recipe,
    /// Compose a plat.
    Plat,
    /// Set how the list is generated, or the language.
    Settings,
    /// Generate the list, or pick a meal.
    List,
    /// Check an item of the groceries, or adjust one.
    Groceries,
}

impl Step {
    /// Every step, in the order the tutorial goes through them.
    pub const ALL: [Step; 5] = [
        Step::Recipe,
        Step::Plat,
        Step::Settings,
        Step::List,
        Step::Groceries,
    ];
}
