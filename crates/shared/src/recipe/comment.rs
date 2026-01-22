#[evento::aggregator]
pub enum Comment {
    Added {
        recipe_id: String,
        body: String,
        owner_name: Option<String>,
        reply_to: Option<String>,
    },
}
