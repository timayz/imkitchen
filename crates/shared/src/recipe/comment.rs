#[evento::aggregator]
pub enum Comment {
    Added {
        recipe_id: String,
        message: String,
        reply_to: Option<String>,
    },
}
