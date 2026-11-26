//! The settings: what a user says of how their list is generated and of the
//! language they read. Like everything of a user, they are theirs alone.

use bitcode::{Decode, Encode};

/// The settings of one user, under the id of the user. A stream that says
/// nothing of a setting leaves it as it is by default.
#[evento::aggregate(name = "imkitchen/Settings")]
pub enum Settings {
    /// How the user's list is generated, as a whole.
    ListChanged {
        /// How many people the list is cooked for.
        servings: u32,
        /// How many meals a generated list holds.
        meals: u32,
        sources: ListSources,
    },
    /// The language the user reads, as its tag (`en`, `fr`); `None` for
    /// the one of the device they are on.
    LanguageChanged { language: Option<String> },
}

/// What the meals of a generated list are drawn from. Persisted in events:
/// its variants are frozen as theirs are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Encode, Decode)]
pub enum ListSources {
    /// The user's plats and their recipes.
    #[default]
    PlatsAndRecipes,
    /// Only the user's plats, as composed.
    Plats,
    /// Only the user's recipes.
    Recipes,
}
