#[evento::aggregator]
pub enum CommentRating {
    LikeChecked { comment_id: String },
    LikeUnchecked { comment_id: String },
    UnlikeChecked { comment_id: String },
    UnlikeUnchecked { comment_id: String },
}
