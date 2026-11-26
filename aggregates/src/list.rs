//! The list: the meals a user plans to cook, generated from their recipes
//! and their plats or picked by hand. Like what it names, it is its
//! owner's alone.

use bitcode::{Decode, Encode};

/// The list of one user, under the id of the user. A user who never filled
/// theirs has no stream, and an empty list.
#[evento::aggregate(name = "imkitchen/List")]
pub enum List {
    /// Generated: these entries are the list, in place of what it held.
    Generated { entries: Vec<Entry> },
    /// A meal was added, after the others.
    EntryAdded { entry: Entry },
    /// The entry `entry.id` is now this, in the place it had.
    EntryReplaced { entry: Entry },
    /// An entry left the list: taken off by the user, or its recipe or
    /// its plat was deleted.
    EntryRemoved { entry_id: String },
}

/// One meal of a list. Persisted in events: its fields are frozen as the
/// variants are.
#[derive(Debug, Clone, PartialEq, Eq, Default, Encode, Decode)]
pub struct Entry {
    /// Its own id: a recipe or a plat may be on a list more than once.
    pub id: String,
    pub meal: Meal,
    /// How many people it is cooked for.
    pub servings: u32,
}

/// What a meal is made from: a recipe of the user, or one of their plats,
/// by its id. Persisted in events: frozen as [`Entry`] is.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode)]
pub enum Meal {
    Recipe(String),
    Plat(String),
}

/// What evento asks of whatever an event holds; no entry is written with
/// it.
impl Default for Meal {
    fn default() -> Self {
        Self::Recipe(String::new())
    }
}
