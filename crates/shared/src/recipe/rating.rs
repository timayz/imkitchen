#[evento::aggregator]
pub enum Rating {
    LikeChecked {
        recipe_id: String,
    },
    LikeUnchecked {
        recipe_id: String,
    },
    UnlikeChecked {
        recipe_id: String,
    },
    UnlikeUnchecked {
        recipe_id: String,
    },
    Viewed {
        recipe_id: String,
    },

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
