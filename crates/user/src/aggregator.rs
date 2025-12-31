use crate::{Role, State};

#[evento::aggregator]
pub enum User {
    Registered {
        email: String,
        lang: String,
        timezone: String,
    },
    LoggedIn {
        role: Role,
        state: State,
        username: Option<String>,
        access_id: String,
        lang: String,
        timezone: String,
        user_agent: String,
        subscription_expire_at: u64,
    },
    UsernameChanged {
        value: String,
    },
    Logout {
        access_id: String,
    },
    MadeAdmin,
    Suspended,
    Activated,
}
