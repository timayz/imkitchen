//! The plats: each one a composed dish, made of recipes of the user it
//! belongs to. Like the recipes it names, a plat is its owner's alone.

/// One plat of one user, under a fresh id (`evento::create`).
#[evento::aggregate(name = "imkitchen/Plat")]
pub enum Plat {
    /// Composed by `owner_id`, whose it stays, of recipes of theirs.
    Created {
        owner_id: String,
        name: String,
        /// The recipes it is made of, in the order they are served; none
        /// twice.
        recipe_ids: Vec<String>,
    },
    /// Recomposed by its owner: its name and its recipes, as a whole.
    Edited {
        name: String,
        recipe_ids: Vec<String>,
    },
    /// Gone for good. The recipes it was made of stay.
    Deleted,
    /// One of its recipes was deleted, and is no longer part of it. A plat
    /// may be left with no recipe: it stays, for its owner to recompose.
    RecipeRemoved { recipe_id: String },
}
