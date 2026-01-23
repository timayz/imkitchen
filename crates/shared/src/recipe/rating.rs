#[evento::aggregator]
pub enum Rating {
    LikeChecked { recipe_id: String },
    LikeUnchecked { recipe_id: String },
    UnlikeChecked { recipe_id: String },
    UnlikeUnchecked { recipe_id: String },
    Viewed { recipe_id: String },
}
