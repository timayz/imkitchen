use bincode::{Decode, Encode};
use evento::AggregatorName;

#[derive(AggregatorName, Encode, Decode)]
pub struct LikeChecked {
    pub liked: bool,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct LikeUnchecked {
    pub liked: bool,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct UnlikeChecked {
    pub unliked: bool,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct UnlikeUnchecked {
    pub unliked: bool,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct Viewed {
    pub viewed: bool,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct CommentAdded {
    pub id: String,
    pub message: String,
    pub reply_to: Option<String>,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct CommentLikeCheked {
    pub comment_id: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct CommentLikeUnchecked {
    pub comment_id: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct CommentUnlikeChecked {
    pub comment_id: String,
}

#[derive(AggregatorName, Encode, Decode)]
pub struct CommentUnlikeUnchecked {
    pub comment_id: String,
}
