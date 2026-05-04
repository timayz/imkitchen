#[evento::aggregator]
pub enum RecipeShare {
    AllSharedToCommunity { owner_name: String },
    AllMadePrivate,
}
