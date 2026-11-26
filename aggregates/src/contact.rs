//! The messages written to imkitchen: by a visitor of www, who has no
//! account, or by a user.

/// One message, under an id of its own.
#[evento::aggregate(name = "imkitchen/ContactMessage")]
pub enum ContactMessage {
    /// Someone wrote to imkitchen.
    Sent {
        /// Who writes, as they name themselves.
        name: String,
        /// Where they are answered. Nothing proves it is theirs.
        email: String,
        message: String,
        /// The language they read, as its tag (`en`, `fr`): the one to
        /// answer them in.
        language: String,
    },
}
