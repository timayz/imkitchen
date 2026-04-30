#[evento::aggregator]
pub enum Comment {
    Added {
        recipe_id: String,
        body: String,
        owner_name: String,
    },
    Replied {
        recipe_id: String,
        body: String,
        owner_name: String,
    },
}
