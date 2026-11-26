//! The suspensions: the users an administrator keeps out of imkitchen for
//! a while, their account and all that is theirs left as they are.

/// Whether one user is kept out, under the id of the user. A stream that
/// says nothing is of a user who never was.
#[evento::aggregate(name = "imkitchen/Suspension")]
pub enum Suspension {
    /// Kept out by the administrator `by`: no session of theirs signs them
    /// in and none is opened, until they are let in again.
    Suspended { by: String },
    /// Let in again by the administrator `by`.
    Reactivated { by: String },
}
