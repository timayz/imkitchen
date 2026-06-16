#[evento::aggregate]
pub enum RecipeShare {
    AllSharedToCommunity { owner_name: String },
    AllMadePrivate,
}
