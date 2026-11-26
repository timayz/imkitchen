//! The imports of recipes from pages of the web: each try, whether it gave
//! a recipe or not.
//!
//! A recipe says nothing of where it came from, and nothing here changes
//! what it is to its owner. The tries are kept apart for the people who
//! run imkitchen: which sites are imported from, and which the api fails
//! to read.

/// One try at importing a recipe, under an id of its own.
#[evento::aggregate(name = "imkitchen/RecipeImport")]
pub enum RecipeImport {
    /// The page became a recipe of `user_id`.
    Imported {
        user_id: String,
        /// The address of the page, as the user gave it.
        url: String,
        /// Whether the client sent the page as its browser got it, in
        /// place of the api fetching it.
        page_sent: bool,
        recipe_id: String,
    },
    /// The page gave `user_id` no recipe.
    Failed {
        user_id: String,
        /// The address of the page, as the user gave it.
        url: String,
        /// Whether the client sent the page as its browser got it, in
        /// place of the api fetching it.
        page_sent: bool,
        /// Why, as the word the api refused the import with:
        /// `page_unreachable`, `no_recipe_on_page`, …
        reason: String,
    },
}
