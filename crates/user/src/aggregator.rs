#[evento::aggregator]
pub enum User {
    Registered {
        email: String,
        lang: String,
        timezone: String,
    },
    LoggedIn {
        access_id: String,
        lang: String,
        timezone: String,
        user_agent: String,
    },
    Logout {
        access_id: String,
    },
    MadeAdmin,
    Suspended,
    Activated,
}
