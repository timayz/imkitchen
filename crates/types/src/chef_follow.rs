/// A user following a chef. One stream per (chef, follower) pair, keyed by
/// `evento::hash_ids([chef_id, user_id])`; the follower is `requested_by`.
#[evento::aggregate]
pub enum ChefFollow {
    Followed {
        chef_id: String,
        /// The chef's username at follow time (the `/cooks/{username}` link).
        chef_name: String,
    },
    Unfollowed {
        chef_id: String,
    },
}
