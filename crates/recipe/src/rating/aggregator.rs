#[evento::aggregator]
pub enum Rating {
    LikeChecked,
    LikeUnchecked,
    UnlikeChecked,
    UnlikeUnchecked,
    Viewed,

    CommentAdded {
        id: String,
        message: String,
        reply_to: Option<String>,
    },

    CommentLikeCheked {
        comment_id: String,
    },

    CommentLikeUnchecked {
        comment_id: String,
    },

    CommentUnlikeChecked {
        comment_id: String,
    },

    CommentUnlikeUnchecked {
        comment_id: String,
    },
}
