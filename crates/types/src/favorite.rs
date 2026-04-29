#[evento::aggregator]
pub enum Favorite {
    Saved {
        recipe_id: String,
        recipe_owner: String,
    },
    Unsaved {
        recipe_id: String,
    },
}
