//! Query handlers and projections

pub mod contact;
pub mod user;

pub use contact::{
    count_messages_by_status, get_contact_message, list_contact_messages, subscribe_contact_query,
    ContactMessageRow,
};
pub use user::{
    get_user, get_user_by_email, get_user_status, subscribe_user_query, UserRow, UserStatus,
};
