#[evento::aggregator]
pub enum Password {
    ResetRequested {
        user_id: String,
        email: String,
        lang: String,
        host: String,
    },
    ResetCompleted,
}
