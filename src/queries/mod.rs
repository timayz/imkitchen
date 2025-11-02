//! Query handlers and projections

pub mod user;

pub use user::{
    get_user, get_user_by_email, get_user_status, subscribe_user_query, UserRow, UserStatus,
};
